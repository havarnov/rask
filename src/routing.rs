use regex::Regex;

use hyper::method::Method;

use Handler;

pub struct Route {
    pub re: Regex,
    pub handler: Box<Handler>,
    pub methods: Vec<Method>,
}

impl Eq for Route {
}

impl PartialEq for Route {
    fn eq(&self, other: &Route) -> bool {
        self.re == other.re
    }
}

impl Route {
    pub fn new<H: 'static + Handler>(re: &str, handler: H) -> Route {
        let route_re = create_routing_rule(re);
        Route {
            re: route_re,
            handler: Box::new(handler),
            methods: Vec::new()}
    }

    pub fn with_methods<H: 'static + Handler>(
        re: &str,
        handler: H,
        methods: &[Method]) -> Route
    {
        let route_re = create_routing_rule(re);
        Route {
            re: route_re,
            handler: Box::new(handler),
            methods: methods.to_vec()}
    }
}

fn create_routing_rule(input: &str) -> Regex {
    let url_exp = input
        .split("/")
        .skip(1)
        .map(|i| create_regex_for_named(&i))
        .fold(String::new(), |a, b| a + &b);

    match Regex::new(&format!(r"^{}$", url_exp)) {
        Ok(re) => re,
        Err(err) => panic!("{}", err)
    }
}

fn create_regex_for_named(s: &str) -> String {
    if s == "" {
        return r"/".to_string();
    }

    let re = Regex::new(r"^\{(?P<named>\w*)\}$|^(?P<part>\w*)$|^(?P<wildcard>\*\*)$").unwrap();
    let caps = re.captures(s).unwrap();

    if let Some(n) = caps.name("named") {
        return format!(r"/(?P<{}>\w*)", n).to_string();
    }

    if let Some(p) = caps.name("part") {
        return format!(r"/{}", p).to_string();
    }

    if let Some(_) = caps.name("wildcard") {
        return format!(r"/(.*)");
    }

    "".to_string()
}
