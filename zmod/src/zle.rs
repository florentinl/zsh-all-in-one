use std::rc::Rc;

/// Zero-sized capability proving "you are called from a zle widget"
pub struct Zle<'z> {
    _no_send_sync: Rc<()>,
    _scope: std::marker::PhantomData<&'z mut ()>,
}

impl<'z> Zle<'z> {
    #[doc(hidden)]
    pub fn new() -> Self {
        Self {
            _no_send_sync: Rc::new(()),
            _scope: Default::default(),
        }
    }
}
