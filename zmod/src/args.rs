use std::{
    ffi::{CStr, c_char},
    marker::PhantomData,
};

pub struct Args<'a> {
    current: *mut *mut c_char,
    _p: PhantomData<&'a mut ()>,
}

impl<'a> Args<'a> {
    #[doc(hidden)]
    pub fn new(ptr: *mut *mut c_char) -> Self {
        Args {
            current: ptr,
            _p: Default::default(),
        }
    }
}

impl<'a> Iterator for Args<'a> {
    type Item = &'a CStr;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if (*self.current).is_null() {
                return None;
            }

            let value = CStr::from_ptr(*self.current);
            self.current = self.current.add(1);
            Some(value)
        }
    }
}
