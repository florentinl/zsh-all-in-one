pub trait Module {
    fn new() -> Self;
    fn setup(&mut self, zsh: crate::Zsh) {
        let _ = zsh;
    }
}
