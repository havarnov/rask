use std::fs::File;
use std::path::PathBuf;

pub struct ServeStatic {
    pub root: PathBuf,
}

impl ServeStatic {
    pub fn find(&self, path: &str) -> Option<File> {
        let path = path.trim_left_matches("/");
        match File::open(self.root.join(path)) {
            Ok(file) => Some(file),
            Err(_) => None
        }
    }
}
