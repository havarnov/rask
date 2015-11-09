use std::any::Any;
use std::io::Result as IoResult;
use std::borrow::Cow;

use hyper::server::response::Response as HttpResponse;
use hyper::status::StatusCode;
use hyper::net::Fresh;
use hyper::header;
use hyper::header::Header;
use hyper::header::HeaderFormat;

use cookie::CookieJar;

/// The struct that holds information about the response.
pub struct Response<'a, W: Any = Fresh> {
    inner: HttpResponse<'a, W>,
    cookie_jar: CookieJar<'static>
}

pub trait Sendable<'a> {
    fn decode(self) -> (Cow<'a, [u8]>, StatusCode);
}

impl<'a> Sendable<'a> for String {
    fn decode(self) -> (Cow<'a, [u8]>, StatusCode) {
        (Cow::Owned(self.into_bytes()), StatusCode::Ok)
    }
}

impl<'a> Sendable<'a> for &'a str {
    fn decode(self) -> (Cow<'a, [u8]>, StatusCode) {
        (Cow::Borrowed(self.as_bytes()), StatusCode::Ok)
    }
}

impl<'a> Sendable<'a> for (&'a str, StatusCode) {
    fn decode(self) -> (Cow<'a, [u8]>, StatusCode) {
        (Cow::Borrowed(self.0.as_bytes()), self.1)
    }
}

impl<'a> Sendable<'a> for StatusCode {
    fn decode(self) -> (Cow<'a, [u8]>, StatusCode) {
        (Cow::Borrowed("".as_bytes()), self)
    }
}

impl<'a> Response<'a, Fresh> {
    pub fn new(res: HttpResponse<'a, Fresh>, cookie_jar: CookieJar<'static>) -> Response<'a, Fresh> {
        Response {
            inner: res,
            cookie_jar: cookie_jar
        }
    }

    pub fn status(&mut self, status: StatusCode) {
        *self.inner.status_mut() = status;
    }

    pub fn set_header<H: Header + HeaderFormat>(&mut self, header: H) {
        self.inner.headers_mut().set(header);
    }

    pub fn cookies<'b>(&'b mut self) -> &'b mut CookieJar<'static> {
        &mut self.cookie_jar
    }

    pub fn send<S: 'a + Sendable<'a>>(mut self, s: S) -> IoResult<()> {
        let cookie = header::SetCookie::from_cookie_jar(&self.cookie_jar);
        self.set_header(cookie);

        let (content, status) = s.decode();
        self.status(status);
        if content.len() > 0 {
            self.set_header(header::ContentLength(content.len() as u64));
        }
        self.inner.send(&content)
    }

    pub fn redirect(mut self, path: &str) -> IoResult<()> {
        self.set_header(header::Location(path.to_owned()));
        self.send(StatusCode::Found)
    }
}

