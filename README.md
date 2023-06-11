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

 - The number of elements in the sequence is written as a u64 (number of elements is'nt always number of bytes.)
 - All elements are then serialized.

#### Maps

Maps are a sequences of key-value pairs, so they are treated as sequences where an element of the sequence is the key-value pair.


#### Unknown length sequences

Sequences with an unknown length are first serialized in an allocated buffer (a `Vec<u8>`), 
the number of items serialized is counted. When the sequence is finished, the length is written and the buffer is flushed. 
This means that for sequences with unknown length, dynamic allocation is needed. 
This is only done if the `alloc` or `std` feature is enabled

### Strings

Strings are treated as sequences of bytes, so they are encoded as such. The length is in bytes, not in characters count.

There is one case where the format is different, some types are serialized by serde using their `fmt::Display` implementation,
the default behavior for serde is to create a string and feed that to the formatter, and then serialize the string.
But for optimization and avoid allocation, we can feed the writer directly to the formatter, 
but the formatter does'nt feed the writer with all the bytes in one go so the length of the string is unknown at the beggining of it's serialization.
To avoid this problem we can use a solution like a null terminated string, but `NUL` is a valid UTF-8 char, so we can't just set the end byte with `0u8`, but `0xD800` is not valid UTF-8, so we can use that. 
So we set the length to `u64::MAX`, and end sequence with `0xD800`.

```
| length (u64::MAX) |  bytes (UTF-8)  |   end bytes (0xD800)  |
|      u8 * 8       |      u8 * ?     |        u8 * 2         |
```

Such fomat can't be implemented for regular sequences, as the types in the sequences produces any bytes, so there is no end marker that we can be sure it would be unique in the bytes produced.

### Char

Chars are for now converted to a `u32` and serialized as such, might be serialized in UTF-8 in the future.


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


## Module any

The module `serde_bin::any` implement a serializer/deserializer that include the data type in the binary, allowing the use of `serde::de::Deserializer::deserialize_any` and can serialize/deserialize sequences and maps with unknown size without the need of the `alloc` or `std` feature. This can for example allow the deserialization of untagged enums.

### Format change

The any format include a tag describing the next element type, and some changes are brought for either optimizations or more flexibility.

#### Tags

| Tag                   | value  |
|-----------------------|--------|
| None                  | 0      |
| Some                  | 1      |
| BoolFalse             | 2      |
| BoolTrue              | 3      |
| I8                    | 4      |
| I16                   | 5      |
| I32                   | 6      |
| I64                   | 7      |
| U8                    | 8      |
| U16                   | 9      |
| U32                   | 10     |
| U64                   | 11     |
| F32                   | 12     |
| F64                   | 13     |
| Char1                 | 14     |
| Char2                 | 15     |
| Char3                 | 16     |
| Char4                 | 17     |
| String                | 18     |
| UnsizedString         | 19     |
| ByteArray             | 20     |
| Unit                  | 21     |
| UnitStruct            | 22     |
| UnitVariant           | 23     |
| NewTypeStruct         | 24     |
| NewTypeVariant        | 25     |
| Seq                   | 26     |
| UnsizedSeq            | 27     |
| UnsizedSeqEnd         | 28     |
| Tuple                 | 29     |
| TupleStruct           | 30     |
| TupleVariant          | 31     |
| Map                   | 32     |
| UnsizedMap            | 33     |
| Struct                | 34     |
| StructVariant         | 35     |
| I128                  | 36     |
| U128                  | 37     |

#### Option

Options don't insert a `0` or a `1`, the tag describes it: the `Some` tag means an option with the `Some` variant, and the `None` tag means an empty option.

#### Bool

The boolean states is in the tag, `BoolFalse` means a boolean of value `false`, and `BoolTrue` a boolean of value `true`.

#### Char

The tag allows to use UTF8 encoded char and state the encoded size in the tag, so `Char1` means a char with 1 byte, `Char2` 2 bytes, ect... 
Meaning that a char takes beetween 2 and 5 bytes to encode.

#### String

Strings are still encoded the same, plus the inserted tag, but Strings coming from a `fmt::Display` implementation don't need the inserted `u64::MAX`, they now use the `UnsizedString` tag.
They still end with the end marker.

#### Seq

Sequences with an unknown size can now be serialized, the start with the `UnsizedSeq` tag, and end with the `UnsizedSeqEnd` tag. This is now possible due to the fact that each element start with its own tag, so an unique value is now possible.

#### Map

Maps can also be unsized, and are treated as sequence of key-value pair, so they start with `UnsizedMap`, and end with `UnsizedSeqEnd`.

#### Sequence Type

Types that are serialized as sequence such as Tuple, TupleStruct, TupleVariant, Struct and StructVariant now encode the number of elements they contains. This implementation assume their fields count can fit in a `u8`, and encode the length in 1 byte. This is needed to support untagged unions.


## Features
- default: The `std` feature is enabled by default.
- `std`: Enable the use of the std-lib and also enable the `alloc` feature. Writers implementing `io::Write` can be used
- `alloc`: Enable the use of the `alloc` crate, when enabled sequences with unknown size can be serialized.
- `no-unsized-seq`: Disable the serialization of sequences with unknown size when the `alloc` or `std` feature is enabled.
- `test-utils`: Enable the features needed for the crate tests such as `std` and `serde/derive`