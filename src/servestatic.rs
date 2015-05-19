use std::fs::File;
use std::path::PathBuf;
use std::io::Read;
use std::sync::Arc;

use hyper::status::StatusCode;

use super::Handler;
use super::get_path_and_query_string;
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
    fn handle(&self, req: &Request, res: &mut Response) {
        let path = match get_path_and_query_string(&req.uri) {
            Some((path, _)) => path,
            None => {
                res.body = "Internal 500 error".into();
                res.status = StatusCode::InternalServerError;
                return;
            }
        };

        match File::open(self.root.join(path.trim_left_matches(&self.prefix))) {
            Ok(ref mut file) => {
                // FIXME: handle error.
                let _ = file.read_to_string(&mut res.body);
            }
            Err(_) => {
                self.not_found_handler.handle(req, res);
            }
        }
    }
}
