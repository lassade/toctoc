use darling::{FromDeriveInput, FromField, FromVariant};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{parse_quote, Data, DataEnum, DataStruct, DeriveInput, Error, Fields, Type};

use crate::common::*;
use crate::DeriveResult;
use crate::{attr, bound};

pub fn derive(input: DeriveInput) -> DeriveResult<TokenStream> {
    match &input.data {
        Data::Struct(DataStruct { fields, .. }) => derive_struct(&input, &fields),
        Data::Enum(enumeration) => derive_enum(&input, enumeration),
        _ => Err(Error::new(Span::call_site(), "unions aren't supported").to_compile_error()),
    }
}

fn derive_struct(input: &DeriveInput, fields: &Fields) -> DeriveResult<TokenStream> {
    let body = match fields {
        Fields::Named(fields) => {
            let mut field_name = vec![];
            let mut field = vec![];

            for f in &fields.named {
                let opt = ToctocFieldOptions::from_field(f).map_err(|err| err.write_errors())?;

                if opt.skip || opt.no_ser {
                    continue;
                }

                let ident = opt.name().unwrap();
                field.push(ident.clone());
                field_name.push(ident.to_string());
            }

            quote! {
                v.map()
                #(.field(#field_name, &self.#field, c))*
                .done()
            }
        }
        Fields::Unnamed(fields) => {
            let (field_name, field): (Vec<_>, Vec<_>) = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(i, _)| (i.to_string(), make_literal_int(i)))
                .unzip();

            quote! {
                v.map()
                #(.field(#field_name, &self.#field, c))*
                .done()
            }
        }
        Fields::Unit => quote! { v.null() },
    };

    let derive_opt = ToctocOptions::from_derive_input(input).map_err(|err| err.write_errors())?;
    let crate_path = derive_opt.crate_path_or_default();

    let ident = &input.ident;
    let (impl_generics, ty_generics, _) = input.generics.split_for_impl();

    // TODO: Custom bounds

    let bound = parse_quote!(__crate::ser::Serialize);
    let where_clause = bound::where_clause_with_bound(&input.generics, bound);

    Ok(quote! {
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            use #crate_path as __crate;

            impl #impl_generics __crate::ser::Serialize for #ident #ty_generics #where_clause {
                fn begin(&self, v: __crate::ser::Visitor, c: &dyn __crate::ser::Context) -> __crate::ser::Done {
                    #body
                }
            }
        };
    })
}

fn derive_enum(input: &DeriveInput, enumeration: &DataEnum) -> DeriveResult<TokenStream> {
    let derive_opt = ToctocOptions::from_derive_input(input).map_err(|err| err.write_errors())?;
    let crate_path = derive_opt.crate_path_or_default();

    let ident = &input.ident;
    let (impl_generics, ty_generics, _) = input.generics.split_for_impl();

    // TODO: Custom bounds

    let bound = parse_quote!(__crate::ser::Serialize);
    let where_clause = bound::where_clause_with_bound(&input.generics, bound);

    let inner_generics = bound::with_lifetime_bound(&input.generics, "'a");
    let (inner_impl_generics, inner_ty_generics, inner_where_clause) =
        inner_generics.split_for_impl();

    let mut arm = vec![];
    for v in &enumeration.variants {
        let opt = ToctocVariantOptions::from_variant(v).map_err(|err| err.write_errors())?;

        if opt.skip || opt.no_ser {
            continue;
        }

        let variant = &opt.ident;
        let name = opt.name().unwrap().to_string();

        match &v.fields {
            Fields::Named(fields) => {
                let mut field_name = vec![];
                let mut field = vec![];
                let mut field_ty = vec![];
                let mut field_deref = vec![];

                for f in &fields.named {
                    let opt =
                        ToctocFieldOptions::from_field(f).map_err(|err| err.write_errors())?;

                    if opt.skip || opt.no_ser {
                        continue;
                    }

                    let ident = opt.name().unwrap();
                    field.push(ident.clone());
                    field_name.push(ident.to_string());

                    match &f.ty {
                        Type::Reference(r) => {
                            field_deref.push(Some(syn::token::Star::default()));
                            field_ty.push(r.elem.as_ref().clone());
                        }
                        ty => {
                            field_deref.push(None);
                            field_ty.push(ty.clone());
                        }
                    }
                }

                arm.push(quote! {
                    #ident::#variant { #(#field,)* } => {
                        struct __Inner #inner_impl_generics {
                            #( #field: &'a #field_ty, )*
                        }

                        impl #inner_impl_generics __crate::ser::Serialize for __Inner #inner_ty_generics #inner_where_clause {
                            fn begin(&self, v: __crate::ser::Visitor, c: &dyn __crate::ser::Context) -> __crate::ser::Done {
                                v.map()
                                #(.field(#field_name, &*self.#field, c))*
                                .done()
                            }
                        }

                        v.map()
                        .field(#name, &__Inner { #( #field: #field_deref #field, )* }, c)
                        .done()
                    }
                })
            }
            Fields::Unnamed(fields) => {
                // ? NOTE: Depends on the implementation for tuples

                let field: Vec<_> = fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(i, _)| make_ident(i))
                    .collect();

                arm.push(quote! {
                    #ident::#variant (#(#field,)*) => {
                        v.map()
                        .field(#name, &(#(#field,)*), c)
                        .done()
                    }
                })
            }
            Fields::Unit => arm.push(quote! {
                #ident::#variant => v.string(#name)
            }),
        }
    }

    Ok(quote! {
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            use #crate_path as __crate;

            impl #impl_generics __crate::ser::Serialize for #ident #ty_generics #where_clause {
                fn begin(&self, v: __crate::ser::Visitor, c: &dyn __crate::ser::Context) -> __crate::ser::Done {
                    match self {
                        #(#arm,)*
                    }
                }
            }
        };
    })
}
