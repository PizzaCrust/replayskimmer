# Bincode

This version of bincode has a modified int encoding that has u32 len instead of u64 + additional patches:

- fstring is default string deserialization
- boolean instead from byte is from u32