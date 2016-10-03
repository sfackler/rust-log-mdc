//! A mapped diagnostic context (MDC) for use with the `log` crate.
//!
//! An MDC is a thread local map of strings used to make relevant information
//! from a system available in its log messages. Logging crates such as
//! [log4rs][log4rs] will retrieve values from the MDC for output.
//!
//! For example, a web server may process many requests simultaneously on
//! different threads. Generating an ID for each request and storing it in the
//! MDC makes it easy to partition log messages on a per-request basis.
//!
//! # Examples
//!
//! Forwarding the contents of the MDC to a new thread:
//!
//! ```
//! use std::thread;
//!
//! let mut mdc = vec![];
//! log_mdc::iter(|k, v| mdc.push((k.to_owned(), v.to_owned())));
//!
//! thread::spawn(|| {
//!     log_mdc::extend(mdc);
//! });
//! ```
//!
//! [log4rs]: https://crates.io/crates/log4rs
#![warn(missing_docs)]

use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;

thread_local!(static MDC: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new()));

/// Inserts a new entry into the MDC, returning the old value.
pub fn insert<K, V>(key: K, value: V) -> Option<String>
    where K: Into<String>,
          V: Into<String>
{
    MDC.with(|m| m.borrow_mut().insert(key.into(), value.into()))
}

/// Inserts a new entry into the MDC in a scoped fashion.
///
/// When the returned guard falls out of scope, it will restore the old value corresponding to the
/// key.
///
/// # Examples
///
/// ```
/// let guard = log_mdc::insert_scoped("foo", "a");
/// log_mdc::get("foo", |v| assert_eq!(Some("a"), v));
///
/// drop(guard);
/// log_mdc::get("foo", |v| assert_eq!(None, v));
/// ```
///
/// ```
/// log_mdc::insert("foo", "a");
///
/// let guard = log_mdc::insert_scoped("foo", "b");
/// log_mdc::get("foo", |v| assert_eq!(Some("b"), v));
///
/// drop(guard);
/// log_mdc::get("foo", |v| assert_eq!(Some("a"), v));
/// ```
pub fn insert_scoped<K, V>(key: K, value: V) -> InsertGuard
    where K: Into<String>,
          V: Into<String>
{
    let key = key.into();
    let old_value = insert(&*key, value);

    InsertGuard {
        key: Some(key),
        old_value: old_value,
    }
}

/// Extends the MDC with new entries.
pub fn extend<K, V, I>(entries: I)
    where K: Into<String>,
          V: Into<String>,
          I: IntoIterator<Item = (K, V)>
{
    MDC.with(|m| m.borrow_mut().extend(entries.into_iter().map(|(k, v)| (k.into(), v.into()))));
}

/// Extends the MDC with new entries in a scoped fashion.
///
/// When the returned guard falls out of scope, it will restore the old values corresponding to the
/// keys.
///
/// # Examples
///
/// ```
/// log_mdc::insert("foo", "a");
///
/// let guard = log_mdc::extend_scoped([("foo", "b"), ("fizz", "buzz")].iter().cloned());
/// log_mdc::get("foo", |v| assert_eq!(Some("b"), v));
/// log_mdc::get("fizz", |v| assert_eq!(Some("buzz"), v));
///
/// drop(guard);
/// log_mdc::get("foo", |v| assert_eq!(Some("a"), v));
/// log_mdc::get("fizz", |v| assert_eq!(None, v));
/// ```
pub fn extend_scoped<K, V, I>(entries: I) -> ExtendGuard
    where K: Into<String>,
          V: Into<String>,
          I: IntoIterator<Item = (K, V)>
{
    MDC.with(|m| {
        let mut m = m.borrow_mut();

        let old_entries = entries.into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .map(|(k, v)| {
                let v = m.insert(k.clone(), v);
                (k, v)
            })
            .collect();

        ExtendGuard(old_entries)
    })
}

/// Retrieves a value from the MDC.
pub fn get<Q: ?Sized, F, T>(key: &Q, f: F) -> T
    where String: Borrow<Q>,
          Q: Hash + Eq,
          F: FnOnce(Option<&str>) -> T
{
    MDC.with(|m| f(m.borrow().get(key).map(|v| &**v)))
}

/// Removes a value from the MDC.
pub fn remove<Q: ?Sized>(key: &Q) -> Option<String>
    where String: Borrow<Q>,
          Q: Hash + Eq
{
    MDC.with(|m| m.borrow_mut().remove(key))
}

/// Removes all values from the MDC.
pub fn clear() {
    MDC.with(|m| m.borrow_mut().clear())
}

/// Invokes the provided closure for each entry in the MDC.
pub fn iter<F>(mut f: F)
    where F: FnMut(&str, &str)
{
    MDC.with(|m| {
        for (key, value) in m.borrow().iter() {
            f(key, value)
        }
    })
}

/// A guard object which restores an MDC entry when dropped.
pub struct InsertGuard {
    key: Option<String>,
    old_value: Option<String>,
}

impl Drop for InsertGuard {
    fn drop(&mut self) {
        let key = self.key.take().unwrap();
        match self.old_value.take() {
            Some(value) => insert(key, value),
            None => remove(&key),
        };
    }
}

/// A guard objects which restores MDC entries when dropped.
pub struct ExtendGuard(Vec<(String, Option<String>)>);

impl Drop for ExtendGuard {
    fn drop(&mut self) {
        MDC.with(|m| {
            let mut m = m.borrow_mut();

            for (key, value) in self.0.drain(..) {
                match value {
                    Some(value) => m.insert(key, value),
                    None => m.remove(&key),
                };
            }
        })
    }
}
