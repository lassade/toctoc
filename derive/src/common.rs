//! Common stuff shared across all the derive implementation

use darling::util::Override;
use darling::{FromDeriveInput, FromField, FromVariant};
use proc_macro2::Span;

fn default_path() -> syn::Path {
    syn::parse_str("Default::default").unwrap()
}

pub fn make_ident<D: std::fmt::Display>(postfix: D) -> syn::Ident {
    let field = format!("_{}", postfix);
    syn::Ident::new(&field, proc_macro2::Span::call_site())
}

pub fn make_literal_int(i: usize) -> syn::LitInt {
    let i = format!("{}", i);
    syn::LitInt::new(&i, proc_macro2::Span::call_site())
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(toctoc))]
pub struct ToctocOptions {
    pub ident: syn::Ident,
    // /// Use the default to fill out any missing field from this `struct`.
    // ///
    // /// It's also possible to specify a function to create the default value like so:
    // /// `#[toctoc(default = "path::to::default_function")`
    // ///
    // /// **Note** Beware `enums` aren't suported because they can just swap variants while loading
    // /// thus invalidating all the defaults
    // #[darling(default)]
    // pub default: Option<Override<syn::Path>>,
    /// Specify a path to the toctoc crate instance to use when referring to toctoc APIs
    /// from generated code. This is normally only applicable when invoking re-exported
    /// toctoc derives from a public macro in a different crate.
    #[darling(default, rename = "crate")]
    pub crate_path: Option<syn::Path>,
}

impl ToctocOptions {
    pub fn crate_path_or_default(&self) -> syn::Path {
        self.crate_path
            .as_ref()
            .map_or_else(|| syn::parse_str("toctoc").unwrap(), |p| p.clone())
    }

    // /// Returns the default behavior
    // pub fn default_behaviour(&self) -> Option<syn::Path> {
    //     use Override::*;
    //     match &self.default {
    //         Some(Explicit(path)) => Some(path.clone()),
    //         Some(Inherit) => Some(default_path()),
    //         None => None,
    //     }
    // }
}

#[derive(Default, FromField)]
#[darling(default, attributes(toctoc))]
pub struct ToctocFieldOptions {
    pub ident: Option<syn::Ident>,
    /// Rename field ident
    pub rename: Option<syn::Ident>,
    /// Skips field (de)serialization. The field must implement `Default::default()`
    /// or specify any default function with `#[toctoc(default = "path::to::default_function")`
    pub skip: bool,
    /// Skip deserialization require a default value
    pub skip_deserializing: bool,
    /// Skip serialization
    pub skip_serializing: bool,
    /// Use the default implementation when this field is missing.
    ///
    /// It's also possible to specify a function to create the default value like so:
    /// `#[toctoc(default = "path::to::default_function")`
    pub default: Option<Override<syn::Path>>,
    // TODO: `bytes` allow (de)serialization using aligned bytes
    // TODO: allow custom save functions, but their signatures must be reviewed first
    // /// Especial `load` function to be used on this field.
    // ///
    // /// Uses this signature:
    // /// `fn load<L: Loader + ?Sized>(field: &mut FIELD_TYPE, loader: L) -> Result<(), L::Err>`
    // pub de_with: Option<syn::Path>,
    // /// Especial `save` function to be used on this field.
    // ///
    // /// Uses this signature:
    // /// `fn save<S: Saver + ?Sized>(field: &FIELD_TYPE, saver: S) -> Result<S::Ok, S::Err>`
    // pub ser_with: Option<syn::Path>,
}

impl ToctocFieldOptions {
    /// Field name
    pub fn name(&self) -> Option<&syn::Ident> {
        if self.rename.is_some() {
            self.rename.as_ref()
        } else {
            self.ident.as_ref()
        }
    }

    /// Returns the default behavior
    pub fn default_behavior(&self) -> Option<syn::Path> {
        use Override::*;
        match &self.default {
            Some(Explicit(path)) => Some(path.clone()),
            Some(Inherit) => Some(default_path()),
            None => None,
        }
    }

    /// Returns the default behavior forced on, used when the field is skipped
    pub fn default_behavior_forced(&self) -> syn::Path {
        use Override::*;
        match &self.default {
            Some(Explicit(path)) => path.clone(),
            Some(Inherit) | None => default_path(),
        }
    }
}

#[derive(FromVariant)]
#[darling(default, attributes(toctoc))]
pub struct ToctocVariantOptions {
    pub ident: syn::Ident,
    /// Rename field ident
    pub rename: Option<syn::Ident>,
    /// Skips field (de)serialization. The field must implement `Default::default()`
    /// or specify any default function with `#[toctoc(default = "path::to::default_function")`
    pub skip: bool,
    /// Skip deserialization require a default value
    pub no_de: bool,
    /// Skip serialization
    pub no_ser: bool,
}

impl Default for ToctocVariantOptions {
    fn default() -> Self {
        ToctocVariantOptions {
            ident: syn::Ident::new("Unknown", Span::call_site()),
            rename: None,
            skip: false,
            no_de: false,
            no_ser: false,
        }
    }
}

impl ToctocVariantOptions {
    pub fn name(&self) -> &syn::Ident {
        if let Some(name) = self.rename.as_ref() {
            name
        } else {
            &self.ident
        }
    }
}
