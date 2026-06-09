use std::{
    ffi::{CStr, CString, c_char},
    fs::File,
    io::{ErrorKind, Read as _, Write as _},
    os::{fd::IntoRawFd as _, unix::fs::OpenOptionsExt as _},
    ptr::null_mut,
    rc::Rc,
    str::FromStr as _,
};

use nix::{sys::stat::Mode, unistd};
use tempfile::TempDir;

use crate::internal::zle::bind_filedesc;

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

    pub fn add_zle_fd_listener_widget(&self, widget_name: &CStr) -> WidgetFdWriter {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(widget_name.to_string_lossy().to_string());

        unistd::mkfifo(&path, Mode::S_IRUSR | Mode::S_IWUSR).unwrap();

        let write_end_file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(nix::libc::O_APPEND)
            .open(&path)
            .expect("Failed to open fifo for writing");

        let read_end_file = std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(nix::libc::O_NONBLOCK)
            .open(&path)
            .expect("Failed to open fifo for reading");

        bind_filedesc(read_end_file.into_raw_fd(), widget_name);

        WidgetFdWriter {
            file: write_end_file,
            _temp_dir: dir,
        }
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

pub struct WidgetFdWriter {
    file: File,
    #[doc(hidden)]
    _temp_dir: TempDir,
}

impl WidgetFdWriter {
    pub fn write(&mut self, value: &str) -> std::io::Result<usize> {
        self.file.write(value.as_bytes())
    }
}
pub struct WidgetFdReader {
    pub(crate) file: Option<File>,
}

impl WidgetFdReader {
    pub fn read_to_end(&mut self) -> Result<String, String> {
        if let Some(file) = self.file.as_mut() {
            let mut content = vec![];
            let mut buf = [0u8; 64];
            loop {
                match file.read(&mut buf) {
                    Ok(0) => break,
                    Ok(len) => content.extend_from_slice(&buf[..len]),
                    Err(err) if err.kind() == ErrorKind::Interrupted => continue,
                    Err(err) if err.kind() == ErrorKind::WouldBlock => break,
                    Err(err) => return Err(err.to_string()),
                }
            }
            let content = String::from_utf8(content).map_err(|e| e.to_string())?;
            Ok(content)
        } else {
            Err(String::from_str("No files available").unwrap())
        }
    }
}

impl Drop for WidgetFdReader {
    fn drop(&mut self) {
        if let Some(file) = self.file.take() {
            // we should not close this file ever
            let _ = file.into_raw_fd();
        }
    }
}
