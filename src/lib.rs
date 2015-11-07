//! A micro web framework.
//!
//! ```no_run
//! use rask::{Rask, StatusCode, Method};
//! use rask::Handler;
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
use std::str::FromStr;
use std::sync::Arc;
use std::borrow::Cow;
use std::collections::HashMap;

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
use request::Request;
use response::Response;
use servestatic::ServeStatic;

pub mod routing;
pub mod response;
pub mod request;
pub mod session;
pub mod cookies;
mod servestatic;

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
        let mut default_error_handlers: HashMap<StatusCode, Arc<Box<Handler>>> = HashMap::new();
        default_error_handlers.insert(StatusCode::NotFound, Arc::new(Box::new(default_404_handler)));
        default_error_handlers.insert(StatusCode::InternalServerError, Arc::new(Box::new(default_500_handler)));
        Rask {
            routes: Vec::new(),
            error_handlers: default_error_handlers }
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

    /// Register a error handler for the specified http status code. This will only have an
    /// effect for NotFound (404) and InternalServerError (500) for now.
    pub fn register_error_handler<H: 'static + Handler>(&mut self, status_code: StatusCode, handler: H) {
        self.error_handlers.insert(status_code, Arc::new(Box::new(handler)));
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
        let serve_static_handler = ServeStatic::new(dir, &path, self.error_handlers[&StatusCode::NotFound].clone());
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
        let mut response = Response::new(res);

        let path = match get_path_and_query_string(&req.uri) {
            Some((path, _)) => path,
            None => {
                let request = Request::new(req, None);
                warn!("Couldn't parse path and/or query string from RequestUri. Failing with 500 error.");
                self.error_handlers[&StatusCode::InternalServerError].handle(&request, response);
                return;
            }
        };

        info!("{:?} {:?}", req.method, path);

        match self.find_route(&path, &req.method) {
            RouteResult::Found(router) => {
                let captures = router.re.captures(&path);
                let request = Request::new(req, captures);
                (*router.handler).handle(&request, response);
            },
            RouteResult::MethodNotAllowed => {
                response.status(StatusCode::MethodNotAllowed);
                let _ = response.write_body("405 Method Not Allowed");
            }
            RouteResult::NotFound => {
                let req = Request::new(req, None);
                self.error_handlers[&StatusCode::NotFound].handle(&req, response);
            }
        }
    }
}

//fn write_response(hyper_res: HttpResponse<Fresh>, rask_res: Response) {
    //let mut rask_res = rask_res;
    //let mut hyper_res = hyper_res;

    //let bytes = rask_res.body.as_bytes();
    //let bytes_len = bytes.len();

    //if let Some(ref cookie_jar) = *rask_res.session.cookie_jar.borrow() {
        //let set_cookie_header = header::SetCookie::from_cookie_jar(cookie_jar);
        //rask_res.headers.set(set_cookie_header);
    //}

    //rask_res.headers.set(header::Server(SERVER_NAME.into()));
    //rask_res.headers.set(header::ContentLength(bytes_len as u64));

    //*hyper_res.status_mut() = rask_res.status;
    //*hyper_res.headers_mut() = rask_res.headers;

    //let mut result = hyper_res.start().unwrap();
    //if bytes_len > 0 {
        //result.write_all(bytes).unwrap();
    //}

    //result.end().unwrap();
//}

fn default_404_handler(_: &Request, res: Response) {
    let mut res = res;
    res.status(StatusCode::NotFound);
    let _ = res.write_body("404 Not Found");
}

fn default_500_handler(_: &Request, res: Response) {
    let mut res = res;
    res.status(StatusCode::InternalServerError);
    let _ = res.write_body("500 Internal server error");
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
