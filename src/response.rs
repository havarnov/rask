use cookie::{CookieJar,Cookie};

use hyper::header::{Headers,Location};
use hyper::status::StatusCode;

/// The struct that holds information about the response.
pub struct Response<'a> {
    pub body: String,
    pub status: StatusCode,
    pub headers: Headers,
    pub cookie_jar: Option<CookieJar<'a>>,
}

impl<'a> Response<'a> {
    pub fn new(secret: &[u8]) -> Response {
        Response {
            body: "".into(),
            status: StatusCode::Ok,
            headers: Headers::new(),
            cookie_jar: Some(CookieJar::new(secret)),
        }
    }

    pub fn no_cookies() -> Response<'a> {
        Response {
            body: "".into(),
            status: StatusCode::Ok,
            headers: Headers::new(),
            cookie_jar: None,
        }
    }

    pub fn redirect(&mut self, location: &str) {
        self.status = StatusCode::Found;
        // TODO: what's correct behaviour if 'Location' is already set.
        if !self.headers.has::<Location>() {
            self.headers.set(Location(location.into()));
        }
    }

    pub fn set_cookie(&mut self, key: &str, value: &str) {
        match self.cookie_jar {
            Some(ref mut cookie_jar) => cookie_jar.add(Cookie::new(key.into(), value.into())),
            None => panic!("trying to set a cookie on a non coookie Response")
        };
    }

    pub fn set_session(&mut self, key: &str, value: &str) {
        match self.cookie_jar {
            Some(ref mut cookie_jar) => cookie_jar.encrypted().add(Cookie::new(key.into(), value.into())),
            None => panic!("trying to set a session on a non coookie Response")
        }
    }

    pub fn pop_session(&self, key: &str) {
        if let Some(ref jar) = self.cookie_jar {
            jar.encrypted().remove(key);
        }
    }
}

