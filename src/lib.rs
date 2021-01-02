use lazy_static::*;
use std::{
    fmt::Debug,
    ops::Deref,
    borrow::Borrow,
    sync::{Arc, RwLock, Weak},
};
use weak_table::WeakHashSet;

lazy_static! {
    static ref STRING_TABLE: RwLock<WeakHashSet<Weak<str>>> = RwLock::new(WeakHashSet::new());
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Debug)]
pub struct ImmutableString(Arc<str>);

impl<T> From<T> for ImmutableString where
T: ?Sized + Deref<Target=str> + Into<Arc<str>> {
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

impl ImmutableString {
    #[inline]
    pub fn use_count(&self) -> usize {
        Arc::strong_count(&self.0)
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
}
