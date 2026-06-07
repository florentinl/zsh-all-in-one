use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::Ident;
use proc_macro2::Span;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::ImplItemFn;
use syn::Type;
use syn::TypePath;

use crate::BUILTIN_METHOD_MARKER;
use crate::FUNCTION_METHOD_MARKER;
use crate::ModuleMethodType;
use crate::WIDGET_METHOD_MARKER;

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

pub fn find_fn_type(method: &mut ImplItemFn) -> Result<Option<ModuleMethodType>, String> {
    let mut index = None;
    let mut typ = None;
    for (i, attr) in method.attrs.iter().enumerate() {
        if attr.path().is_ident(BUILTIN_METHOD_MARKER) {
            index = Some(i);
            typ = Some(ModuleMethodType::Builtin);
        }

        if attr.path().is_ident(FUNCTION_METHOD_MARKER) {
            index = Some(i);
            typ = Some(ModuleMethodType::Function);
        }

        if attr.path().is_ident(WIDGET_METHOD_MARKER) {
            index = Some(i);
            typ = Some(ModuleMethodType::Widget);
        }
    }

    if let Some(index) = index {
        method.attrs.remove(index);
    }
    Ok(typ)
}
