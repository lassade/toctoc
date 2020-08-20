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
    let input_generics = bound::within_lifetime_bound(&input.generics, "'de"); // Add deserialzier lifetime
    let (impl_de_generics, _, _) = input_generics.split_for_impl();

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let fieldname = fields.named.iter().map(|f| &f.ident).collect::<Vec<_>>();
    let fieldty = fields.named.iter().map(|f| &f.ty);
    let fieldstr = fields
        .named
        .iter()
        .map(attr::name_of_field)
        .collect::<Result<Vec<_>>>()?;

    let bound = parse_quote!(knocknoc::Deserialize);
    let bounded_where_clause = bound::where_clause_with_bound(&input.generics, bound);

    Ok(quote! {
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            use knocknoc as __crate;

            #[repr(C)]
            struct __Visitor #impl_generics #where_clause {
                __out: __crate::export::Option<#ident #ty_generics>,
            }

            impl #impl_de_generics __crate::Deserialize<'de> for #ident #ty_generics #bounded_where_clause {
                fn begin(__out: &mut __crate::export::Option<Self>) -> &mut dyn __crate::de::Visitor<'de> {
                    unsafe {
                        &mut *{
                            __out
                            as *mut __crate::export::Option<Self>
                            as *mut __Visitor #ty_generics
                        }
                    }
                }
            }

            impl #impl_de_generics __crate::de::Visitor<'de> for __Visitor #ty_generics #bounded_where_clause {
                fn map(&mut self, __m: &mut dyn __crate::de::Map<'de>, __c: &mut dyn __crate::de::Context) -> __crate::Result<()> {
                    #(
                        let mut #fieldname: __crate::export::Option<#fieldty> = __crate::Deserialize::default();
                    )*

                    while let Some(__k) = __m.next()? {
                        match __k {
                            #(
                                #fieldstr => __m.visit(__crate::Deserialize::begin(&mut #fieldname), __c)?,
                            )*
                            _ => __m.visit(__crate::de::Visitor::ignore(), __c)?,
                        }
                    }
                    #(
                        let #fieldname = #fieldname.take().ok_or(__crate::Error)?;
                    )*
                    self.__out = __crate::export::Some(#ident {
                        #(
                            #fieldname,
                        )*
                    });
                    Ok(())
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

            impl<'de> knocknoc::Deserialize<'de> for #ident {
                fn begin(__out: &mut knocknoc::export::Option<Self>) -> &mut dyn knocknoc::de::Visitor<'de> {
                    unsafe {
                        &mut *{
                            __out
                            as *mut knocknoc::export::Option<Self>
                            as *mut __Visitor
                        }
                    }
                }
            }

            impl<'de> knocknoc::de::Visitor<'de> for __Visitor {
                fn string(&mut self, s: &'de knocknoc::export::str, context: &mut dyn knocknoc::de::Context) -> knocknoc::Result<()> {
                    let value = match s {
                        #( #names => #ident::#var_idents, )*
                        _ => { knocknoc::export::Err(knocknoc::Error)? },
                    };
                    self.__out = knocknoc::export::Some(value);
                    knocknoc::export::Ok(())
                }
            }
        };
    })
}
