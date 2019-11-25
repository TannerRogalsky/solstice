#![feature(proc_macro_span)]
#![feature(proc_macro_hygiene)]
#![feature(proc_macro_quote)]
extern crate proc_macro;

mod attrib;
mod semantics;
mod vertex;

use crate::semantics::generate_enum_semantics_impl;
use crate::vertex::generate_vertex_impl;
use glsl::syntax::{ArraySpecifier, TypeSpecifierNonArray};
use proc_macro::TokenStream;
use syn::{self, parse_macro_input, Data, DeriveInput};

/// The [`Vertex`] derive proc-macro.
///
/// That proc-macro allows you to create custom vertex types easily without having to care about
/// implementing the required traits for your types to be usable with the rest of the crate.
///
/// # The `Vertex` trait
///
/// The [`Vertex`] trait must be implemented if you want to use a type as vertex (passed-in via
/// slices to [`Tess`]). Either you can decide to implement it on your own, or you could just let
/// this crate do the job for you.
///
/// > Important: the [`Vertex`] trait is `unsafe`, which means that all of its implementors must be
/// > as well. This is due to the fact that vertex formats include information about raw-level
/// > GPU memory and a bad implementation can have undefined behaviors.
///
/// # Deriving `Vertex`
///
/// You can derive the [`Vertex`] trait if your type follows these conditions:
///
///   - It must be a `struct` with named fields. This is just a temporary limitation that will get
///     dropped as soon as the crate is stable enough.
///   - Its fields must have a type that implements [`VertexAttrib`]. This is mandatory so that the
///     backend knows enough about the types used in the structure to correctly align memory, pick
///     the right types, etc.
///   - Its fields must have a type that implements [`HasSemantics`] as well. This trait is just a
///     type family that associates a single constant (i.e. the semantics) that the vertex attribute
///     uses.
///
/// Once all those requirements are met, you can derive [`Vertex`] pretty easily.
///
/// > Note: feel free to look at the [`Semantics`] proc-macro as well, that provides a way
/// > to generate semantics types in order to completely both implement [`Semantics`] for an
/// > `enum` of your choice, but also generate *field* types you can use when defining your vertex
/// > type.
///
/// ## Syntax
///
/// The syntax is the following:
///
/// ```rust
/// # use luminance_derive::{Vertex, Semantics};
///
/// // visit the Semantics proc-macro documentation for further details
/// #[derive(Clone, Copy, Debug, PartialEq, Semantics)]
/// pub enum Semantics {
///   #[sem(name = "position", repr = "[f32; 3]", wrapper = "VertexPosition")]
///   Position,
///   #[sem(name = "color", repr = "[f32; 4]", wrapper = "VertexColor")]
///   Color
/// }
///
/// #[derive(Clone, Copy, Debug, PartialEq, Vertex)] // just add Vertex to the list of derived traits
/// #[vertex(sem = "Semantics")] // specify the semantics to use for this type
/// struct MyVertex {
///   position: VertexPosition,
///   color: VertexColor
/// }
/// ```
///
/// > Note: the `Semantics` enum must be public because of the implementation of [`HasSemantics`]
/// > trait.
///
/// Besides the `Semantics`-related code, this will:
///
///   - Create a type called `MyVertex`, a struct that will hold a single vertex.
///   - Implement `Vertex for MyVertex`.
///
/// The proc-macro also supports an optional `#[vertex(instanced = "<bool>")]` struct attribute.
/// This attribute allows you to specify whether the fields are to be instanced or not. For more
/// about that, have a look at [`VertexInstancing`].
///
/// [`Vertex`]: https://docs.rs/luminance/latest/luminance/vertex/trait.Vertex.html
#[proc_macro_derive(Vertex, attributes(vertex))]
pub fn derive_vertex(input: TokenStream) -> TokenStream {
    let di: DeriveInput = parse_macro_input!(input);

    match di.data {
        // for now, we only handle structs
        Data::Struct(struct_) => match generate_vertex_impl(di.ident, di.attrs.iter(), struct_) {
            Ok(impl_) => impl_,
            Err(e) => panic!("{}", e),
        },

        _ => panic!("only structs are currently supported for deriving Vertex"),
    }
}

/// The [`Semantics`] derive proc-macro.
///
/// [`Semantics`]: https://docs.rs/luminance/latest/luminance/vertex/trait.Semantics.html
#[proc_macro_derive(Semantics, attributes(sem))]
pub fn derive_semantics(input: TokenStream) -> TokenStream {
    let di: DeriveInput = parse_macro_input!(input);

    match di.data {
        // for now, we only handle enums
        Data::Enum(enum_) => match generate_enum_semantics_impl(di.ident, enum_) {
            Ok(impl_) => impl_,
            Err(e) => panic!("{}", e),
        },

        _ => panic!("only enums are currently supported for deriving VertexAttribSem"),
    }
}

