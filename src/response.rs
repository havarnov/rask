use hyper::status::StatusCode;

/// The struct that holds information about the response.
pub struct Response {
    pub body: String,
    pub status: StatusCode,
}
