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

    pub fn set_param_string(&self, param_name: &CStr, value: &CStr) {
        unsafe {
            zsh_sys::setsparam(param_name.as_zsh_ptr(), value.metadup());
        }
    }

    pub fn set_param_array(&self, param_name: &CStr, values: &[&CStr]) {
        unsafe {
            zsh_sys::setaparam(param_name.as_zsh_ptr(), values.metadup());
        }
    }

    pub fn append_param_array(&self, param_name: &CStr, values: &[&CStr]) {
        unsafe {
            zsh_sys::assignaparam(
                param_name.as_zsh_ptr(),
                values.metadup(),
                zsh_sys::ASSPM_AUGMENT as i32,
            );
        }
    }

    pub fn exec(&self, script: &CStr) {
        unsafe {
            zsh_sys::execstring(script.as_ptr().cast_mut(), 1, 0, null_mut());
        }
    }
}

pub(crate) trait CStrUtils {
    fn metadup(&self) -> *mut c_char;
    fn as_zsh_ptr(&self) -> *mut c_char;
}

impl CStrUtils for &CStr {
    fn metadup(&self) -> *mut c_char {
        unsafe { zsh_sys::ztrdup_metafy(self.as_ptr().cast_mut()) }
    }

    fn as_zsh_ptr(&self) -> *mut c_char {
        self.as_ptr().cast_mut()
    }
}

pub(crate) trait CStrArrayUtils {
    fn metadup(&self) -> *mut *mut c_char;
}

impl CStrArrayUtils for &[&CStr] {
    fn metadup(&self) -> *mut *mut c_char {
        unsafe {
            let zvalues = zsh_sys::zshcalloc(self.len() + 1) as *mut *mut c_char;
            for (i, &val) in self.iter().enumerate() {
                (*zvalues.add(i)) = val.metadup();
            }
            zvalues
        }
    }
}
