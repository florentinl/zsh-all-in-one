use std::ffi::CString;

use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::quote;
use syn::{ImplItem, ImplItemFn, parse_quote};

use crate::ModuleImplBuilder;

impl ModuleImplBuilder {
    pub(crate) fn process_function(&mut self, method: &mut ImplItemFn) {
        let fname = method.sig.ident.clone();

        self.add_function(fname);
    }

    pub(crate) fn add_function(&mut self, fname: Ident) {
        let fname_lit = Literal::c_string(&CString::new(fname.to_string()).unwrap());
        self.function_names.push((fname.clone(), fname_lit.clone()));
        let trampoline = Ident::new(
            &format!("__zmod_function_builtin_{}_{}", self.self_ty, fname),
            Span::call_site(),
        );
        let bn_name = Literal::c_string(&CString::new(trampoline.to_string()).unwrap());

        self.add_builtin(&bn_name, &fname, &trampoline);

        let registration_script = Literal::c_string(
            &CString::new(format!("{fname}() {{ builtin {trampoline} $@ }}")).unwrap(),
        );

        self.setup_parts.push(quote! {
            zsh.exec(#registration_script);
        });
    }

    pub fn functions_struct(&self) -> (TokenStream, ImplItem) {
        let function_struct_name =
            Ident::new(&format!("{}Functions", self.self_ty), Span::call_site());

        let fields = self
            .function_names
            .iter()
            .map(|(ident, _)| quote! { #ident: &'static CStr });

        let inits = self
            .function_names
            .iter()
            .map(|(ident, lit)| quote! { #ident: #lit });

        (
            quote! {
                struct #function_struct_name {
                    #( #fields, )*
                }
            },
            parse_quote! {
                const FUNCTIONS: #function_struct_name = #function_struct_name {
                    #( #inits, )*
                };
            },
        )
    }
}
