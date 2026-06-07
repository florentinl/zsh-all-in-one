pub mod error;

#[doc(hidden)]
pub use zsh_sys;

mod module_trait;
mod zle;
pub mod zsh;

pub use zmod_macros::module_impl;

pub use module_trait::Module;
pub use zle::Zle;
pub use zsh::Zsh;

pub mod args;

#[doc(hidden)]
pub mod internal;
