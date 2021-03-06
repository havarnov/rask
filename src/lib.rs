//! A micro web framework.
//!
//! ```
//! use rask::{Rask, StatusCode, Method};
//! use rask::Handler;
//! use rask::request::Request;
//! use rask::response::Response;
//!
//! fn index(req: &Request, res: Response) {
//!     // defaults to status 200 (Ok)
//!     res.send("Hello world!");
//! }
//!
//! fn create(req: &Request, mut res: Response) {
//!     // do something with req.body
//!     res.send(("Hello world!", StatusCode::Created));
//! }
//!
//! fn profile(req: &Request, res: Response) {
//!     let name = req.vars.get("name").unwrap();
//!     res.send(format!("Hello, {0}", name));
//! }
//!
//! fn main() {
//!
//!     let mut app = Rask::new("SUPER SECRET KEY");
//!
//!     app.register("/", index); // all methods
//!     app.register_with_methods("/create", &[Method::Post], create);
//!     app.register_with_methods("/profile/{name}", &[Method::Get], profile);
//!
//!     // must be commented out due to 'rust test'.
//!     // app.run("0.0.0.0", 8080);
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
use std::str::FromStr;
use std::sync::Arc;
use std::collections::HashMap;

use cookie::CookieJar;

use hyper::Server;
use hyper::uri::RequestUri;
use hyper::server::response::Response as HttpResponse;
use hyper::server::request::Request as HttpRequest;
use hyper::server::Handler as HttpHandler;
use hyper::net::Fresh;
pub use hyper::header;
pub use hyper::status::StatusCode;
pub use hyper::method::Method;

use url::UrlParser;

use routing::Route;
use request::Request;
use response::Response;

pub mod routing;
pub mod response;
pub mod request;

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
///     fn handle(&self, req: &Request, res: Response) {
///         // handle request
///     }
/// }
///
/// ```
pub trait Handler: Sync + Send {
    fn handle(&self, &Request, Response);
}

impl<F> Handler for F where F: Fn(&Request, Response), F: Sync + Send {
    fn handle(&self, req: &Request, res: Response) {
        (*self)(req, res);
    }
}

/// The Rask web application.
pub struct Rask {
    routes: Vec<Route>,
    error_handlers: HashMap<StatusCode, Arc<Box<Handler>>>,
    secret: String,
}

impl Rask {
    /// Creates a new Rask web application.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rask::Rask;
    ///
    /// let app = Rask::new("SUPER SECRET KEY");
    /// ```
    pub fn new(secret: &str) -> Rask {
        let mut default_error_handlers: HashMap<StatusCode, Arc<Box<Handler>>> = HashMap::new();
        default_error_handlers.insert(StatusCode::NotFound, Arc::new(Box::new(default_404_handler)));
        default_error_handlers.insert(StatusCode::InternalServerError, Arc::new(Box::new(default_500_handler)));
        Rask {
            routes: Vec::new(),
            error_handlers: default_error_handlers,
            secret: secret.into(),
        }
    }

    /// Starts the web application. Blocks and dispatches new incoming requests.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use rask::Rask;
    ///
    /// let app = Rask::new("SUPER SECRET KEY");
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
        Server::http(SocketAddrV4::new(ip, port)).unwrap().handle(self).unwrap();
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
    /// fn index(_: &Request, _: Response) {
    /// }
    ///
    /// let mut app = Rask::new("SUPER SECRET KEY");
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
    /// fn only_post_and_put(_: &Request, _: Response) {
    /// }
    ///
    /// let mut app = Rask::new("SUPER SECRET KEY");
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

    /// Register a error handler for the specified http status code. This will only have an
    /// effect for NotFound (404) and InternalServerError (500) for now.
    pub fn register_error_handler<H: 'static + Handler>(&mut self, status_code: StatusCode, handler: H) {
        self.error_handlers.insert(status_code, Arc::new(Box::new(handler)));
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
        let cookie_jar = {
            let key = &self.secret.as_bytes();
            match req.headers.get::<header::Cookie>() {
                Some(cookie) => cookie.to_cookie_jar(key),
                None => CookieJar::new(key)
            }
        };

        let response = Response::new(res, cookie_jar);

        let (path, query_string) = match get_path_and_query_string(&req.uri) {
            Some((path, query_string)) => (path, query_string),
            None => {
                let request = Request::new(req, None, None, None);
                warn!("Couldn't parse path and/or query string from RequestUri. Failing with 500 error.");
                self.error_handlers[&StatusCode::InternalServerError].handle(&request, response);
                return;
            }
        };

        info!("{:?} {:?}", req.method, path);

        match self.find_route(&path, &req.method) {
            RouteResult::Found(router) => {
                let captures = router.re.captures(&path);
                let request = Request::new(req, captures, Some(path.clone()), query_string);
                (*router.handler).handle(&request, response);
            },
            RouteResult::MethodNotAllowed => {
                let _ = response.send(("405 Method Not Allowed", StatusCode::MethodNotAllowed));
            }
            RouteResult::NotFound => {
                let req = Request::new(req, None, Some(path), query_string);
                self.error_handlers[&StatusCode::NotFound].handle(&req, response);
            }
        }
    }
}

fn default_404_handler(_: &Request, res: Response) {
    let _ = res.send(("404 Not Found", StatusCode::NotFound));
}

fn default_500_handler(_: &Request, res: Response) {
    let _ = res.send(("500 Internal server error", StatusCode::InternalServerError));
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
