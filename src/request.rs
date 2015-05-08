use std::collections::HashMap;
use std::io::Read;

use regex::Captures;

use hyper::header::Headers;
use hyper::method::Method;
use hyper::server::request::Request as HttpRequest;
use hyper::uri::RequestUri;

use multimap::MultiMap;

/// The struct that holds information about the incoming Request. The handlers will borrow this
/// struct.
pub struct Request {
    pub method: Method,
    pub headers: Headers,
    pub uri: RequestUri,
    pub gets: MultiMap<String, String>,
    pub vars: HashMap<String, String>,
    pub body: String,
    pub form: MultiMap<String, String>,
}

impl Request {
    #[doc(hidden)]
    pub fn new(req: HttpRequest, captures: Option<Captures>, query_string: Option<String>) -> Request {
        let mut req = req;
        let mut body = String::new();
        match req.read_to_string(&mut body) {
            Ok(_) => (),
            Err(_) => body = String::new()
        }

        Request {
            method: req.method,
            headers: req.headers,
            uri: req.uri,
            gets: match query_string {
                Some(query_string) => parse_query_string(&query_string),
                None => MultiMap::new()
            },
            vars: match captures {
                Some(c) => {
                    let mut v = HashMap::new();
                    for (key, value) in c.iter_named() {
                        v.insert(key.to_string(), value.unwrap().to_string());
                    }
                    v
                },
                None => HashMap::new()
            },
            form: parse_query_string(&body),
            body: body,
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

