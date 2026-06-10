use std::{
    ffi::{CStr, c_void},
    ptr::{self, null_mut},
    rc::Rc,
};

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

    pub fn get_buffer(&self) -> String {
        unsafe {
            let ptr = if !zsh_sys::zlemetaline.is_null() {
                zsh_sys::dupstring(zsh_sys::zlemetaline)
            } else {
                zsh_sys::zlelineasstring(
                    zsh_sys::zleline,
                    zsh_sys::zlell,
                    1,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    1,
                )
            };

            let mut new_len = 0;
            let unmetafied = zsh_sys::unmetafy(ptr, &raw mut new_len);

            CStr::from_ptr(unmetafied).to_string_lossy().to_string()
        }
    }

    pub fn set_region_highlight(&self, ranges: &[RegionHighlight]) {
        unsafe {
            let n_region_highlights = ranges.len() + zsh_sys::N_SPECIAL_HIGHLIGHTS as usize;
            let diffsize = n_region_highlights as i32 - zsh_sys::n_region_highlights;
            zsh_sys::free_region_highlights_memos();
            zsh_sys::region_highlights = zsh_sys::zrealloc(
                zsh_sys::region_highlights as *mut c_void,
                n_region_highlights * size_of::<zsh_sys::region_highlight>(),
            ) as *mut zsh_sys::region_highlight;

            if diffsize > 0 {
                zsh_sys::memset(
                    zsh_sys::region_highlights.add(n_region_highlights - diffsize as usize)
                        as *mut c_void,
                    0,
                    (size_of::<zsh_sys::region_highlight>() * (diffsize as usize)) as u64,
                );
            }
            zsh_sys::n_region_highlights = n_region_highlights as i32;

            for i in 0..ranges.len() {
                let range = ranges.get_unchecked(i);

                let mut atr = 0;

                if let Some(color) = &range.foreground {
                    atr |= zsh_sys::TXTFGCOLOUR as u64
                        | zsh_sys::TXT_ATTR_FG_24BIT as u64
                        | (color.as_rgb() as u64) << zsh_sys::TXT_ATTR_FG_COL_SHIFT;
                }

                if let Some(color) = &range.background {
                    atr |= zsh_sys::TXTBGCOLOUR as u64
                        | zsh_sys::TXT_ATTR_BG_24BIT as u64
                        | (color.as_rgb() as u64) << zsh_sys::TXT_ATTR_BG_COL_SHIFT;
                }

                (*zsh_sys::region_highlights.add(i + zsh_sys::N_SPECIAL_HIGHLIGHTS as usize)) =
                    zsh_sys::region_highlight {
                        atr,
                        start: range.start as i32,
                        start_meta: 0,
                        end: range.end as i32,
                        end_meta: 0,
                        flags: 0,
                        memo: null_mut(),
                    };
            }

            zsh_sys::zrefresh();
        }
    }
}

#[derive(Default, Debug)]
pub struct HighlightColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl HighlightColor {
    fn as_rgb(&self) -> u32 {
        ((((self.red as u32) << 8) + (self.green as u32)) << 8) + (self.blue as u32)
    }
}

#[derive(Default, Debug)]
pub struct RegionHighlight {
    pub start: usize,
    pub end: usize,
    pub foreground: Option<HighlightColor>,
    pub background: Option<HighlightColor>,
}
