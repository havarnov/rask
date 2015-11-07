# Rask

A micro web framework for Rust. Based on [hyper](https://github.com/hyperium/hyper).

This project is very much a work in progress.

## Example

````rust
extern crate rask;

use rask::{Rask, StatusCode, Method};
use rask::request::Request;
use rask::response::Response;

fn index(_: &Request, mut res: Response) {
    res.write_body("Hello world!");
    // defaults to Statuscode::Ok
}

fn create(req: &Request, mut res: Response) {
    // do something with req.body
    res.status(StatusCode::Created);
    res.write_body("something created");
}

fn profile(req: &Request, res: &mut Response) {
    let name = req.vars.get("name").unwrap();
    res.write_body(&format!("Hello, {0}", name));
}

fn main() {
    let mut app = Rask::new();

    app.register("/", index); // all methods
    app.register_with_methods("/create", &[Method::Post], create);
    app.register_with_methods("/profile/{name}", &[Method::Get], profile);

    app.serve_static("/static/", "static/");

    app.run("0.0.0.0", 8080);
}
````

The following code is not up to date, e.g. ‘session’ will be handled differently.

````rust
extern crate rask;

use rask::{Rask, Method};
use rask::request::Request;
use rask::response::Response;

fn index(req: &Request, res: &mut Response) {
    res.body = match req.session.get("username") {
        Some(username) => format!("You're logged in as '{0}'.", username),
        None => format!("You're not logged in.")
    };
}

fn login(req: &Request, res: &mut Response) {
    if req.method == Method::Post {
        res.session.set("username", &req.form["username"]);
        res.redirect("/");
    }
    else {
        res.body = "
            <form action=\"\" method=\"POST\">
                <p><input type=\"text\" name=\"username\" />
                <p><input type=\"submit\" value=\"LOGIN\" />
            </form>
            ".into();
    }
}

fn logout(req: &Request, res: &mut Response) {
    res.session.pop("username");
    res.redirect("/");
}

fn main() {
    let mut app = Rask::new();

    app.register("/", index);
    app.register("/login", login);
    app.register("/logout", logout);

    app.run("0.0.0.0", 8080);
}
````

## Planned features

* redirect to handler/url
* session
* cookies
* blueprints (ala flask’s blueprints)

