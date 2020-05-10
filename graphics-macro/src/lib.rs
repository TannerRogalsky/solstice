extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{self, parse_macro_input, Data, DeriveInput, Field, Fields, Meta, NestedMeta};

fn has_attr(field: &Field, i: &str) -> bool {
    field.attrs.iter().any(|attr| {
        if let Some(attr_ident) = attr.path.get_ident() {
            attr_ident == i
        } else {
            false
        }
    })
}

#[proc_macro_derive(Shader, attributes(uniform))]
pub fn derive_shader(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let ident = input.ident;

    let fields = match input.data {
        Data::Struct(s) => match s.fields {
            Fields::Named(fields) => fields
                .named
                .iter()
                .filter(|field| has_attr(field, "uniform"))
                .map(|field| {
                    let field_ident = field.ident.as_ref().unwrap();
                    let field_ty = &field.ty;

                    quote! {
                        impl ::graphics::shader::UniformGetter<#field_ty> for #ident {
                            fn get_uniform(&self) -> &#field_ty {
                                &self.#field_ident
                            }
                        }

                        impl ::graphics::shader::UniformGetterMut<#field_ty> for #ident {
                            fn get_uniform_mut(&mut self) -> &mut #field_ty {
                                &mut self.#field_ident
                            }
                        }
                    }
                })
                .collect::<Vec<_>>(),
            _ => panic!("only named fields are supported"),
        },
        _ => panic!("only structs are supported"),
    };

    TokenStream::from(quote! {
        #(#fields)*
        impl ::graphics::shader::BasicUniformSetter for #ident {}
    })
}

#[proc_macro_derive(Uniform, attributes(location))]
pub fn derive_uniform(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let ident = input.ident;

    let fields = match input.data {
        Data::Struct(s) => match s.fields {
            Fields::Named(fields) => fields
                .named
                .iter()
                .filter(|field| has_attr(field, "location"))
                .map(|field| {
                    let field_ident = field.ident.as_ref().unwrap();
                    quote! {
                        impl ::graphics::shader::UniformTrait for #ident {
                            type Value = [f32; 16];
                            const NAME: &'static str = "#field_ident";

                            fn get_location(&self) -> Option<&::graphics::shader::UniformLocation> {
                                self.#field_ident.as_ref()
                            }
                        }
                    }
                })
                .collect::<Vec<_>>(),
            _ => panic!("only named fields are supported"),
        },
        _ => panic!("only structs are supported"),
    };

    TokenStream::from(quote! {
        #(#fields)*
    })
}

#[proc_macro_derive(Vertex)]
pub fn derive_vertex(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    assert!(
        input.attrs.iter().any(|attr| {
            if let Some(ident) = attr.path.get_ident() {
                if ident == "repr" {
                    if let Ok(syn::Meta::List(ref meta_list)) = attr.parse_meta() {
                        let reprs = meta_list
                            .nested
                            .iter()
                            .filter_map(|nested| match nested {
                                NestedMeta::Meta(Meta::Path(path)) => path.get_ident(),
                                _ => None,
                            })
                            .collect::<Vec<_>>();
                        return ["packed", "C"]
                            .iter()
                            .all(|repr| reprs.iter().find(|&r| r == repr).is_some());
                    }
                }
            }
            false
        }),
        "Vertex structs must be `#[repr(C, packed)]`"
    );

    match input.data {
        Data::Struct(s) => match s.fields {
            Fields::Named(fields) => {
                let field_types = fields.named.iter().scan(vec![], |state, field| {
                    state.push(&field.ty);
                    Some(state.clone())
                });

                let vertex_formats = field_types.zip(fields.named.iter()).map(|(mut types, ident)| {
                        let this_type = types.pop().unwrap();
                        let name = format!("{:}", ident.ident.as_ref().unwrap());
                        let offset = if types.is_empty() {
                            quote! {
                                0usize
                            }
                        } else {
                            quote! {
                                #(::std::mem::size_of::<#types>())+*
                            }
                        };

                        quote! {
                            ::graphics::vertex::VertexFormat {
                                name: #name,
                                offset: #offset,
                                atype: <#this_type as ::graphics::vertex::VertexAttributeType>::A_TYPE,
                                normalize: false,
                            }
                        }
                    });
                let ident = format_ident!("{}", input.ident);
                TokenStream::from(quote! {
                    impl Vertex for #ident {
                        fn build_bindings() -> &'static [::graphics::vertex::VertexFormat] {
                            &[
                                #(#vertex_formats),*
                            ]
                        }
                    }
                })
            }
            Fields::Unnamed(_) | Fields::Unit => panic!("only named fields are supported"),
        },
        Data::Enum(_) | Data::Union(_) => panic!("only structs are supported"),
    }
}
