use std::ffi::CString;

use proc_macro2::Literal;
use quote::quote;
use syn::ImplItemFn;

use crate::ModuleImplBuilder;

use proc_macro2::Ident as Ident2;
use proc_macro2::Span as Span2;

impl ModuleImplBuilder {
    pub(crate) fn process_function(&mut self, method: &mut ImplItemFn) {
        let fname = method.sig.ident.clone();

        self.add_function(fname);
    }

    pub(crate) fn add_function(&mut self, fname: Ident2) {
        let trampoline = Ident2::new(
            &format!("__zmod_function_builtin_{}_{}", self.self_ty, fname),
            Span2::call_site(),
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
}
