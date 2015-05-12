use std::rc::Rc;
use std::cell::RefCell;

use cookie::{CookieJar, Cookie};

pub struct Session<'a> {
    pub cookie_jar: Rc<RefCell<Option<CookieJar<'a>>>>,
}

impl<'a> Session<'a> {
    pub fn new(cookie_jar: Rc<RefCell<Option<CookieJar<'a>>>>) -> Session<'a> {
        Session {
            cookie_jar: cookie_jar,
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        match *self.cookie_jar.borrow() {
            Some(ref cookie_jar) => cookie_jar.encrypted().find(key).and_then(|c| Some(c.value)),
            None => None
        }
    }

    pub fn set(&mut self, key: &str, value: &str) {
        match *self.cookie_jar.borrow_mut() {
            Some(ref cookie_jar) => cookie_jar.encrypted().add(Cookie::new(key.into(), value.into())),
            None => panic!("cant set on a cookieless..")
        }
    }

    pub fn pop(&mut self, key: &str) {
        match *self.cookie_jar.borrow_mut() {
            Some(ref cookie_jar) => cookie_jar.encrypted().remove(key),
            None => panic!("cant pop on a cookieless..")
        }
    }
}
