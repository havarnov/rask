use hyper::header::{Headers,Location};
use hyper::status::StatusCode;

/// The struct that holds information about the response.
pub struct Response {
    pub body: String,
    pub status: StatusCode,
    pub headers: Headers,
}

impl Response {
    pub fn new() -> Response {
        Response {
            body: "".into(),
            status: StatusCode::Ok,
            headers: Headers::new(),
        }
    }
}

pub fn redirect(res: &mut Response, location: &str) {
    res.status = StatusCode::Found;
    res.headers.set(Location(location.into()));
}
