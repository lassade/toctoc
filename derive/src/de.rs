use crate::{attr, bound};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    parse_quote, Data, DataEnum, DataStruct, DeriveInput, Error, Fields, FieldsNamed, Ident, Result,
};

pub fn derive(input: DeriveInput) -> Result<TokenStream> {
    match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => derive_struct(&input, fields),
        Data::Enum(enumeration) => derive_enum(&input, enumeration),
        _ => Err(Error::new(
            Span::call_site(),
            "currently only structs with named fields are supported",
        )),
    }
}

pub fn derive_struct(input: &DeriveInput, fields: &FieldsNamed) -> Result<TokenStream> {
    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let dummy = Ident::new(
        &format!("_IMPL_MINIDESERIALIZE_FOR_{}", ident),
        Span::call_site(),
    );

    let fieldname = fields.named.iter().map(|f| &f.ident).collect::<Vec<_>>();
    let fieldty = fields.named.iter().map(|f| &f.ty);
    let fieldstr = fields
        .named
        .iter()
        .map(attr::name_of_field)
        .collect::<Result<Vec<_>>>()?;

    let wrapper_generics = bound::with_lifetime_bound(&input.generics, "'__a");
    let (wrapper_impl_generics, wrapper_ty_generics, _) = wrapper_generics.split_for_impl();
    let bound = parse_quote!(knocknoc::Deserialize);
    let bounded_where_clause = bound::where_clause_with_bound(&input.generics, bound);

    Ok(quote! {
        #[allow(non_upper_case_globals)]
        const #dummy: () = {
            #[repr(C)]
            struct __Visitor #impl_generics #where_clause {
                __out: knocknoc::export::Option<#ident #ty_generics>,
            }

            impl #impl_generics knocknoc::Deserialize for #ident #ty_generics #bounded_where_clause {
                fn begin(__out: &mut knocknoc::export::Option<Self>) -> &mut dyn knocknoc::de::Visitor {
                    unsafe {
                        &mut *{
                            __out
                            as *mut knocknoc::export::Option<Self>
                            as *mut __Visitor #ty_generics
                        }
                    }
                }
            }

            impl #impl_generics knocknoc::de::Visitor for __Visitor #ty_generics #bounded_where_clause {
                fn map(&mut self, context: &mut Option<&mut dyn knocknoc::de::Context>) -> knocknoc::Result<knocknoc::export::Box<dyn knocknoc::de::Map + '_>> {
                    Ok(knocknoc::export::Box::new(__State {
                        #(
                            #fieldname: knocknoc::Deserialize::default(),
                        )*
                        __out: &mut self.__out,
                    }))
                }
            }

            struct __State #wrapper_impl_generics #where_clause {
                #(
                    #fieldname: knocknoc::export::Option<#fieldty>,
                )*
                __out: &'__a mut knocknoc::export::Option<#ident #ty_generics>,
            }

            impl #wrapper_impl_generics knocknoc::de::Map for __State #wrapper_ty_generics #bounded_where_clause {
                fn key(&mut self, __k: &knocknoc::export::str) -> knocknoc::Result<&mut dyn knocknoc::de::Visitor> {
                    match __k {
                        #(
                            #fieldstr => knocknoc::export::Ok(knocknoc::Deserialize::begin(&mut self.#fieldname)),
                        )*
                        _ => knocknoc::export::Ok(knocknoc::de::Visitor::ignore()),
                    }
                }

                fn finish(&mut self) -> knocknoc::Result<()> {
                    #(
                        let #fieldname = self.#fieldname.take().ok_or(knocknoc::Error)?;
                    )*
                    *self.__out = knocknoc::export::Some(#ident {
                        #(
                            #fieldname,
                        )*
                    });
                    knocknoc::export::Ok(())
                }
            }
        };
    })
}

pub fn derive_enum(input: &DeriveInput, enumeration: &DataEnum) -> Result<TokenStream> {
    if input.generics.lt_token.is_some() || input.generics.where_clause.is_some() {
        return Err(Error::new(
            Span::call_site(),
            "Enums with generics are not supported",
        ));
    }

    let ident = &input.ident;
    let dummy = Ident::new(
        &format!("_IMPL_MINIDESERIALIZE_FOR_{}", ident),
        Span::call_site(),
    );

    let var_idents = enumeration
        .variants
        .iter()
        .map(|variant| match variant.fields {
            Fields::Unit => Ok(&variant.ident),
            _ => Err(Error::new_spanned(
                variant,
                "Invalid variant: only simple enum variants without fields are supported",
            )),
        })
        .collect::<Result<Vec<_>>>()?;
    let names = enumeration
        .variants
        .iter()
        .map(attr::name_of_variant)
        .collect::<Result<Vec<_>>>()?;

    Ok(quote! {
        #[allow(non_upper_case_globals)]
        const #dummy: () = {
            #[repr(C)]
            struct __Visitor {
                __out: knocknoc::export::Option<#ident>,
            }

            impl knocknoc::Deserialize for #ident {
                fn begin(__out: &mut knocknoc::export::Option<Self>) -> &mut dyn knocknoc::de::Visitor {
                    unsafe {
                        &mut *{
                            __out
                            as *mut knocknoc::export::Option<Self>
                            as *mut __Visitor
                        }
                    }
                }
            }

            impl knocknoc::de::Visitor for __Visitor {
                fn string(&mut self, s: &knocknoc::export::str, context: &mut Option<&mut dyn knocknoc::de::Context>) -> knocknoc::Result<()> {
                    let value = match s {
                        #( #names => #ident::#var_idents, )*
                        _ => { return knocknoc::export::Err(knocknoc::Error) },
                    };
                    self.__out = knocknoc::export::Some(value);
                    knocknoc::export::Ok(())
                }
            }
        };
    })
}
