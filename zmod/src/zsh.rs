use std::{
    ffi::{CStr, CString, c_char},
    ptr::null_mut,
    rc::Rc,
};

pub enum ShellHook {
    ChPwd,
    PreCmd,
    PreExec,
    Periodic,
    ZshAddHistory,
    ZshExit,
    ZshDirectoryName,
}

impl ShellHook {
    const fn array_name(&self) -> &'static CStr {
        match self {
            ShellHook::ChPwd => c"chpwd_functions",
            ShellHook::PreCmd => c"precmd_functions",
            ShellHook::PreExec => c"preexec_functions",
            ShellHook::Periodic => c"periodic_functions",
            ShellHook::ZshAddHistory => c"zshaddhistory_functions",
            ShellHook::ZshExit => c"zshexit_functions",
            ShellHook::ZshDirectoryName => c"zsh_directory_name_functions",
        }
    }
}

pub enum ZleWidgetHook {
    ISearchExit,
    ISearchUpdate,
    LinePreRedraw,
    LineInit,
    LineFinish,
    HistoryLineSet,
    KeymapSelect,
}

impl ZleWidgetHook {
    const fn hook_name(&self) -> &'static CStr {
        match self {
            ZleWidgetHook::ISearchExit => c"isearch-exit",
            ZleWidgetHook::ISearchUpdate => c"isearch-update",
            ZleWidgetHook::LinePreRedraw => c"line-pre-redraw",
            ZleWidgetHook::LineInit => c"line-init",
            ZleWidgetHook::LineFinish => c"line-finish",
            ZleWidgetHook::HistoryLineSet => c"history-line-set",
            ZleWidgetHook::KeymapSelect => c"keymap-select",
        }
    }
}

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
            zsh_sys::execstring(script.as_zsh_ptr(), 1, 0, null_mut());
        }
    }

    pub fn add_hook(&self, hook: ShellHook, shell_function_name: &CStr) {
        self.append_param_array(hook.array_name(), &[shell_function_name]);
    }

    pub fn add_zle_hook_widget(&self, hook: ZleWidgetHook, shell_function_name: &CStr) {
        let script = format!(
            "add-zle-hook-widget {:?} {:?}",
            hook.hook_name(),
            shell_function_name
        );
        let cscript = CString::new(script).unwrap();
        self.exec(&cscript);
    }
}

pub(crate) trait CStrUtils {
    fn dup(&self) -> *mut c_char;
    fn metadup(&self) -> *mut c_char;
    fn as_zsh_ptr(&self) -> *mut c_char;
}

impl CStrUtils for &CStr {
    fn metadup(&self) -> *mut c_char {
        unsafe { zsh_sys::ztrdup_metafy(self.as_ptr().cast_mut()) }
    }

    fn dup(&self) -> *mut c_char {
        unsafe { zsh_sys::ztrdup(self.as_ptr()) }
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
