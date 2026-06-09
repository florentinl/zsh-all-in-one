use std::{
    ffi::{CStr, CString},
    io::Write as _,
};

use zmod::{Module as _, args::Args, zsh::ShellHook};

struct ZExp {}

impl zmod::Module for ZExp {
    fn new() -> Self {
        ZExp {}
    }

    fn setup(&mut self, zsh: zmod::Zsh) {
        zsh.add_hook(ShellHook::ChPwd, Self::FUNCTIONS.__zexp_cwd);
        self.compute_prompt(&zsh);
    }
}

#[zmod::module_impl]
impl ZExp {
    fn compute_prompt(&mut self, zsh: &zmod::Zsh) {
        let dir = std::env::current_dir().unwrap_or("<unknown>".into());

        let dir_segment = if let Some(home) = std::env::home_dir()
            && let Ok(dir) = dir.strip_prefix(&home)
        {
            format!("~/{}", dir.to_string_lossy())
        } else {
            dir.to_string_lossy().to_string()
        };

        let user = std::env::var("USER").unwrap_or("anon".into());

        let mut buf = Vec::new();
        write!(&mut buf, "{user} > {} --> ", dir_segment).unwrap();
        let prompt = unsafe { CString::from_vec_unchecked(buf) };

        zsh.set_param_string(c"PROMPT", &prompt);
    }

    #[function]
    fn __zexp_cwd(&mut self, zsh: zmod::Zsh, _args: Args) -> Result<(), zmod::error::ZshErr> {
        self.compute_prompt(&zsh);
        Ok(())
    }
}
