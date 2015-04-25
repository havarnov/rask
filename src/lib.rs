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

pub trait Handler: Sync + Send {
    fn handle(&self, &Request, &mut Response);
}

impl<F> Handler for F where F: Fn(&Request, &mut Response), F: Sync + Send {
    fn handle(&self, req: &Request, res: &mut Response) {
        (*self)(req, res);
    }
}

pub struct Rask {
    routes: Vec<Route>
}

impl Rask {
    pub fn new() -> Rask {
        Rask { routes: Vec::new() }
    }

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

    pub fn register<H: 'static + Handler>(&mut self, route: &str, handler: H) {
        let route = Route::new(route, handler);
        self.routes.push(route);
    }

    pub fn register_with_methods<H: 'static + Handler>(
        &mut self,
        route: &str,
        methods: &[Method],
        handler: H)
    {
        let route = Route::with_methods(route, handler, methods);
        self.routes.push(route);
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
            None => default_404_handler(&Request::new(req, None, None), &mut response)
        }

        let mut res = res;
        *res.status_mut() = response.status;
        let mut result = res.start().unwrap();
        result.write_all((&response.body).as_bytes()).unwrap();
        result.end().unwrap();
    }
}

fn default_404_handler(_: &Request, res: &mut Response) {
    res.body = format!("Page not found");
    res.status = StatusCode::NotFound;
}

