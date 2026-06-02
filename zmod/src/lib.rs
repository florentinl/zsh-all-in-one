pub mod error;

#[doc(hidden)]
pub use zsh_sys;
pub mod builtin;

mod module_trait;
mod zsh;

pub use module_trait::Module;
pub use zmod_macros::builtin;
pub use zmod_macros::module_impl;
pub use zsh::Zsh;
