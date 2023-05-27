# Serde Bin

Serde data format for serializing and deserializing in binary.

## The format

Here are the details for how the format operate.

### Numbers

All fixed size numbers, signed or unsigned integers and floats, are written down in there respective big endian representation.

### Bool

Booleans are written with 1 byte, containing either `0` or `1`, any other value is considered invalid.

### Sequences

Sequences are represented as such:

 - The length of the sequence is written as a u64.
 - All elements are then serialized.

#### Maps

Maps are a sequences of key-value pairs, so they are treated as sequences where an element of the sequence is the key-value pair.


#### Unknown length sequences

Sequences with an unknown length are first serialized in an allocated buffer (a `Vec<u8>`), 
the number of items serialized is counted. When the sequence is finished, the length is written and the buffer is flushed. 
This means that for sequences with unknown length, dynamic allocation is needed. 
This is only done if the `alloc` or `std` feature is enabled

### Unit

All unit types (unit, unit struct, unit enum variant) are not serialized, they are treated as ZST.

### Newtype

Newtype types are just wrapper around the contained value, so they are treated as such, only the contained value is serialized.

### Struct/Tuple

Struct and tuples are treated as sequences, the struct field name or the tuple index position are not serialized.

```rust

// the struct
struct Foo {
    bar: u16,
    foobar: i8
}

// is serialized as:
// |    bar     | foobar |
// | u8  |  u8  |   u8   |

```
This mean that the order of serialization and deserialization of the fields matter.

### Option

An option is serialized with a 1 byte tag, either `0` or `1`, any other value is considered invalid.

If the option is empty, no additional byte is written.

If the option contain a value, the value is then serialized after it.

### Enum

For serializing Enums, a tag is first written down as a `u32`. Then the variant is serialized depending on its categorie (unit, newtype, tuple, struct).


## Features
- default: The `std` feature is enabled by default.
- `std`: Enable the use of the std-lib and also enable the `alloc` feature. Writers implementing `io::Write` can be used
- `alloc`: Enable the use of the `alloc` crate, when enabled sequences with unknown size can be serialized.
- `no-unsized-seq`: Disable the serialization of sequences with unknown size when the `alloc` or `std` feature is enabled.
- `test-utils`: Enable the features needed for the crate tests such as `std` and `serde/derive`