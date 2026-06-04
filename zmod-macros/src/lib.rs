mod builtin;
mod function;
mod utils;

use std::vec;

use builtin::BintabRecord;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{ImplItem, ItemImpl, parse_macro_input};
use utils::{crate_root_path_from_name, find_fn_type, path_ident};

enum ModuleMethodType {
    Builtin,
    Function,
}

#[proc_macro_attribute]
pub fn module_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let imp: ItemImpl = parse_macro_input!(item as ItemImpl);
    ModuleImplBuilder::build(imp).into()
}

#[proc_macro_attribute]
pub fn builtin(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn function(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

struct ModuleImplBuilder {
    zmod: TokenStream2,
    self_ty: syn::Ident,

    bintab_records: Vec<BintabRecord>,
    setup_parts: Vec<TokenStream2>,
    trampolines: Vec<TokenStream2>,
}

impl ModuleImplBuilder {
    fn build(mut imp: ItemImpl) -> TokenStream2 {
        let zmod = crate_root_path_from_name("zmod");
        let self_ty = imp.self_ty.clone();
        let self_ty = path_ident(&self_ty).unwrap();

        let mut builder = ModuleImplBuilder {
            zmod,
            self_ty,
            bintab_records: vec![],
            setup_parts: vec![],
            trampolines: vec![],
        };

        for item in &mut imp.items {
            if let ImplItem::Fn(method) = item {
                let fn_type = find_fn_type(method).unwrap();
                match fn_type {
                    Some(ModuleMethodType::Builtin) => builder.process_builtin(method),
                    Some(ModuleMethodType::Function) => builder.process_function(method),
                    None => (),
                }
            }
        }

        let trampolines = &builder.trampolines;
        let setup_parts = &builder.setup_parts;
        let zmod = &builder.zmod;
        let self_ty = &builder.self_ty;
        let (bintab, bincount) = builder.make_bintab(zmod);

        quote! {
            #imp

            #bintab

            thread_local! {
                pub(crate) static STATE: std::cell::RefCell<#self_ty> =
                    std::cell::RefCell::new(#self_ty::new());
            }

            #(#trampolines)*

            #[allow(static_mut_refs)]
            static mut MODULE_FEATURES: #zmod::zsh_sys::features = #zmod::zsh_sys::features {
                bn_list: unsafe { BINTAB_PTR },
                bn_size: #bincount,
                cd_list: ::core::ptr::null_mut(),
                cd_size: 0,
                mf_list: ::core::ptr::null_mut(),
                mf_size: 0,
                pd_list: ::core::ptr::null_mut(),
                pd_size: 0,
                n_abstract: 0,
            };

            // The C ABI expected by zsh.
            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn boot_(_: #zmod::zsh_sys::Module) -> ::core::ffi::c_int {
                0
            }

            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn features_(
                m: #zmod::zsh_sys::Module,
                features: *mut *mut *mut ::core::ffi::c_char
            ) -> ::core::ffi::c_int {
                *features = #zmod::zsh_sys::featuresarray(m, &mut MODULE_FEATURES);
                0
            }

            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn enables_(
                m: #zmod::zsh_sys::Module,
                enables: *mut *mut ::core::ffi::c_int
            ) -> ::core::ffi::c_int {
                #zmod::zsh_sys::handlefeatures(m, &mut MODULE_FEATURES, enables)
            }

            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn setup_(_: #zmod::zsh_sys::Module) -> ::core::ffi::c_int {

                if let Err(e) = ::std::panic::catch_unwind(|| {
                    let zsh = #zmod::Zsh::new();

                    #(#setup_parts)*

                    STATE.with_borrow_mut(move |state| state.setup(zsh));
                }) {
                    println!("crashed in #self_ty: {:?}", e);
                }
                0
            }

            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn cleanup_(
                m: #zmod::zsh_sys::Module
            ) -> ::core::ffi::c_int {
                #zmod::zsh_sys::setfeatureenables(m, &mut MODULE_FEATURES, ::core::ptr::null_mut())
            }

            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn finish_(_: #zmod::zsh_sys::Module) -> ::core::ffi::c_int { 0 }
        }
    }
}
