use hyper::status::StatusCode;

pub struct Response {
    pub body: String,
    pub status: StatusCode,
}
