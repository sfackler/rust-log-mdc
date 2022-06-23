//! A global mapped diagnostic context (MDC) for use with the `log` crate.
//!

use std::borrow::Borrow;
use std::hash::Hash;
use std::{collections::HashMap, sync::RwLock};

lazy_static::lazy_static! {
static ref GLOBAL_MDC: RwLock<HashMap<String, String>> = RwLock::new(HashMap::new());
}

/// Inserts a new entry into the global MDC, returning the old value.
pub fn insert(key: impl Into<String>, value: impl Into<String>) -> Option<String> {
    let mut mdc = GLOBAL_MDC.write().unwrap();
    mdc.insert(key.into(), value.into())
}

/// Extends the global MDC with new entries.
pub fn extend<K, V, I>(entries: I)
where
    K: Into<String>,
    V: Into<String>,
    I: IntoIterator<Item = (K, V)>,
{
    let mut mdc = GLOBAL_MDC.write().unwrap();
    mdc.extend(entries.into_iter().map(|(k, v)| (k.into(), v.into())));
}

/// Retrieves a value from the global MDC.
pub fn get<Q: ?Sized, F, T>(key: &Q, f: F) -> T
where
    String: Borrow<Q>,
    Q: Hash + Eq,
    F: FnOnce(Option<&str>) -> T,
{
    let mdc = GLOBAL_MDC.read().unwrap();
    f(mdc.get(key).map(|v| v.as_str()))
}

/// Removes a value from the global MDC.
pub fn remove<Q: ?Sized>(key: &Q) -> Option<String>
where
    String: Borrow<Q>,
    Q: Hash + Eq,
{
    let mut mdc = GLOBAL_MDC.write().unwrap();
    mdc.remove(key)
}

/// Removes all values from the global MDC.
pub fn clear() {
    let mut mdc = GLOBAL_MDC.write().unwrap();
    mdc.clear();
}

/// Invokes the provided closure for each entry in the global MDC.
pub fn iter<F>(mut f: F)
where
    F: FnMut(&str, &str),
{
    let mdc = GLOBAL_MDC.read().unwrap();
    for (key, value) in mdc.iter() {
        f(key, value)
    }
}
