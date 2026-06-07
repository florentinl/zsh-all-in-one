use std::{ffi::CString, io::Write as _};

use zmod::{Module as _, args::Args};

struct MyModule {
    lines: usize,
}

impl zmod::Module for MyModule {
    fn new() -> Self {
        MyModule { lines: 0 }
    }

    fn setup(&mut self, zsh: zmod::Zsh) {
        zsh.append_param_array(c"precmd_functions", &[c"prompt_precmd"]);
        zsh.exec(c"bindkey \"^H\" custom_widget");
    }
}

#[zmod::module_impl]
impl MyModule {
    #[builtin]
    fn mybuiltin(&mut self, _zsh: zmod::Zsh, args: Args) -> Result<(), zmod::error::ZshErr> {
        for (i, arg) in args.enumerate() {
            let str = arg.to_string_lossy();
            println!("{i}: {str}");
        }
        Ok(())
    }

    #[widget]
    fn custom_widget(
        &mut self,
        _zsh: zmod::Zsh,
        _zle: zmod::Zle,
        args: Args,
    ) -> Result<(), zmod::error::ZshErr> {
        println!("Called from a custom_widget");

        for (i, arg) in args.enumerate() {
            let str = arg.to_string_lossy();
            println!("{i}: {str}");
        }

        Ok(())
    }

    #[function]
    fn prompt_precmd(&mut self, zsh: zmod::Zsh, _args: Args) -> Result<(), zmod::error::ZshErr> {
        let mut buf = Vec::new();
        write!(&mut buf, "lines: {} --> ", self.lines).unwrap();

        let prompt = unsafe { CString::from_vec_unchecked(buf) };

        zsh.set_param_string(c"PROMPT", &prompt);

        self.lines += 1;
        Ok(())
    }
}
