use std::rc::Rc;
use std::cell::RefCell;

use cookie::CookieJar;

use hyper::header::{Headers,Location};
use hyper::status::StatusCode;

use session::Session;
use cookies::Cookies;

pub enum ResponseMarker {}

/// The struct that holds information about the response.
pub struct Response<'a> {
    pub body: String,
    pub status: StatusCode,
    pub headers: Headers,
    pub session: Session<'a, ResponseMarker>,
    pub cookies: Cookies<'a, ResponseMarker>,
}

impl<'a> Response<'a> {
    pub fn new(secret: &[u8]) -> Response {
        let session_jar = Rc::new(RefCell::new(Some(CookieJar::new(secret))));
        let cookies_jar = session_jar.clone();
        Response {
            body: "".into(),
            status: StatusCode::Ok,
            headers: Headers::new(),
            session: Session::new(session_jar),
            cookies: Cookies::new(cookies_jar),
        }
    }

    pub fn no_cookies() -> Response<'a> {
        let empty_jar = Rc::new(RefCell::new(None));
        let empty_jar2 = empty_jar.clone();
        Response {
            body: "".into(),
            status: StatusCode::Ok,
            headers: Headers::new(),
            session: Session::new(empty_jar),
            cookies: Cookies::new(empty_jar2),
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

