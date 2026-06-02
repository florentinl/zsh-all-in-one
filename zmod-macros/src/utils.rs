use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::Ident;
use proc_macro2::Span;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::ImplItemFn;
use syn::Type;
use syn::TypePath;

use crate::ModuleMethodType;

pub fn crate_root_path_from_name(public_name: &str) -> TokenStream2 {
    match crate_name(public_name) {
        Ok(FoundCrate::Itself) => quote!(crate),
        Ok(FoundCrate::Name(found)) => {
            let ident = Ident::new(&found, Span::call_site());
            quote!(::#ident)
        }
        Err(_) => {
            let ident = Ident::new(public_name, Span::call_site());
            quote!(::#ident)
        }
    }
}

pub fn path_ident(ty: &Type) -> Option<Ident> {
    if let Type::Path(TypePath { qself: None, path }) = ty {
        path.segments.last().map(|segment| segment.ident.clone())
    } else {
        None
    }
}

pub fn find_fn_type(method: &ImplItemFn) -> Result<Option<ModuleMethodType>, String> {
    let last_seg: Vec<String> = method
        .attrs
        .iter()
        .filter_map(|attr| attr.path().segments.last().map(|seg| seg.ident.to_string()))
        .collect();

    if last_seg.iter().any(|seg| seg == "builtin") {
        return Ok(Some(ModuleMethodType::Builtin));
    }

    Ok(None)
}
