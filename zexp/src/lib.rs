mod highlighter;

use std::{
    ffi::{CStr, CString},
    io::Write as _,
    sync::{
        Arc, Mutex,
        mpsc::{self, Sender},
    },
    thread,
};

use highlighter::Highlighter;
use zmod::{
    Module as _,
    args::Args,
    zle::RegionHighlight,
    zsh::{ShellHook, ZleWidgetHook},
};

#[derive(Default)]
struct ZExp {
    region_highlight: Arc<Mutex<Vec<RegionHighlight>>>,
    buffer_sender: Option<Sender<String>>,
}

impl zmod::Module for ZExp {
    fn new() -> Self {
        Self::default()
    }

    fn setup(&mut self, zsh: zmod::Zsh) {
        zsh.add_hook(ShellHook::ChPwd, Self::FUNCTIONS.__zexp_cwd);
        zsh.add_zle_hook_widget(
            ZleWidgetHook::LinePreRedraw,
            Self::WIDGETS.__zexp_line_pre_redraw,
        );
        let writer = zsh.add_zle_fd_listener_widget(Self::WIDGETS.__zexp_async_callback);

        self.compute_prompt(&zsh);

        let (buffer_sender, buffer_receiver) = mpsc::channel::<String>();
        self.buffer_sender = Some(buffer_sender);

        let region_highlight = Arc::clone(&self.region_highlight);
        thread::spawn(move || Highlighter::do_highlight(region_highlight, buffer_receiver, writer));
        println!("Setup is done and dusted");
    }
}

#[zmod::module_impl]
impl ZExp {
    fn compute_prompt(&mut self, zsh: &zmod::Zsh) {
        let dir = std::env::current_dir().unwrap_or("<unknown>".into());

        let dir_segment = if let Some(home) = std::env::home_dir()
            && let Ok(dir) = dir.strip_prefix(&home)
        {
            format!("~/{}", dir.to_string_lossy())
        } else {
            dir.to_string_lossy().to_string()
        };

        let user = std::env::var("USER").unwrap_or("anon".into());

        let mut buf = Vec::new();
        write!(&mut buf, "{user} > {} --> ", dir_segment).unwrap();
        let prompt = unsafe { CString::from_vec_unchecked(buf) };

        zsh.set_param_string(c"PROMPT", &prompt);
    }

    #[function]
    fn __zexp_cwd(&mut self, zsh: zmod::Zsh, _args: Args) -> Result<(), zmod::error::ZshErr> {
        self.compute_prompt(&zsh);
        Ok(())
    }

    #[widget]
    fn __zexp_line_pre_redraw(
        &mut self,
        _zsh: zmod::Zsh,
        zle: zmod::Zle,
        _args: Args,
    ) -> Result<(), zmod::error::ZshErr> {
        if let Some(buffer_sender) = &self.buffer_sender {
            let buffer = zle.get_buffer();
            if let Err(e) = buffer_sender.send(buffer) {
                eprintln!("Buffer sender failed on {e}");
            }
        }
        Ok(())
    }

    #[widget]
    fn __zexp_async_callback(
        &mut self,
        _zsh: zmod::Zsh,
        zle: zmod::Zle,
        args: Args,
    ) -> Result<(), zmod::error::ZshErr> {
        if let Some(mut reader) = args.as_fd_reader() {
            let _ = reader.read_to_end();
        }

        if let Ok(region_highlight) = self.region_highlight.try_lock() {
            zle.set_region_highlight(&region_highlight);
        }

        Ok(())
    }
}
