# immutable_string - Immutable Shared Strings for Rust
This crate provides a global table for immutable string implemented using Arc<str>.
It avoids duplication of strings in memory when parsing large files with multiple instances of the same sequence of characters.

A global shared WeakHashSet keeps a reference for every live ImmutableString.
A ImmutableString can have multiple owners.
Once every owner drops a ImmutableString, it is lazily removed from the WeakHashSet and dealocated.

The globally shared WeakHashSet is protected by a RwLock which allows multiple concurrent readers but guarantees that any writer has exclusive access.
When instantiating an ImmutableString, the constructor first acquires a reader to check whether the value is already present in the map.
If not, it forgoes the reader lock and attempt to acquire the exclusive writer lock.
Once it has exclusive writer access, it checks again if the string is not present in the map.
Then, it allocates the string and store a weak copy in the hashmap.

The globally shared WeakHashSet may present a performance bottleneck and in the future should be replaced by a distribuited hashmap.

## Changelog
- 0.1.0
  - Initial Release
- 0.1.1
  - Include implementation for Display and Debug trait
