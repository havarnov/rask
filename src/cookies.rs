use std::rc::Rc;
use std::cell::RefCell;
use std::marker::PhantomData;

use cookie::{CookieJar, Cookie};

use response::ResponseMarker;

pub struct Cookies<'a, T> {
    cookie_jar: Rc<RefCell<Option<CookieJar<'a>>>>,
    _marker : PhantomData<T>,
}

impl<'a, T> Cookies<'a, T> {
    pub fn new(cookie_jar: Rc<RefCell<Option<CookieJar<'a>>>>) -> Cookies<'a, T> {
        Cookies {
            cookie_jar: cookie_jar,
            _marker: PhantomData,
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        match *self.cookie_jar.borrow() {
            Some(ref cookie_jar) => cookie_jar.find(key).and_then(|c| Some(c.value)),
            None => None
        }
    }
}

impl<'a> Cookies<'a, ResponseMarker> {
    pub fn set(&mut self, key: &str, value: &str) {
        match *self.cookie_jar.borrow_mut() {
            Some(ref cookie_jar) => cookie_jar.add(Cookie::new(key.into(), value.into())),
            None => panic!("cant set on a cookieless..")
        }
    }
}
