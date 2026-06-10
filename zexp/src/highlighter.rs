use std::sync::{Arc, Mutex, mpsc::Receiver};

use syntect::{
    easy::HighlightLines, highlighting::ThemeSet, parsing::SyntaxSet, util::LinesWithEndings,
};
use zmod::{zle::RegionHighlight, zsh::WidgetFdWriter};

pub struct Highlighter;

impl Highlighter {
    pub fn do_highlight(
        region_highlight: Arc<Mutex<Vec<RegionHighlight>>>,
        buffer_receiver: Receiver<String>,
        mut writer: WidgetFdWriter,
    ) {
        let syntax_set = SyntaxSet::load_defaults_newlines();

        let zsh_syntax = syntax_set
            .find_syntax_by_name("Shell-Unix-Generic")
            .unwrap();

        let theme = ThemeSet::load_defaults().themes["base16-ocean.dark"].clone();

        loop {
            if let Ok(buffer) = buffer_receiver.recv() {
                let mut buffer = buffer;
                while let Ok(newer_buffer) = buffer_receiver.try_recv() {
                    buffer = newer_buffer;
                }

                let mut highlighter = HighlightLines::new(zsh_syntax, &theme);
                let mut ranges: Vec<RegionHighlight> = vec![];
                let mut offset = 0;

                for line in LinesWithEndings::from(&buffer) {
                    let regions = &highlighter.highlight_line(line, &syntax_set).unwrap();

                    for (style, content) in regions {
                        let len = content.chars().count();
                        ranges.push(RegionHighlight {
                            start: offset,
                            end: offset + len,
                            foreground: Some(zmod::zle::HighlightColor {
                                red: style.foreground.r,
                                green: style.foreground.g,
                                blue: style.foreground.b,
                            }),
                            background: None,
                        });

                        offset += len;
                    }
                }

                if let Ok(mut lrh) = region_highlight.lock() {
                    *lrh = ranges;

                    drop(lrh);

                    if let Err(e) = writer.write("x") {
                        eprintln!("Highlighter thread while writing to fd: {e}");
                        return;
                    }
                }
            } else {
                return;
            }
        }
    }
}
