//! A micro web framework.
//!
//! ```no_run
//! use rask::{Rask, StatusCode, Method};
//! use rask::request::Request;
//! use rask::response::Response;
//!
//! fn index(req: &Request, res: &mut Response) {
//!     res.body = "Hello world!".into();
//!     // defaults to Statuscode::Ok
//! }
//!
//! fn create(req: &Request, res: &mut Response) {
//!     // do something with req.body
//!     res.body = "something created".into();
//!     res.status = StatusCode::Created;
//! }
//!
//! fn profile(req: &Request, res: &mut Response) {
//!     let name = req.vars.get("name").unwrap();
//!     res.body = format!("Hello, {0}", name);
//! }
//!
//! fn main() {
//!
//!     let mut app = Rask::new();
//!
//!     app.register("/", index); // all methods
//!     app.register_with_methods("/create", &[Method::Post], create);
//!     app.register_with_methods("/profile/{name}", &[Method::Get], profile);
//!
//!     app.run("0.0.0.0", 8080);
//! }
//! ```

#[macro_use]
extern crate log;

extern crate regex;
extern crate hyper;
extern crate url;
extern crate multimap;
extern crate cookie;

use std::net::{Ipv4Addr, SocketAddrV4};
use std::io::Write;
use std::str::FromStr;
use std::sync::Arc;
use std::borrow::Cow;

use hyper::Server;
use hyper::header;
use hyper::uri::RequestUri;
use hyper::server::response::Response as HttpResponse;
use hyper::server::request::Request as HttpRequest;
use hyper::server::Handler as HttpHandler;
use hyper::net::Fresh;
pub use hyper::status::StatusCode;
pub use hyper::method::Method;

use url::UrlParser;

use routing::Route;
use request::Request;
use response::Response;
use servestatic::ServeStatic;

pub mod routing;
pub mod response;
pub mod request;
pub mod session;
pub mod cookies;
mod servestatic;

const SECRET: &'static str = "SUPER SECRET STRING";

/// Trait that all handlers must implement.
///
/// Default implementation for `Fn(&Request, &mut Response)`.
///
/// # Examples
///
/// ```rust
/// use rask::Handler;
/// use rask::request::Request;
/// use rask::response::Response;
///
/// struct FooHandler {
///     something: usize,
/// }
///
/// impl Handler for FooHandler {
///     fn handle(&self, req: &Request, res: &mut Response) {
///         // handle request
///     }
/// }
///
/// ```
pub trait Handler: Sync + Send {
    fn handle(&self, &Request, &mut Response);
}

impl<F> Handler for F where F: Fn(&Request, &mut Response), F: Sync + Send {
    fn handle(&self, req: &Request, res: &mut Response) {
        (*self)(req, res);
    }
}

/// The Rask web application.
pub struct Rask {
    routes: Vec<Route>,
    not_found_handler: Arc<Box<Handler>>,
}

impl Rask {
    /// Creates a new Rask web application.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rask::Rask;
    ///
    /// let app = Rask::new();
    /// ```
    pub fn new() -> Rask {
        Rask {
            routes: Vec::new(),
            not_found_handler: Arc::new(Box::new(default_404_handler)) }
    }

    /// Starts the web application. Blocks and dispatches new incoming requests.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use rask::Rask;
    ///
    /// let app = Rask::new();
    /// app.run("127.0.0.1", 8080);
    /// ```
    ///
    /// # Panics
    ///
    /// This method panics if `host` can´t be parsed to a Ipv4Addr or that it fails to start
    /// the web application for the given host and port.
    pub fn run(self, host: &str, port: u16) {
        // TODO: What about Ipv6Addr?
        let ip = match Ipv4Addr::from_str(host) {
            Ok(addr) => addr,
            Err(e) => panic!(e)
        };
        info!("Running on {:?}:{:?}", host, port);
        // FIXME: hard code number of threads, no good.
        Server::http(SocketAddrV4::new(ip, port)).unwrap().handle_threads(self, 2).unwrap();
    }

