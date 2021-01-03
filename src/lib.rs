use lazy_static::lazy_static;
use std::{
    borrow::Borrow,
    fmt::Debug,
    iter::{FromIterator, IntoIterator},
    ops::Deref,
    sync::{Arc, RwLock, Weak},
};
use weak_table::WeakHashSet;

lazy_static! {
    static ref STRING_TABLE: RwLock<WeakHashSet<Weak<str>>> = RwLock::new(WeakHashSet::new());
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Debug)]
pub struct ImmutableString(Arc<str>);

impl ImmutableString {
    #[inline]
    pub fn use_count(&self) -> usize {
        Arc::strong_count(&self.0)
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

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for ImmutableString {
    fn as_ref(&self) -> &str {
        self
    }
}

impl Borrow<str> for ImmutableString {
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
}
