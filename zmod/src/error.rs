use std::fmt::Display;

pub struct ZshErr {
    pub code: i32,
    pub message: String,
}

impl Display for ZshErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