    /// Register a handler for a given route. Rask will dispatch request that matches the
    /// route to the handler for all http methods.
    ///
    /// `route` must be on the following syntax:
    ///
    /// * "/" -> requests matching the "/" literal.
    /// * "/profile" -> requests matching the "/profile" literal.
    /// * "/{name}" -> requests matching any requests with "/" + a word.
    /// The name variable will be
    /// accesible from `Request.vars`.
    ///
    /// Rask will search for a matching handler in the order they are registered and
    /// either returns a 405 (Method not allowed) or a 404 (Not found) error.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use rask::Rask;
    /// use rask::request::Request;
    /// use rask::response::Response;
    ///
    /// fn index(_: &Request, _: &mut Response) {
    /// }
    ///
    /// let mut app = Rask::new();
    /// app.register("/", index);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the given route can't be compiled to a valid regex.
    pub fn register<H: 'static + Handler>(&mut self, route: &str, handler: H) {
        let route = Route::new(route, handler);
        self.routes.push(route);
    }

    /// Same as `register`, but also specifies which http methods the handler will receive.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rask::Rask;
    /// use rask::request::Request;
    /// use rask::response::Response;
    /// use rask::Method::*;
    ///
    /// fn only_post_and_put(_: &Request, _: &mut Response) {
    /// }
    ///
    /// let mut app = Rask::new();
    /// app.register_with_methods("/", &[Post, Put], only_post_and_put);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the given route can't be compiled to a valid regex.
    pub fn register_with_methods<H: 'static + Handler>(
        &mut self,
        route: &str,
        methods: &[Method],
        handler: H)
    {
        let route = Route::with_methods(route, handler, methods);
        self.routes.push(route);
    }

    /// Register a 404 handler. This handler will be called if an incoming request
    /// doesn't match any of the registered routes.
    ///
    /// If no 404 handler is registered a default handler will be called instead.
    pub fn register_404_handler<H: 'static + Handler>(&mut self, handler: H) {
        self.not_found_handler = Arc::new(Box::new(handler));
    }

    /// Setup the app to serve a directory as static resources. Typically used for
    /// html, js and css.
    ///
    /// ```rust
    ///
    /// use rask::Rask;
    /// use rask::request::Request;
    /// use rask::response::Response;
    ///
    /// let mut app = Rask::new();
    /// app.serve_static("/static/", "static/");
    /// ```
    ///
    pub fn serve_static(&mut self, path: &str, dir: &str) {
        let path = trailing_slash(path);
        let serve_static_handler = ServeStatic::new(dir, &path, self.not_found_handler.clone());
        let route = Route::with_methods(&format!("{}**", path), serve_static_handler, &[Method::Get]);
        self.routes.push(route);
    }

    fn find_route(&self, path: &str, method: &Method) -> RouteResult {
        for route in self.routes.iter() {
            if route.re.is_match(path) {
                if route.methods.is_empty() || route.methods.contains(method) {
                    return RouteResult::Found(&route);
                }
                else {
                    return RouteResult::MethodNotAllowed;
                }
            }
        }

        RouteResult::NotFound
    }
}

enum RouteResult<'a> {
    Found(&'a Route),
    MethodNotAllowed,
    NotFound,
}


impl HttpHandler for Rask {
    fn handle(&self, req: HttpRequest, res: HttpResponse<Fresh>) {
        let (path, query_string) = match get_path_and_query_string(&req.uri) {
            Some(u_q) => u_q,
            None => {
                write_500_error(res);
                return;
            }
        };

        info!("{:?} {:?}", req.method, path);

        let mut response = Response::new(SECRET.as_bytes());

        match self.find_route(&path, &req.method) {
            RouteResult::Found(router) => {
                let captures = router.re.captures(&path);
                let request = Request::new(req, captures, query_string, SECRET.as_bytes());
                (*router.handler).handle(&request, &mut response);
            },
            RouteResult::MethodNotAllowed => {
                response.body = "405 Method Not Allowed".into();
                response.status = StatusCode::MethodNotAllowed;
            }
            RouteResult::NotFound => {
                let req = &Request::new(req, None, None, SECRET.as_bytes());
                self.not_found_handler.handle(req, &mut response);
            }
        }

        write_response(res, response);
    }
}

fn write_500_error(hyper_res: HttpResponse<Fresh>) {
    let mut res = Response::no_cookies();
    res.body = "500 Internal server error".into();
    res.status = StatusCode::InternalServerError;
    write_response(hyper_res, res);
}

const SERVER_NAME: &'static str = "Rask/0.0.1 Rust/1.0-beta2";

fn write_response(hyper_res: HttpResponse<Fresh>, rask_res: Response) {
    let mut rask_res = rask_res;
    let mut hyper_res = hyper_res;

    let bytes = rask_res.body.as_bytes();
    let bytes_len = bytes.len();

    if let Some(ref cookie_jar) = *rask_res.session.cookie_jar.borrow() {
        let set_cookie_header = header::SetCookie::from_cookie_jar(&cookie_jar);
        rask_res.headers.set(set_cookie_header);
    }

    rask_res.headers.set(header::Server(SERVER_NAME.into()));
    rask_res.headers.set(header::ContentLength(bytes_len as u64));

    *hyper_res.status_mut() = rask_res.status;
    *hyper_res.headers_mut() = rask_res.headers;

    let mut result = hyper_res.start().unwrap();
    if bytes_len > 0 {
        result.write_all(bytes).unwrap();
    }

    result.end().unwrap();
}

fn default_404_handler(_: &Request, res: &mut Response) {
    res.body = "404 Not Found".into();
    res.status = StatusCode::NotFound;
}

fn trailing_slash<'a>(i: &'a str) -> Cow<'a, str> {
    if !i.ends_with("/") {
        Cow::Owned(format!("{}/", i))
    }
    else {
        Cow::Borrowed(i)
    }
}

fn get_path_and_query_string(uri: &RequestUri) -> Option<(String, Option<String>)> {
    match *uri {
        RequestUri::AbsolutePath(ref p) => {
            let parser = UrlParser::new();
            match parser.parse_path(p) {
                Ok((path, query_string, _)) => {
                    Some((path.iter().fold(String::from(""), |a, b| a + "/" + &b), query_string))
                },
                Err(_) => {
                    error!("Couldn't parse path: {:?}.", p);
                    None
                }
            }
        },
        ref uri => {
            error!("Not supported 'RequestUri': {:?}.", uri);
            None
        }
    }
}
