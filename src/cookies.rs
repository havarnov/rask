use std::rc::Rc;
use std::cell::RefCell;

use cookie::{CookieJar, Cookie};

pub struct Cookies<'a> {
    cookie_jar: Rc<RefCell<Option<CookieJar<'a>>>>,
}

impl<'a> Cookies<'a> {
    pub fn new(cookie_jar: Rc<RefCell<Option<CookieJar<'a>>>>) -> Cookies<'a> {
        Cookies {
            cookie_jar: cookie_jar,
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        match *self.cookie_jar.borrow() {
            Some(ref cookie_jar) => cookie_jar.find(key).and_then(|c| Some(c.value)),
            None => None
        }
    }

    pub fn set(&mut self, key: &str, value: &str) {
        match *self.cookie_jar.borrow_mut() {
            Some(ref cookie_jar) => cookie_jar.add(Cookie::new(key.into(), value.into())),
            None => panic!("cant set on a cookieless..")
        }
    }
}
