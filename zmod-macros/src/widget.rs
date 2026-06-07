use quote::quote;
use std::ffi::CString;

use proc_macro2::{Ident, Literal, Span, TokenStream};
use syn::{ImplItem, ImplItemFn, parse_quote};

use crate::ModuleImplBuilder;

impl ModuleImplBuilder {
    pub fn process_widget(&mut self, method: &mut ImplItemFn) {
        let fname = method.sig.ident.clone();

        let trampoline = Ident::new(
            &format!("__zmod_builtin_{}_{}", self.self_ty, fname),
            Span::call_site(),
        );

        let fname_lit = Literal::c_string(&CString::new(fname.to_string()).unwrap());
        self.widget_names.push((fname.clone(), fname_lit.clone()));

        let zmod = &self.zmod;
        self.trampolines.push(quote! {
            pub unsafe extern "C" fn #trampoline(
                argv: *mut *mut ::core::ffi::c_char
            ) -> ::core::ffi::c_int {
                let res = ::std::panic::catch_unwind(|| {
                    let args = #zmod::args::Args::new(argv);
                    let zsh = #zmod::Zsh::new();
                    let zle = #zmod::Zle::new();
                    STATE.with_borrow_mut(move |state| {
                        match state.#fname(zsh, zle, args) {
                            Ok(()) => 0,
                            Err(err) => err.code,
                        }
                    })
                });

                match res {
                    Ok(code) => code,
                    Err(panic) => {
                        println!("paniced in {:?}: {:?}", #fname_lit, panic);
                        1
                    }
                }
            }
        });

        self.setup_parts.push(quote! {
            if let Err(err) = #zmod::internal::zle::add_widget(#fname_lit, #trampoline) {
                eprintln!("Failed to bind widget {:?}: {}", #fname_lit, err);
            }
        });
    }

    pub fn widgets_struct(&self) -> (TokenStream, ImplItem) {
        let widget_struct_name = Ident::new(&format!("{}Widgets", self.self_ty), Span::call_site());

        let fields = self
            .widget_names
            .iter()
            .map(|(ident, _)| quote! { #ident: &'static CStr });

        let inits = self
            .widget_names
            .iter()
            .map(|(ident, lit)| quote! { #ident: #lit });

        (
            quote! {
                struct #widget_struct_name {
                    #( #fields, )*
                }
            },
            parse_quote! {
                const WIDGETS: #widget_struct_name = #widget_struct_name {
                    #( #inits, )*
                };
            },
        )
    }
}
