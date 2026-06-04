use zmod::{Module as _, args::Args};

struct MyModule;

impl zmod::Module for MyModule {
    fn new() -> Self {
        MyModule
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

    #[zmod::builtin]
    fn param_set_string(&mut self, zsh: zmod::Zsh, _args: Args) -> Result<(), zmod::error::ZshErr> {
        zsh.set_param_string(c"MY_STRING", c"IS_DEFINITELY_A_STRING");
        Ok(())
    }

    #[zmod::function]
    fn myfunction(&mut self, zsh: zmod::Zsh, _args: Args) -> Result<(), zmod::error::ZshErr> {
        println!("Called a zsh function named: myfunction");
        zsh.set_param_string(c"MY_STRING", c"IS_DEFINITELY_ANOTHER_STRING");
        Ok(())
    }
}
