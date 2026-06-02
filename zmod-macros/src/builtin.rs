use std::ffi::CString;

use proc_macro2::{Ident as Ident2, Literal, Span as Span2, TokenStream as TokenStream2};
use quote::quote;
use syn::{ImplItemFn, LitInt};

use crate::ModuleImplBuilder;

pub(crate) struct BintabRecord {
    nam: Literal,
    handlerfunc: Ident2,
    minargs: LitInt,
    maxargs: LitInt,
}

impl BintabRecord {
    pub fn to_tokenstream(&self, zmod: &TokenStream2) -> TokenStream2 {
        let BintabRecord {
            nam,
            handlerfunc,
            minargs,
            maxargs,
        } = &self;
        quote! {
            #zmod::zsh_sys::builtin {
                node: #zmod::zsh_sys::hashnode {
                    next: ::core::ptr::null_mut(),
                    nam: #nam.as_ptr().cast_mut(),
                    flags: 0,
                },
                handlerfunc: Some(#handlerfunc),
                minargs: #minargs,
                maxargs: #maxargs,
                funcid: 0,
                optstr: ::core::ptr::null_mut(),
                defopts: ::core::ptr::null_mut(),
            }
        }
    }
}

impl ModuleImplBuilder {
    pub fn process_builtin(&mut self, method: &mut ImplItemFn) {
        let fname = method.sig.ident.clone();

        let trampoline = Ident2::new(
            &format!("__zmod_builtin_{}_{}", self.self_ty, fname),
            Span2::call_site(),
        );
        let fname_lit = Literal::c_string(&CString::new(fname.to_string()).unwrap());
        let zmod = &self.zmod;

        self.trampolines.push(quote! {
            pub unsafe extern "C" fn #trampoline(
                _name: *mut ::core::ffi::c_char,
                argv: *mut *mut ::core::ffi::c_char,
                _options: #zmod::zsh_sys::Options,
                _func: ::core::ffi::c_int
            ) -> ::core::ffi::c_int {
                let res = ::std::panic::catch_unwind(|| {
                    let args = #zmod::builtin::BuiltinArgs::new(argv);
                    let zsh = #zmod::Zsh::new();
                    STATE.with_borrow_mut(move |state| {
                        match state.#fname(zsh, args) {
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

        let minargs = LitInt::new("0", Span2::call_site());
        let maxargs = LitInt::new("-1", Span2::call_site());

        self.bintab_records.push(BintabRecord {
            nam: fname_lit,
            handlerfunc: trampoline,
            minargs,
            maxargs,
        })
    }

    pub fn make_bintab(&self, zmod: &TokenStream2) -> (TokenStream2, LitInt) {
        let builtin_count = self.bintab_records.len();

        let builtins = self.bintab_records.iter().map(|r| r.to_tokenstream(zmod));

        if !self.bintab_records.is_empty() {
            (
                quote! {
                    static mut BINTAB: [#zmod::zsh_sys::builtin; #builtin_count] = [
                        #(#builtins)*
                    ];

                    #[allow(static_mut_refs)]
                    static mut BINTAB_PTR: *mut #zmod::zsh_sys::builtin = unsafe {BINTAB.as_mut_ptr()};
                },
                LitInt::new(&builtin_count.to_string(), Span2::call_site()),
            )
        } else {
            (
                quote! {
                    #[allow(static_mut_refs)]
                    static mut BINTAB_PTR: *mut  #zmod::zsh_sys::builtin = unsafe {::core::ptr::null_mut()};
                },
                LitInt::new("0", Span2::call_site()),
            )
        }
    }
}