#[proc_macro]
pub fn include_str(input: TokenStream) -> TokenStream {
    use std::str::FromStr;

    let call_site = proc_macro::Span::call_site().source_file();
    let mut path = call_site.path();
    path.pop();

    let input = syn::parse_macro_input!(input as syn::LitStr);
    path.push(std::path::PathBuf::from_str(input.value().as_str()).unwrap());

    let contents = std::fs::read_to_string(path.as_path()).unwrap();

    let lit = proc_macro::Literal::string(contents.as_str());
    proc_macro::TokenTree::Literal(lit).into()
}

#[proc_macro_attribute]
pub fn shader(attr: TokenStream, item: TokenStream) -> TokenStream {
    #[derive(Debug)]
    struct ShaderSource {
        vertex: String,
        fragment: String,
    }

    let call_site = proc_macro::Span::call_site().source_file();
    let mut path = call_site.path();
    path.pop();

    let t: syn::ExprTuple = syn::parse(attr.clone()).unwrap();
    let mut srcs = t
        .elems
        .iter()
        .map(|e| match e {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(lit),
                ..
            }) => {
                let mut path = path.clone();
                lit.value()
            }
            _ => panic!("oh god oh no"),
        })
        .collect::<Vec<_>>();

    //    let shader_stage = ShaderSource {
    //        vertex: srcs.pop().unwrap().value(),
    //        fragment: srcs.pop().unwrap().value()
    //    };
    //
    //    println!("attr: {:#?}", shader_stage);
    println!("item: \"{}\"", item.to_string());
    item
}

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
            let (mut type_str, mut init_str, uniform_type) = match &type_spec.ty {
                TypeSpecifierNonArray::Int => (quote!(i32), quote!(0), quote!(SignedInt)),
                TypeSpecifierNonArray::Float => (quote!(f32), quote!(0.), quote!(Float)),
                TypeSpecifierNonArray::Vec2 => (quote!([f32; 2]), quote!([0.; 2]), quote!(Vec2)),
                TypeSpecifierNonArray::Vec3 => (quote!([f32; 3]), quote!([0.; 3]), quote!(Vec3)),
                TypeSpecifierNonArray::Vec4 => (quote!([f32; 4]), quote!([0.; 4]), quote!(Vec4)),
                TypeSpecifierNonArray::Mat2 => (quote!([f32; 4]), quote!([0.; 4]), quote!(Mat2)),
                TypeSpecifierNonArray::Mat3 => (quote!([f32; 9]), quote!([0.; 9]), quote!(Mat3)),
                TypeSpecifierNonArray::Mat4 => (quote!([f32; 16]), quote!([0.; 16]), quote!(Mat4)),
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
            (id, type_str, init_str, uniform_type)
        })
        .collect::<Vec<_>>();

    let properties = uniforms
        .iter()
        .map(|(id, type_str, ..)| quote::quote!(#id: #type_str));

    let location_properties = uniforms.iter().map(|(id, ..)| {
        let prop = quote::format_ident!("{}_location", id);
        quote::quote!(#prop: graphics::shader::UniformLocation)
    });

    let initializers = uniforms
        .iter()
        .map(|(id, _, init_str, ..)| quote::quote!(#id: #init_str));

    let location_initializers = uniforms.iter().map(|(id, ..)| {
        let prop = quote::format_ident!("{}_location", id);
        quote::quote! {
            #prop: {
                let gl = gl.borrow();
                let shader = gl.get_shader(inner).unwrap();
            }
        }
    });

    let getters = uniforms.iter().map(|(id, type_str, ..)| {
        let func = quote::format_ident!("get_{}", id);
        quote::quote! {
            pub fn #func(&self) -> &#type_str {
                &self.#id
            }
        }
    });

    let setters = uniforms.iter().map(|(id, type_str, _, uniform_type)| {
        let func = quote::format_ident!("set_{}", id);
        let location = quote::format_ident!("{}_location", id);
        quote::quote! {
                    pub fn #func(&mut self, v: #type_str) {
                        if self.#id != v {
        //                    self.gl.borrow_mut().set_uniform_by_location(
        //                        self.#location,
        //                        &graphics::shader::RawUniformValue::#uniform_type(v),
        //                    );
                            let location = self.#location;
                            let uniform = &graphics::shader::RawUniformValue::#uniform_type(v);
                            self.#id = v;
                        }
                    }
                }
    });

    let result = quote::quote! {
            struct ShaderTest {
                #(#properties, #location_properties), *
            }

            impl ShaderTest {
    //            pub fn new(gl: std::rc::Rc<std::cell::RefCell<graphics::Context>>) -> Self {
                pub fn new() -> Self {
                    Self {
    //                    gl: gl,
                        #(#initializers), *
                    }
                }

                #(#getters #setters)*
            }
        };

    result.into()
}
