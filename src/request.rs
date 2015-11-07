use std::collections::HashMap;

use regex::Captures;

use hyper::server::request::Request as HttpRequest;
use hyper::uri::RequestUri;

use url::UrlParser;

use multimap::MultiMap;

pub enum RequestMarker {}

/// The struct that holds information about the incoming Request. The handlers will borrow this
/// struct.
pub struct Request<'a, 'b: 'a> {
    inner: HttpRequest<'a, 'b>,
    pub vars: HashMap<String, String>,
}

impl<'a, 'b> Request<'a, 'b> {
    #[doc(hidden)]
    pub fn new(req: HttpRequest<'a, 'b>, captures: Option<Captures>) -> Request<'a, 'b> {
        Request {
            inner: req,
            vars: captures
                .map(|c| c
                     .iter_named()
                     .map(|(k,v)| (k.to_string(), v.unwrap().to_string())).collect())
                .unwrap_or(HashMap::new()),
        }
    }

    pub fn gets(&self) -> MultiMap<String, String> {
        get_query_string(&self.inner.uri)
            .map(|s| parse_query_string(&s))
            .unwrap_or(MultiMap::new())
    }
}

fn get_query_string(uri: &RequestUri) -> Option<String> {
    match *uri {
        RequestUri::AbsolutePath(ref p) => {
            let parser = UrlParser::new();
            match parser.parse_path(p) {
                Ok((_, query_string, _)) => {
                    query_string
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

//#[test]
//fn create_multimap_one_key_value_pair() {
    //let m = parse_query_string("key=value");

    //assert_eq!(m.len(), 1);
    //assert_eq!(m["key"], "value".to_string());
//}

//#[test]
//fn create_multimap_multiple_pairs() {
    //let m = parse_query_string("key=value&key2=value2&key3=value3");

    //assert_eq!(m.len(), 3);
    //assert_eq!(m["key"], "value".to_string());
    //assert_eq!(m["key2"], "value2".to_string());
    //assert_eq!(m["key3"], "value3".to_string());
//}

//#[test]
//fn create_multimap_one_key_multiple_values() {
    //let m = parse_query_string("key=value&key=value2&key=value3");

    //assert_eq!(m.len(), 1);
    //assert_eq!(m["key"], "value".to_string());
    //assert_eq!(m.get_vec("key"), Some(&vec!["value".into(), "value2".into(), "value3".into()]));
//}

