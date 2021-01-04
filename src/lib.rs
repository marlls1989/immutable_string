//! # immutable_string - Immutable Single Instance Strings for Rust
//! This crate provides a global table for immutable string implemented using Arc<str>.
//! It avoids duplication of strings in memory when parsing large files with multiple instances of the same sequence of characters.
//!
//! A global shared WeakHashSet keeps a reference for every live ImmutableString.
//! A ImmutableString can have multiple owners.
//! Once every owner drops a ImmutableString, it is lazily removed from the WeakHashSet and dealocated.
//!
//! The globally shared WeakHashSet is protected by a RwLock which allows multiple concurrent readers but guarantees that any writer has exclusive access.
//! When instantiating an ImmutableString, the constructor first acquires a reader to check whether the value is already present in the map.
//! If not, it forgoes the reader lock and attempt to acquire the exclusive writer lock.
//! Once it has exclusive writer access, it checks again if the string is not present in the map.
//! Then, it allocates the string and store a weak copy in the hashmap.
//!
//! The globally shared WeakHashSet may present a performance bottleneck and in the future should be replaced by a distribuited hashmap.

use lazy_static::lazy_static;
use std::{
    borrow::Borrow,
    fmt,
    iter::{FromIterator, IntoIterator},
    ops::Deref,
    sync::{Arc, RwLock, Weak},
};
use weak_table::WeakHashSet;

lazy_static! {
    static ref STRING_TABLE: RwLock<WeakHashSet<Weak<str>>> = RwLock::new(WeakHashSet::new());
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct ImmutableString(Arc<str>);

impl ImmutableString {
    /// Returns the number of ImmutableStrings referencing the same data.
    ///
    /// ```
    /// use immutable_string::*;
    ///
    /// let a  = ImmutableString::from("a");
    /// let a0 = ImmutableString::from("a");
    /// let a1 = a.clone();
    /// let b  = ImmutableString::from("b");
    ///
    /// assert_eq!(a.use_count(), 3);
    /// assert_eq!(a0.use_count(), 3);
    /// assert_eq!(a1.use_count(), 3);
    /// assert_eq!(b.use_count(), 1);
    /// ```
    #[inline]
    pub fn use_count(&self) -> usize {
        Arc::strong_count(&self.0)
    }
}

impl fmt::Display for ImmutableString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl fmt::Debug for ImmutableString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T> From<T> for ImmutableString
where
    T: ?Sized + Deref<Target = str> + Into<Arc<str>>,
{
    fn from(s: T) -> Self {
        // Attempt to aquire string without locking the hashmap first
        let str_map = STRING_TABLE.read().expect("Corrupted STRING_TABLE");
        if let Some(val) = str_map.get(&s) {
            ImmutableString(val)
        } else {
            drop(str_map); //Drop read lock to aquire write lock
            let mut str_map = STRING_TABLE.write().expect("Corrupted STRING_TABLE");

            // Double check if string was not inserted after asking for write lock
            if let Some(val) = str_map.get(&s) {
                ImmutableString(val)
            } else {
                let val: Arc<str> = s.into();
                str_map.insert(val.clone());
                ImmutableString(val)
            }
        }
    }
}

impl Deref for ImmutableString {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for ImmutableString {
    #[inline]
    fn as_ref(&self) -> &str {
        self
    }
}

impl Borrow<str> for ImmutableString {
    #[inline]
    fn borrow(&self) -> &str {
        self
    }
}

impl<'a> FromIterator<&'a char> for ImmutableString {
    fn from_iter<I: IntoIterator<Item = &'a char>>(iter: I) -> Self {
        let mut buffer = String::new();
        buffer.extend(iter);
        buffer.shrink_to_fit();
        buffer.into()
    }
}

impl FromIterator<char> for ImmutableString {
    fn from_iter<I: IntoIterator<Item = char>>(iter: I) -> Self {
        let mut buffer = String::new();
        buffer.extend(iter);
        buffer.shrink_to_fit();
        buffer.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_insertion_sharing() {
        let a1 = ImmutableString::from("a");
        let a2 = ImmutableString::from("a");
        assert_eq!(a1.use_count(), 2);
        assert_eq!(a2.use_count(), 2);
        assert_eq!(a1.as_ref(), "a");
        assert_eq!(a2.as_ref(), "a");
    }

    #[test]
    fn iter_collect() {
        use std::iter::repeat;
        let a: ImmutableString = repeat('a').take(5).collect();
        let b: ImmutableString = repeat('a').take(5).collect();
        let c: ImmutableString = repeat('a').take(4).collect();
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_ne!(b, c);
        assert_eq!(a.use_count(), 2);
        assert_eq!(b.use_count(), 2);
        assert_eq!(c.use_count(), 1);
    }

    #[test]
    fn display() {
        let a = ImmutableString::from("a");
        assert_eq!(format!("a: {}", a), "a: a");
    }
}
