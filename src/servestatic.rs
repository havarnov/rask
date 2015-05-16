use std::fs::File;
use std::path::PathBuf;

pub struct ServeStatic {
    pub root: PathBuf,
}

impl ServeStatic {
    pub fn find(&self, path: &str) -> Option<File> {
        let mut path: String = path.into();
        let _  = path.remove(0);
        match File::open(self.root.join(path)) {
            Ok(file) => Some(file),
            Err(_) => None
        }
    }
}
