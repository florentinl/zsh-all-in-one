use std::{
    ffi::{CStr, c_char},
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
}

trait MetaDup {
    fn metadup(&self) -> *mut c_char;
}

impl<'a> MetaDup for &'a CStr {
    fn metadup(&self) -> *mut c_char {
        unsafe {
            zsh_sys::metafy(
                self.as_ptr().cast_mut(),
                self.count_bytes() as i32,
                zsh_sys::META_DUP as i32,
            )
        }
    }
}
