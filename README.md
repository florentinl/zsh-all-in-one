# Building my own zsh experience as a dynamically loaded module

Crates:

- `zexp`: the shell experience that I am building for myself
- `zsh-sys`: auto-generated bindings to zsh's exported functions
- `zmod`: safe wrappers around `zsh-sys`
- `zmod-macros`: proc macros to make it simpler to build a module with `zmod`

The goal is not to have extensive coverage of the zsh API in zmod, just to build what is necessary for me to create `zexp` as I want.

This is more of a personal learning project than anything else but If you want to try it out and have constructive feedback, that's of course always appreciated.
