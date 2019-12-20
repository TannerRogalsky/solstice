extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{self, parse_macro_input, Data, DeriveInput, Fields};

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
