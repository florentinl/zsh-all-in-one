use std::{
    ffi::{CStr, CString},
    io::Write as _,
};

use zmod::{Module as _, args::Args, zsh::ShellHook};

struct ZExp {
    lines: usize,
}

impl zmod::Module for ZExp {
    fn new() -> Self {
        ZExp { lines: 0 }
    }

    fn setup(&mut self, zsh: zmod::Zsh) {
        zsh.add_hook(ShellHook::PreCmd, Self::FUNCTIONS.__zexp_prompt_precmd);
    }
}

#[zmod::module_impl]
impl ZExp {
    #[function]
    fn __zexp_prompt_precmd(
        &mut self,
        zsh: zmod::Zsh,
        _args: Args,
    ) -> Result<(), zmod::error::ZshErr> {
        let mut buf = Vec::new();
        write!(&mut buf, "lines: {} --> ", self.lines).unwrap();

        let prompt = unsafe { CString::from_vec_unchecked(buf) };

        zsh.set_param_string(c"PROMPT", &prompt);

        self.lines += 1;
        Ok(())
    }
}
