# vu128: Efficient variable-length integers

`vu128` is a variable-length integer encoding, with smaller values being
encoded using fewer bytes. Integer sizes up to 128 bits are supported.
The compression ratio of `vu128` equals or exceeds the widely used [VLQ]
and [LEB128] encodings, and is faster on modern pipelined architectures.

[VLQ]: https://en.wikipedia.org/wiki/Variable-length_quantity
[LEB128]: https://en.wikipedia.org/wiki/LEB128

# Encoding details

Values in the range `[0, 2^7)` are encoded as a single byte with
the same bits as the original value.

Values in the range `[2^7, 2^28)` are encoded as a unary length prefix,
followed by `(length*7)` bits, in little-endian order. This is conceptually
similar to LEB128, but the continuation bits are placed in upper half
of the initial byte. This arrangement is also known as a "prefix varint".

```text
MSB ------------------ LSB

      10101011110011011110  Input value (0xABCDE)
   0101010 1111001 1011110  Zero-padded to a multiple of 7 bits
01010101 11100110 ___11110  Grouped into octets, with 3 continuation bits
01010101 11100110 11011110  Continuation bits `110` added
    0x55     0xE6     0xDE  In hexadecimal

        [0xDE, 0xE6, 0x55]  Encoded output (order is little-endian)
```

Values in the range `[2^28, 2^128)` are encoded as a binary length prefix,
followed by payload bytes, in little-endian order. To differentiate this
format from the format of smaller values, the top 4 bits of the first byte
are set. The length prefix value is the number of payload bytes minus one;
equivalently it is the total length of the encoded value minus two.

```text
MSB ------------------------------------ LSB

               10010001101000101011001111000  Input value (0x12345678)
         00010010 00110100 01010110 01111000  Zero-padded to a multiple of 8 bits
00010010 00110100 01010110 01111000 11110011  Prefix byte is `0xF0 | (4 - 1)`
    0x12     0x34     0x56     0x78     0xF3  In hexadecimal

              [0xF3, 0x78, 0x56, 0x34, 0x12]  Encoded output (order is little-endian)
```

# Handling of over-long encodings

The `vu128` format permits over-long encodings, which encode a value using
a byte sequence that is unnecessarily long:

* Zero-padding beyond that required to reach a multiple of 7 or 8 bits.
* Using a length prefix byte for a value in the range `[0, 2^7)`.
* Using a binary length prefix byte for a value in the range `[0, 2^28)`.

The `encode_*` functions in this module will not generate such over-long
encodings, but the `decode_*` functions will accept them. This is intended
to allow `vu128` values to be placed in a buffer before the value to be
written is known. Applications that require a single canonical encoding for
any given value should perform appropriate checking in their own code.

# Signed integers and floating-point values

Signed integers and IEEE-754 floating-point values may be encoded with
`vu128` by mapping them to unsigned integers. It is recommended that the
mapping functions be chosen so as to minimize the number of zeroes in the
higher-order bits, which enables better compression.

This library includes helper functions that use Protocol Buffer's ["ZigZag"
encoding] for signed integers and reverse-endian layout for floating-point.

["ZigZag" encoding]: https://protobuf.dev/programming-guides/encoding/#signed-ints
