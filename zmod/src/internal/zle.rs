use std::ffi::{CStr, CString};
use std::os::fd::RawFd;
use std::ptr;

use crate::zsh::CStrUtils as _;

use crate::error::ZshErr;

type WidgetFunc =
    unsafe extern "C" fn(arg1: *mut *mut ::std::os::raw::c_char) -> ::std::os::raw::c_int;

pub fn add_widget(name: &CStr, trampoline: WidgetFunc) -> Result<(), ZshErr> {
    let ptr = unsafe { zsh_sys::addzlefunction(name.dup(), Some(trampoline), 0) };

    if ptr.is_null() {
        return Err(ZshErr {
            code: 1,
            message: format!("Name clash when adding zle widget {:?}", name),
        });
    }

    Ok(())
}

pub(crate) fn bind_filedesc(fd: RawFd, widget_name: &CStr) {
    unsafe {
        let mut ops = zsh_sys::options {
            ind: [0; 128],
            args: ptr::null_mut(),
            argscount: 0,
            argsalloc: 0,
        };

        ops.ind['w' as usize] = 1;
        ops.ind['F' as usize] = 1;

        let fd = CString::new(fd.to_string()).unwrap();
        let fd_ptr = fd.as_c_str().as_zsh_ptr();

        let mut args = [fd_ptr, widget_name.as_zsh_ptr(), ptr::null_mut()];

        zsh_sys::bin_zle(c"zle".as_zsh_ptr(), args.as_mut_ptr(), &mut ops, 1);
    }
}
