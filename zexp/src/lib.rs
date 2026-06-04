use zmod::{Module as _, builtin::BuiltinArgs};

struct MyModule;

impl zmod::Module for MyModule {
    fn new() -> Self {
        MyModule
    }
}

#[zmod::module_impl]
impl MyModule {
    #[zmod::builtin]
    fn mybuiltin(&mut self, _zsh: zmod::Zsh, args: BuiltinArgs) -> Result<(), zmod::error::ZshErr> {
        for (i, arg) in args.enumerate() {
            let str = arg.to_string_lossy();
            println!("{i}: {str}");
        }
        Ok(())
    }

    #[zmod::builtin]
    fn param_set_string(
        &mut self,
        zsh: zmod::Zsh,
        _args: BuiltinArgs,
    ) -> Result<(), zmod::error::ZshErr> {
        zsh.set_param_string(c"MY_STRING", c"IS_DEFINITELY_A_STRING");
        Ok(())
    }
}
