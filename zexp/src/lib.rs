use std::{
    ffi::{CStr, CString},
    io::Write as _,
    thread,
    time::Duration,
};

use zmod::{
    Module as _,
    args::Args,
    zsh::{ShellHook, ZleWidgetHook},
};

struct ZExp {
    lines: usize,
}

impl zmod::Module for ZExp {
    fn new() -> Self {
        ZExp { lines: 0 }
    }

    fn setup(&mut self, zsh: zmod::Zsh) {
        zsh.add_hook(ShellHook::PreCmd, Self::FUNCTIONS.__zexp_precmd);
        zsh.add_zle_hook_widget(
            ZleWidgetHook::LinePreRedraw,
            Self::WIDGETS.__zexp_line_pre_redraw,
        );
        let mut writer = zsh.add_zle_fd_listener_widget(Self::WIDGETS.__zexp_periodic);

        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(5));
                writer.write("x").unwrap();
            }
        });
    }
}

#[zmod::module_impl]
impl ZExp {
    #[function]
    fn __zexp_precmd(&mut self, zsh: zmod::Zsh, _args: Args) -> Result<(), zmod::error::ZshErr> {
        let mut buf = Vec::new();
        write!(&mut buf, "lines: {} --> ", self.lines).unwrap();

        let prompt = unsafe { CString::from_vec_unchecked(buf) };

        zsh.set_param_string(c"PROMPT", &prompt);

        self.lines += 1;
        Ok(())
    }

    #[widget]
    fn __zexp_line_pre_redraw(
        &mut self,
        _zsh: zmod::Zsh,
        _zle: zmod::Zle,
        _args: Args,
    ) -> Result<(), zmod::error::ZshErr> {
        Ok(())
    }

    #[widget]
    fn __zexp_periodic(
        &mut self,
        _zsh: zmod::Zsh,
        _zle: zmod::Zle,
        args: Args,
    ) -> Result<(), zmod::error::ZshErr> {
        if let Some(mut reader) = args.as_fd_reader() {
            let _ = reader.read_to_end();
        }

        Ok(())
    }
}
