use std::ffi::CStr;

use crate::zsh::CStrUtils as _;

use crate::error::ZshErr;

type WidgetFunc =
    unsafe extern "C" fn(arg1: *mut *mut ::std::os::raw::c_char) -> ::std::os::raw::c_int;

pub fn add_widget(name: &CStr, trampoline: WidgetFunc) -> Result<(), ZshErr> {
    let ptr = unsafe { zsh_sys::addzlefunction(name.as_zsh_ptr(), Some(trampoline), 0) };

    if ptr.is_null() {
        return Err(ZshErr {
            code: 1,
            message: format!("Name clash when adding zle widget {:?}", name),
        });
    }

    Ok(())
}
