use std::rc::Rc;
use std::cell::RefCell;

use cookie::CookieJar;

use hyper::header::{Headers,Location};
use hyper::status::StatusCode;

use session::Session;

/// The struct that holds information about the response.
pub struct Response<'a> {
    pub body: String,
    pub status: StatusCode,
    pub headers: Headers,
    pub session: Session<'a>,
}

impl<'a> Response<'a> {
    pub fn new(secret: &[u8]) -> Response {
        Response {
            body: "".into(),
            status: StatusCode::Ok,
            headers: Headers::new(),
            session: Session::new(Rc::new(RefCell::new(Some(CookieJar::new(secret))))),
        }
    }

    pub fn no_cookies() -> Response<'a> {
        Response {
            body: "".into(),
            status: StatusCode::Ok,
            headers: Headers::new(),
            session: Session::new(Rc::new(RefCell::new(None))),
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

