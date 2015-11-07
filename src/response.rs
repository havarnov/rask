use std::any::Any;
use std::io::Result as IoResult;

use hyper::server::response::Response as HttpResponse;
use hyper::status::StatusCode;
use hyper::net::Fresh;
use hyper::header;
use hyper::header::Header;
use hyper::header::HeaderFormat;

/// The struct that holds information about the response.
pub struct Response<'a, W: Any = Fresh> {
    inner: HttpResponse<'a, W>
}

impl<'a> Response<'a, Fresh> {
    pub fn new(res: HttpResponse<'a, Fresh>) -> Response<'a, Fresh> {
        Response {
            inner: res
        }
    }

    pub fn status(&mut self, status: StatusCode) {
        *self.inner.status_mut() = status;
    }

    pub fn set_header<H: Header + HeaderFormat>(&mut self, header: H) {
        self.inner.headers_mut().set(header);
    }

    pub fn write_body(self, body: &str) -> IoResult<()> {
        let mut mut_self = self;

        let bytes = body.as_bytes();
        mut_self.set_header(header::ContentLength(bytes.len() as u64));
        mut_self.inner.send(&body.as_bytes())
    }
}

