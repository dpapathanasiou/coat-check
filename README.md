# About

This [key-value store](https://en.wikipedia.org/wiki/Key%E2%80%93value_database) service is the capstone project for [CS644: Intro to Systems Programming](https://iafisher.com/cs644), [Fall 2025](https://iafisher.com/cs644/fall2025)

The name [coat check](https://dictionary.cambridge.org/example/english/coat-check) is a fanciful real-world analogy to what this software does with data

# Design

Given the requirement to store all key-value pairs in a single file, this is a first attempt at a data format and algorithm that can accommodate variable-sized values:

```
[key] - hashed, fixed length
[size of value] - in bytes, usize
[value] - variable length
...
```

- Fetches work by reading the first n bytes of the key, and if equivalent, returning the corresponding value; otherwise, the size parameter found just after the key is used to skip (`lseek`) ahead to the next key, and the process repeats until either a match is found, or end of file is reached
- Inserts work by confirming the key does not already exist, and if so, adding the `[key][size of value][value]` bytes to the end of the file
- Attempting to write the same key more than once does *not* result in an [upsert](https://en.wikipedia.org/wiki/Merge_%28SQL%29), the original value remains

## Limitations

While the data format meets the basic requirements, including the ability to accommodate a value of any size and type, it also has the following limitations:

- Does not scale easily, since writes are accepted in the order received, and reads do not have the benefit of using an index, etc.
- Keys must hash to the same size, otherwise the read algorithm does not work
- No way to delete or upsert existing key-value pairs
