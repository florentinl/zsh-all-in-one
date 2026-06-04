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
        zsh.set_param_array(c"precmd_functions", &[c"prompt_precmd"]);
    }
}

#[zmod::module_impl]
impl MyModule {
    #[zmod::builtin]
    fn mybuiltin(&mut self, _zsh: zmod::Zsh, args: Args) -> Result<(), zmod::error::ZshErr> {
        for (i, arg) in args.enumerate() {
            let str = arg.to_string_lossy();
            println!("{i}: {str}");
        }
        Ok(())
    }

    #[zmod::function]
    fn prompt_precmd(&mut self, zsh: zmod::Zsh, _args: Args) -> Result<(), zmod::error::ZshErr> {
        let mut buf = Vec::new();
        write!(&mut buf, "lines: {} --> ", self.lines).unwrap();

        let prompt = unsafe { CString::from_vec_unchecked(buf) };

        zsh.set_param_string(c"PROMPT", &prompt);

        self.lines += 1;
        Ok(())
    }
}
