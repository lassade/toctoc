use darling::{FromDeriveInput, FromField, FromVariant};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{parse_quote, Data, DataEnum, DataStruct, DeriveInput, Error, Fields};

use crate::bound;
use crate::common::*;
use crate::DeriveResult;

pub fn derive(input: DeriveInput) -> DeriveResult<TokenStream> {
    match &input.data {
        Data::Struct(DataStruct { fields, .. }) => derive_struct(&input, fields),
        Data::Enum(enumeration) => derive_enum(&input, enumeration),
        _ => Err(Error::new(Span::call_site(), "unions aren't supported").to_compile_error()),
    }
}

fn derive_struct(input: &DeriveInput, fields: &Fields) -> DeriveResult<TokenStream> {
    let ident = &input.ident;

    let body = match fields {
        Fields::Named(fields) => {
            let mut field = vec![];
            let mut field_ty = vec![];
            let mut field_name = vec![];
            let mut field_unwrap = vec![];
            let mut skipped = vec![];
            let mut skipped_default = vec![];

            for f in &fields.named {
                let opt = ToctocFieldOptions::from_field(f).map_err(|err| err.write_errors())?;

                if opt.skip || opt.skip_deserializing {
                    let ident = &opt.ident;
                    skipped.push(ident.clone());
                    skipped_default.push(opt.default_behavior_forced());
                    continue;
                }

                let ident = opt.name().unwrap();
                let name = ident.to_string();

                // Create the default function if any otherwise result in error
                match opt.default_behavior() {
                    Some(default) => field_unwrap.push(quote! { unwrap_or_else(#default)? }),
                    None => {
                        field_unwrap.push(quote! { ok_or(__crate::Error::missing_field(#name))? })
                    }
                }

                field.push(ident.clone());
                field_ty.push(f.ty.clone());
                field_name.push(name);
            }

            quote! {
                fn map(&mut self, __m: &mut dyn __crate::de::Map<'de>, __c: &mut dyn __crate::de::Context) -> __crate::Result<()> {
                    #(let mut #field: __crate::export::Option<#field_ty> = __crate::Deserialize::default();)*
                    while let Some(__k) = __m.next()? {
                        match __k {
                            #(#field_name => __m.visit(__crate::Deserialize::begin(&mut #field), __c)?,)*
                            _ => __m.visit(__crate::de::Visitor::ignore(), __c)?,
                        }
                    }
                    // Unwrap all
                    #(let #field = #field.take() . #field_unwrap;)*
                    // Build struct
                    self.__out = __crate::export::Some(#ident {
                        #(#field,)*
                        #(#skipped: #skipped_default(),)* // Fill out skipped fields with their default values
                    });
                    Ok(())
                }
            }
        }
        Fields::Unnamed(fields) => {
            let (field, ty): (Vec<_>, Vec<_>) = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(i, f)| (f.ty.clone(), make_literal_int(i)))
                .unzip();

            quote! {
                fn seq(&mut self, __s: &mut dyn __crate::de::Seq<'de>, __c: &mut dyn __crate::de::Context) -> __crate::Result<()> {
                    self.__out = Some((
                        #({
                            let mut value: Option<#ty> = None;
                            __s.visit(Deserialize::begin(&mut value), __c)?;
                            value.ok_or(Error::missing_element(#field))?
                        },)*
                    ));
                    while __s.visit(Visitor::ignore(), __c)? {}
                    Ok(())
                }
            }
        }
        Fields::Unit => quote! {
            fn null(&mut self, _: &mut dyn __crate::de::Context) -> Result<()> {
                self.out = Some(#ident);
                Ok(())
            }
        },
    };

    let derive_opt = ToctocOptions::from_derive_input(input).map_err(|err| err.write_errors())?;
    let crate_path = derive_opt.crate_path_or_default();

    // TODO: Custom bounds

    let bound = parse_quote!(__crate::Deserialize);
    let where_clause = bound::where_clause_with_bound(&input.generics, bound);

    let ident = &input.ident;
    let input_generics = bound::within_lifetime_bound(&input.generics, "'de"); // Add deserialzier lifetime
    let (impl_de_generics, _, _) = input_generics.split_for_impl();

    let (impl_generics, ty_generics, _) = input.generics.split_for_impl();

    Ok(quote! {
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            use #crate_path as __crate;

            #[repr(C)]
            struct __Visitor #impl_generics #where_clause {
                __out: __crate::export::Option<#ident #ty_generics>,
            }

            impl #impl_de_generics __crate::de::Visitor<'de> for __Visitor #ty_generics #where_clause {
                #body
            }

            impl #impl_de_generics __crate::Deserialize<'de> for #ident #ty_generics #where_clause {
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
        };
    })
}

pub fn derive_enum(input: &DeriveInput, enumeration: &DataEnum) -> DeriveResult<TokenStream> {
    let derive_opt = ToctocOptions::from_derive_input(input).map_err(|err| err.write_errors())?;
    let crate_path = derive_opt.crate_path_or_default();

    // TODO: Custom bounds

    let bound = parse_quote!(__crate::Deserialize);
    let where_clause = bound::where_clause_with_bound(&input.generics, bound);

    let ident = &input.ident;
    let input_generics = bound::within_lifetime_bound(&input.generics, "'de"); // Add deserialzier lifetime
    let (impl_de_generics, _, _) = input_generics.split_for_impl();

    let (impl_generics, ty_generics, _) = input.generics.split_for_impl();

    let mut arm = vec![];
    let mut unit_variant = vec![];
    let mut unit_variant_name = vec![];

    for v in &enumeration.variants {
        let opt = ToctocVariantOptions::from_variant(v).map_err(|err| err.write_errors())?;

        if opt.skip || opt.no_de {
            continue;
        }

        let variant = &opt.ident;
        let name = opt.name().to_string();

        match &v.fields {
            Fields::Named(fields) => {
                let mut field = vec![];
                let mut field_ty = vec![];
                let mut field_name = vec![];
                let mut field_unwrap = vec![];
                let mut skipped = vec![];
                let mut skipped_default = vec![];

                for f in &fields.named {
                    let opt =
                        ToctocFieldOptions::from_field(f).map_err(|err| err.write_errors())?;

                    if opt.skip || opt.skip_deserializing {
                        // Field default value
                        let ident = &opt.ident;
                        skipped.push(ident.clone());
                        skipped_default.push(opt.default_behavior_forced());
                        continue;
                    }

                    let ident = &opt.ident;
                    let name = opt.name().unwrap().to_string();

                    // Create the default function if any otherwise result in error
                    match opt.default_behavior() {
                        Some(default) => field_unwrap.push(quote! { unwrap_or_else(#default)? }),
                        None => field_unwrap
                            .push(quote! { ok_or(__crate::Error::missing_field(#name))? }),
                    }

                    field.push(ident.clone());
                    field_ty.push(f.ty.clone());
                    field_name.push(name);
                }

                arm.push(quote! {
                    Some(#name) => {
                        struct __Inner #impl_generics #where_clause {
                            #( #field: #field_ty, )*
                        }

                        impl #impl_de_generics __crate::Deserialize<'de> for __Inner #ty_generics #where_clause {
                            fn begin(__out: &mut __crate::export::Option<Self>) -> &mut dyn __crate::de::Visitor<'de> {
                                unsafe {
                                    &mut *{
                                        __out
                                        as *mut __crate::export::Option<Self>
                                        as *mut __InnerVisitor #ty_generics
                                    }
                                }
                            }
                        }

                        #[repr(C)]
                        struct __InnerVisitor #impl_generics #where_clause {
                            __out: __crate::export::Option<__Inner #ty_generics>,
                        }

                        impl #impl_de_generics __crate::de::Visitor<'de> for __InnerVisitor #ty_generics #where_clause {
                            fn map(&mut self, __m: &mut dyn __crate::de::Map<'de>, __c: &mut dyn __crate::de::Context) -> __crate::Result<()> {
                                #(let mut #field: __crate::export::Option<#field_ty> = __crate::Deserialize::default();)*
                                while let Some(__k) = __m.next()? {
                                    match __k {
                                        #(#field_name => __m.visit(__crate::Deserialize::begin(&mut #field), __c)?,)*
                                        _ => __m.visit(__crate::de::Visitor::ignore(), __c)?,
                                    }
                                }
                                // Unwrap all
                                #(let #field = #field.take() . #field_unwrap;)*
                                // Build struct
                                self.__out = __crate::export::Some(__Inner { #(#field,)* });
                                Ok(())
                            }
                        }
                    
                        let mut __value = None;
                        __m.visit(__Inner::begin(&mut __value), __c)?;
                        let __value = __value.unwrap();
                        self.__out = __crate::export::Some(#ident::#variant {
                            #(#field: __value.#field,)*
                            #(#skipped: #skipped_default(),)* // Fill out skipped fields with their default values
                        })
                    }
                })
            }
            Fields::Unnamed(fields) => {
                let (ty, index): (Vec<_>, Vec<_>) = fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(i, f)| (f.ty.clone(), make_literal_int(i)))
                    .unzip();

                arm.push(quote! {
                    Some(#name) => {
                        let mut __value: Option<( #(#ty,)* )> = None;
                        __m.visit(__crate::de::Deserialize::begin(&mut __value), __c)?;
                        let __value = __value.unwrap();
                        self.__out = Some(#ident::#variant( #(__value.#index,)*));
                    }
                });
            }
            Fields::Unit => {
                unit_variant.push(variant.clone());
                unit_variant_name.push(name);
            }
        }
    }

    // Only create a map visitor if the enum hahs struct and tuple variants
    let map = if arm.len() > 0 {
        Some(quote!{
            fn map(&mut self, __m: &mut dyn __crate::de::Map<'de>, __c: &mut dyn __crate::de::Context) -> __crate::Result<()> {
                match __m.next()? {
                    #( #arm, )*
                    //Some(_) => __m.visit(__crate::de::Visitor::ignore(), __c)?,
                    Some(__variant) => { __crate::export::Err(__crate::Error::unknown_variant(__variant))? },
                    None => { __crate::export::Err(__crate::Error::expecting("variant"))? },
                }

                while let Some(s) = __m.next()? {
                    __m.visit(__crate::de::Visitor::ignore(), __c)?;
                }

                Ok(())
            }
        })
    } else {
        None
    };

    // Only create a string visitor if the enum has unit variants
    let string = if unit_variant.len() > 0 {
        Some(quote!{
            fn string(&mut self, s: &'de __crate::export::str, _: &mut dyn __crate::de::Context) -> __crate::Result<()> {
                let value = match s {
                    #( #unit_variant_name => #ident::#unit_variant, )*
                    __variant => { __crate::export::Err(__crate::Error::unknown_variant(__variant))? },
                };
                self.__out = __crate::export::Some(value);
                __crate::export::Ok(())
            }
        })
    } else {
        None
    };

    Ok(quote! {
        #[allow(non_upper_case_globals)]
        const _: () = {
            use #crate_path as __crate;

            #[repr(C)]
            struct __Visitor #impl_generics #where_clause {
                __out: __crate::export::Option<#ident #ty_generics>,
            }

            impl #impl_de_generics __crate::de::Visitor<'de> for __Visitor #ty_generics #where_clause {
                #map
                #string
            }

            impl #impl_de_generics __crate::Deserialize<'de> for #ident #ty_generics #where_clause {
                fn begin(__out: &mut __crate::export::Option<Self>) -> &mut dyn __crate::de::Visitor<'de> {
                    unsafe {
                        &mut *{
                            __out
                            as *mut __crate::export::Option<Self>
                            as *mut __Visitor
                        }
                    }
                }
            }
        };
    })
}
