pub mod error;

#[doc(hidden)]
pub use zsh_sys;

mod module_trait;
mod zsh;

pub use zmod_macros::builtin;
pub use zmod_macros::function;
pub use zmod_macros::module_impl;

pub use module_trait::Module;
pub use zsh::Zsh;

pub mod args;
