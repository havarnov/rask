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

    pub fn redirect(&mut self, location: &str) {
        self.status = StatusCode::Found;
        // TODO: what's correct behaviour if 'Location' is already set.
        if !self.headers.has::<Location>() {
            self.headers.set(Location(location.into()));
        }
    }
}

