use std::collections::HashMap;
use std::io::Read;

use regex::Captures;

use hyper::header::Headers;
use hyper::method::Method;
use hyper::server::request::Request as HttpRequest;
use hyper::uri::RequestUri;

use multimap::MultiMap;

pub struct Request {
    pub method: Method,
    pub headers: Headers,
    pub uri: RequestUri,
    pub gets: MultiMap<String, String>,
    pub vars: HashMap<String, String>,
    pub body: String,
}

impl Request {
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

