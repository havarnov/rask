# Rask

A simple web framework for Rust. Based on [hyper](https://github.com/hyperium/hyper).

This project is very much a work in progress.

## Example

````rust
extern crate rask;

use rask::{Rask, Request, Response, StatusCode, Method};

fn index(req: &Request, res: &mut Response) {
    res.body = "Hello world!".into();
    // defaults to Statuscode::Ok
}

fn create(req: &Request, res: &mut Response) {
	// do something with req.body
    res.body = "something created".into();
    res.status = StatusCode::Created;
}

fn profile(req: &Request, res: &mut Response) {
    let name = req.vars("name").unwrap();
    res.body = format!("Hello, {0}", name);
}

fn main() {
    let mut app = Rask::new();

    app.register("/", index);
    app.register_with_methods("/create", &[Method::Post], create);
    app.register_with_methods("/profile/{name}", &[Method::Get], profile);

    app.run("0.0.0.0", 8080);
}
````
