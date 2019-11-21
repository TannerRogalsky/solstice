use glsl::syntax::{ArraySpecifier, TypeSpecifierNonArray};
use syn::export::TokenStream;

#[proc_macro]
pub fn glsl_derive(input: TokenStream) -> TokenStream {
    use glsl::{
        parser::Parse,
        syntax::{
            Expr, Identifier, SingleDeclaration, StorageQualifier, TranslationUnit,
            TypeQualifierSpec, TypeSpecifier,
        },
        visitor::{Host, Visit, Visitor},
    };

    let input = syn::parse_macro_input!(input as syn::LitStr);
    let mut tu = TranslationUnit::parse(input.value()).unwrap();

    struct StorageLister {
        uniforms: Vec<(Identifier, TypeSpecifier, Option<ArraySpecifier>)>,
    }

    impl Visitor for StorageLister {
        fn visit_single_declaration(&mut self, declaration: &mut SingleDeclaration) -> Visit {
            if let Some(name) = &declaration.name {
                if let Some(qualifier) = &declaration.ty.qualifier {
                    match &qualifier.qualifiers.0[0] {
                        TypeQualifierSpec::Storage(storage_qualifier) => match &storage_qualifier {
                            StorageQualifier::Uniform => self.uniforms.push((
                                name.clone(),
                                declaration.ty.ty.clone(),
                                declaration.array_specifier.clone(),
                            )),
                            _ => {}
                        },
                        _ => {}
                    }
                }
            }

            Visit::Parent
        }
    }

    let mut v = StorageLister { uniforms: vec![] };
    tu.visit(&mut v);

    let StorageLister { uniforms } = v;
    let uniforms = uniforms
        .iter()
        .map(|(id, type_spec, array_spec)| {
            use quote::quote;
            let (mut type_str, mut init_str) = match &type_spec.ty {
                TypeSpecifierNonArray::Int => (quote!(i32), quote!(0)),
                TypeSpecifierNonArray::Float => (quote!(f32), quote!(0.)),
                TypeSpecifierNonArray::Vec2 => (quote!([f32; 2]), quote!([0., 2])),
                TypeSpecifierNonArray::Vec3 => (quote!([f32; 3]), quote!([0., 3])),
                TypeSpecifierNonArray::Vec4 => (quote!([f32; 4]), quote!([0., 4])),
                TypeSpecifierNonArray::Mat2 => (quote!([f32; 4]), quote!([0., 4])),
                TypeSpecifierNonArray::Mat3 => (quote!([f32; 9]), quote!([0., 9])),
                TypeSpecifierNonArray::Mat4 => (quote!([f32; 15]), quote!([0., 15])),
                _ => panic!("unsupported uniform type: {}", id.0),
            };
            if let Some(array_spec) = array_spec {
                match array_spec {
                    ArraySpecifier::Unsized => {}
                    ArraySpecifier::ExplicitlySized(size) => match **size {
                        Expr::IntConst(size) => {
                            let size = size as usize;
                            type_str = quote!([#type_str; #size]);
                            init_str = quote!([#init_str; #size]);
                        }
                        _ => {}
                    },
                }
            }
            let id = quote::format_ident!("{}", id.0);
            (id, type_str, init_str)
        })
        .collect::<Vec<_>>();

    let properties = uniforms
        .iter()
        .map(|(id, type_str, _)| quote::quote!(#id: #type_str));

    let initializers = uniforms
        .iter()
        .map(|(id, _, init_str)| quote::quote!(#id: #init_str));

    let result = quote::quote! {
        struct ShaderTest {
            #(#properties), *
        }
    };

    result.into()
}
