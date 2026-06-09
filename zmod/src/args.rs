use std::{
    ffi::{CStr, c_char},
    fs::File,
    marker::PhantomData,
    os::fd::FromRawFd as _,
};

use crate::zsh::WidgetFdReader;

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

    pub fn as_fd_reader(mut self) -> Option<WidgetFdReader> {
        let first = self.next()?;
        let raw_fd = first.to_string_lossy().parse().ok()?;

        let file = unsafe { File::from_raw_fd(raw_fd) };
        Some(WidgetFdReader { file: Some(file) })
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
