use quote::quote;
use std::ffi::CString;

use proc_macro2::{Ident as Ident2, Literal, Span as Span2};
use syn::ImplItemFn;

use crate::ModuleImplBuilder;

impl ModuleImplBuilder {
    pub fn process_widget(&mut self, method: &mut ImplItemFn) {
        let fname = method.sig.ident.clone();

        let trampoline = Ident2::new(
            &format!("__zmod_builtin_{}_{}", self.self_ty, fname),
            Span2::call_site(),
        );

        let fname_lit = Literal::c_string(&CString::new(fname.to_string()).unwrap());

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
}
