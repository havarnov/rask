use std::fs::File;
use std::path::PathBuf;
use std::io::Read;
use std::sync::Arc;

use hyper::status::StatusCode;

use super::Handler;
use request::Request;
use response::Response;

pub struct ServeStatic {
    root: PathBuf,
    prefix: String,
    not_found_handler: Arc<Box<Handler>>,
}

impl ServeStatic {
    pub fn new(root: &str, prefix: &str, not_found_handler: Arc<Box<Handler>>) -> ServeStatic {
        ServeStatic {
            root: PathBuf::from(root),
            prefix: prefix.into(),
            not_found_handler: not_found_handler
        }
    }
}

impl Handler for ServeStatic {
    fn handle(&self, req: &Request, mut res: Response) {
        let path = match req.path {
            Some(ref path) => path,
            None => {
                res.status(StatusCode::InternalServerError);
                let _ = res.write_body("Internal 500 error");
                return;
            }
        };

        match File::open(self.root.join(path.trim_left_matches(&self.prefix))) {
            Ok(ref mut file) => {
                let mut buffer = String::new();

                // FIXME: handle error.
                let _ = file.read_to_string(&mut buffer);
                let _ = res.write_body(&buffer);
            }
            Err(_) => {
                self.not_found_handler.handle(req, res);
            }
        }
    }
}
