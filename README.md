# Immutable Single Instance Strings for Rust
This crate provides a global table for immutable string implemented using Arc<str>.
It avoids duplication of strings in memory when parsing large files with multiple instances of the same sequence of characters.

A global shared WeakHashSet keeps a reference for every live ImmutableString.
A ImmutableString can have multiple owners.
Once every owner drops a ImmutableString, it is lazily removed from the WeakHashSet and dealocated.
