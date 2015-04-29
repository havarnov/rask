//! A micro web framework.
//!
//! ```no_run
//! use rask::{Rask, Request, Response, StatusCode, Method};
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

extern crate regex;
extern crate hyper;
extern crate url;
extern crate multimap;

use std::net::{Ipv4Addr, SocketAddrV4};
use std::io::Write;
use std::str::FromStr;

use hyper::Server;
use hyper::uri::RequestUri;
use hyper::server::response::Response as HttpResponse;
use hyper::server::request::Request as HttpRequest;
use hyper::server::Handler as HttpHandler;
use hyper::net::Fresh;
pub use hyper::status::StatusCode;
pub use hyper::method::Method;

use url::UrlParser;

use routing::Route;
pub use request::Request;
pub use response::Response;

mod routing;
mod response;
mod request;

/// Trait that all handlers must implement.
///
/// Default implementation for `Fn(&Request, &mut Response)`.
///
/// # Examples
///
/// ```rust
/// use rask::{Handler, Request, Response};
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
    not_found_handler: Box<Handler>,
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
            not_found_handler: Box::new(default_404_handler) }
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
        println!("Starting....");
        let ip = match Ipv4Addr::from_str(host) {
            Ok(addr) => addr,
            Err(e) => panic!(e)
        };
        // FIXME: hard code number of threads, no good.
        Server::http(self).listen_threads(SocketAddrV4::new(ip, port), 2).unwrap();
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
    /// returns a 404 error if non of the handlers match.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use rask::{Rask, Request, Response};
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
    /// use rask::{Rask, Request, Response};
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
        self.not_found_handler = Box::new(handler);
    }

    fn find_route(&self, uri: &str, method: &Method) -> Option<&Route> {
        for route in self.routes.iter() {
            if route.re.is_match(uri) && (route.methods.is_empty() || route.methods.contains(method)) {
                return Some(&route);
            }
        }
        None
    }
}

impl HttpHandler for Rask {
    fn handle(&self, req: HttpRequest, res: HttpResponse<Fresh>) {
        let (url, query_string) = match req.uri {
            RequestUri::AbsolutePath(ref p) => {
                let parser = UrlParser::new();
                match parser.parse_path(p) {
                    Ok((path, query_string, _)) => {
                        (format!("/{0}", path.connect("/")), query_string)
                    },
                    Err(_) => panic!("url parse error")
                }
            },
            _ => panic!("Woot..!")
        };

        println!("{:?} {:?}", req.method, url);

        let mut response = Response { body: "".into(), status: StatusCode::Ok };

        match self.find_route(&url, &req.method) {
            Some(router) => {
                let captures = router.re.captures(&url);
                let request = Request::new(req, captures, query_string);
                (*router.handler).handle(&request, &mut response);
            },
            None => {
                let req = &Request::new(req, None, None);
                self.not_found_handler.handle(req, &mut response);
            }
        }

        let mut res = res;
        *res.status_mut() = response.status;
        let mut result = res.start().unwrap();
        result.write_all((&response.body).as_bytes()).unwrap();
        result.end().unwrap();
    }
}

fn default_404_handler(_: &Request, res: &mut Response) {
    res.body = "404 Not Found".into();
    res.status = StatusCode::NotFound;
}

