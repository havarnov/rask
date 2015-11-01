use std::collections::HashMap;
use std::io::Read;
use std::rc::Rc;
use std::cell::RefCell;

use cookie::Cookie;

use regex::Captures;

use hyper::header;
use hyper::header::Headers;
use hyper::method::Method;
use hyper::server::request::Request as HttpRequest;
use hyper::uri::RequestUri;

use multimap::MultiMap;

use session::Session;
use cookies::Cookies;

pub enum RequestMarker {}

/// The struct that holds information about the incoming Request. The handlers will borrow this
/// struct.
pub struct Request<'a> {
    pub method: Method,
    pub headers: Headers,
    pub uri: RequestUri,
    pub gets: MultiMap<String, String>,
    pub vars: HashMap<String, String>,
    pub body: String,
    pub form: MultiMap<String, String>,
    pub session: Session<'a, RequestMarker>,
    pub cookies: Cookies<'a, RequestMarker>,
}

impl<'a> Request<'a> {
    #[doc(hidden)]
    pub fn dummy() ->Request<'a> {
        Request {
            method: Method::Extension("dummy".into()),
            headers: Headers::new(),
            uri: RequestUri::AbsolutePath("dummy".into()),
            gets: MultiMap::new(),
            vars: HashMap::new(),
            body: "".into(),
            form: MultiMap::new(),
            session: Session::new(Rc::new(RefCell::new(None))),
            cookies: Cookies::new(Rc::new(RefCell::new(None))) }
    }

    #[doc(hidden)]
    pub fn new(req: HttpRequest, captures: Option<Captures>, query_string: Option<String>, secret: &[u8]) -> Request<'a> {
        let mut req = req;
        let mut body = String::new();
        info!("{:?}", req.headers);
        match req.read_to_string(&mut body) {
            Ok(_) => (),
            Err(_) => body = String::new()
        }
        let session_jar = Rc::new(RefCell::new(req.headers.get::<header::Cookie>().map(|c| c.to_cookie_jar(secret))));
        let cookie_jar = session_jar.clone();
        Request {
            method: req.method,
            uri: req.uri,
            gets: query_string
                .map(|s| parse_query_string(&s))
                .unwrap_or(MultiMap::new()),
            vars: captures
                .map(|c| c
                     .iter_named()
                     .map(|(k,v)| (k.to_string(), v.unwrap().to_string())).collect())
                .unwrap_or(HashMap::new()),
            form: parse_query_string(&body),
            body: body,
            session: Session::new(session_jar),
            cookies: Cookies::new(cookie_jar),
            headers: req.headers,
        }
    }
}

fn parse_query_string(query_string: &str) -> MultiMap<String, String> {
    let mut map = MultiMap::new();
    for (key, value) in query_string
        .split('&')
        .map(|p| {
            let pair: Vec<_> = p.splitn(2, '=').collect();
            (pair.get(0).map(|s| (*s).into()).unwrap(),
            pair.get(1).map(|s| (*s).into()).unwrap_or(String::new()))
        })
        .filter(|&(ref k,_)| k != "")
    {
        map.insert(key, value);
    }
    map
}

#[test]
fn create_multimap_one_key_value_pair() {
    let m = parse_query_string("key=value");

    assert_eq!(m.len(), 1);
    assert_eq!(m["key"], "value".to_string());
}

#[test]
fn create_multimap_multiple_pairs() {
    let m = parse_query_string("key=value&key2=value2&key3=value3");

    assert_eq!(m.len(), 3);
    assert_eq!(m["key"], "value".to_string());
    assert_eq!(m["key2"], "value2".to_string());
    assert_eq!(m["key3"], "value3".to_string());
}

#[test]
fn create_multimap_one_key_multiple_values() {
    let m = parse_query_string("key=value&key=value2&key=value3");

    assert_eq!(m.len(), 1);
    assert_eq!(m["key"], "value".to_string());
    assert_eq!(m.get_vec("key"), Some(&vec!["value".into(), "value2".into(), "value3".into()]));
}

