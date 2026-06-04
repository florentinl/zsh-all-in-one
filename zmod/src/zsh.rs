use std::{
    ffi::{CStr, c_char},
    ptr::null_mut,
    rc::Rc,
};

/// Zero-sized capability proving "you are on the zsh thread right now".
pub struct Zsh<'z> {
    _no_send_sync: Rc<()>,
    _scope: std::marker::PhantomData<&'z mut ()>,
}

impl<'z> Default for Zsh<'z> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'z> Zsh<'z> {
    #[doc(hidden)]
    pub fn new() -> Self {
        Self {
            _no_send_sync: Rc::new(()),
            _scope: Default::default(),
        }
    }

    pub fn set_param_string<'a, 'b>(&self, param_name: &'a CStr, value: &'b CStr) {
        unsafe {
            zsh_sys::setsparam(param_name.as_ptr().cast_mut(), value.metadup());
        }
    }

    pub fn set_param_array<'a, 'b>(&self, param_name: &'a CStr, values: &[&'b CStr]) {
        unsafe {
            let zvalues = zsh_sys::zshcalloc(values.len() + 1) as *mut *mut c_char;
            for (i, &val) in values.iter().enumerate() {
                (*zvalues.add(i)) = val.metadup();
            }

            zsh_sys::setaparam(param_name.as_ptr().cast_mut(), zvalues);
        }
    }

    pub fn exec<'a>(&self, script: &'a CStr) {
        unsafe {
            zsh_sys::execstring(script.as_ptr().cast_mut(), 1, 0, null_mut());
        }
    }
}

trait MetaDup {
    fn metadup(&self) -> *mut c_char;
}

impl<'a> MetaDup for &'a CStr {
    fn metadup(&self) -> *mut c_char {
        unsafe { zsh_sys::ztrdup_metafy(self.as_ptr().cast_mut()) }
    }
}
