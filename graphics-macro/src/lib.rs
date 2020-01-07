extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{self, parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(Shader, attributes(uniform))]
pub fn derive_shader(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let ident = input.ident;

    let fields = match input.data {
        Data::Struct(s) => match s.fields {
            Fields::Named(fields) => fields
                .named
                .iter()
                .filter(|field| {
                    if let Some(attr) = field.attrs.iter().next() {
                        if let Some(attr_ident) = attr.path.get_ident() {
                            return attr_ident == "uniform";
                        }
                    }
                    false
                })
                .map(|field| {
                    let field_ident = field.ident.as_ref().unwrap();
                    let field_ty = &field.ty;

                    quote! {
                        impl engine::graphics::shader::UniformGetterMut<#field_ty> for #ident {
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
        impl engine::graphics::shader::BasicUniformSetter for #ident {}
    })
}

#[proc_macro_derive(Vertex)]
pub fn derive_vertex(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

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
