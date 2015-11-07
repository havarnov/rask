use std::any::Any;

use hyper::server::response::Response as HttpResponse;
use hyper::status::StatusCode;
use hyper::net::Fresh;

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

    pub fn write_body(self, body: &str) -> ::std::io::Result<()> {
        self.inner.send(&body.as_bytes())
    }
}

