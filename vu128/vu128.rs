// Copyright (c) 2024 John Millikin <john@john-millikin.com>
//
// Permission to use, copy, modify, and/or distribute this software for any
// purpose with or without fee is hereby granted.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH
// REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY
// AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,
// INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM
// LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR
// OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR
// PERFORMANCE OF THIS SOFTWARE.
//
// SPDX-License-Identifier: 0BSD

//! # vu128: Efficient variable-length integers
//!
//! `vu128` is a variable-length integer encoding, with smaller values being
//! encoded using fewer bytes. Integer sizes up to 128 bits are supported.
//! The compression ratio of `vu128` equals or exceeds the widely used [VLQ]
//! and [LEB128] encodings, and is faster on modern pipelined architectures.
//!
//! [VLQ]: https://en.wikipedia.org/wiki/Variable-length_quantity
//! [LEB128]: https://en.wikipedia.org/wiki/LEB128
//!
//! # Encoding details
//!
//! Values in the range `[0, 2^7)` are encoded as a single byte with
//! the same bits as the original value.
//!
//! Values in the range `[2^7, 2^28)` are encoded as a unary length prefix,
//! followed by `(length*7)` bits, in little-endian order. This is conceptually
//! similar to LEB128, but the continuation bits are placed in upper half
//! of the initial byte. This arrangement is also known as a "prefix varint".
//!
//! ```text
//! MSB ------------------ LSB
//!
//!       10101011110011011110  Input value (0xABCDE)
//!    0101010 1111001 1011110  Zero-padded to a multiple of 7 bits
//! 01010101 11100110 ___11110  Grouped into octets, with 3 continuation bits
//! 01010101 11100110 11011110  Continuation bits `110` added
//!     0x55     0xE6     0xDE  In hexadecimal
//!
//!         [0xDE, 0xE6, 0x55]  Encoded output (order is little-endian)
//! ```
//!
//! Values in the range `[2^28, 2^128)` are encoded as a binary length prefix,
//! followed by payload bytes, in little-endian order. To differentiate this
//! format from the format of smaller values, the top 4 bits of the first byte
//! are set. The length prefix value is the number of payload bytes minus one;
//! equivalently it is the total length of the encoded value minus two.
//!
//! ```text
//! MSB ------------------------------------ LSB
//!
//!                10010001101000101011001111000  Input value (0x12345678)
//!          00010010 00110100 01010110 01111000  Zero-padded to a multiple of 8 bits
//! 00010010 00110100 01010110 01111000 11110011  Prefix byte is `0xF0 | (4 - 1)`
//!     0x12     0x34     0x56     0x78     0xF3  In hexadecimal
//!
//!               [0xF3, 0x78, 0x56, 0x34, 0x12]  Encoded output (order is little-endian)
//! ```
//!
//! # Handling of over-long encodings
//!
//! The `vu128` format permits over-long encodings, which encode a value using
//! a byte sequence that is unnecessarily long:
//!
//! * Zero-padding beyond that required to reach a multiple of 7 or 8 bits.
//! * Using a length prefix byte for a value in the range `[0, 2^7)`.
//! * Using a binary length prefix byte for a value in the range `[0, 2^28)`.
//!
//! The `encode_*` functions in this module will not generate such over-long
//! encodings, but the `decode_*` functions will accept them. This is intended
//! to allow `vu128` values to be placed in a buffer before the value to be
//! written is known. Applications that require a single canonical encoding for
//! any given value should perform appropriate checking in their own code.
//!
//! # Signed integers and floating-point values
//!
//! Signed integers and IEEE-754 floating-point values may be encoded with
//! `vu128` by mapping them to unsigned integers. It is recommended that the
//! mapping functions be chosen so as to minimize the number of zeroes in the
//! higher-order bits, which enables better compression.
//!
//! This library includes helper functions that use Protocol Buffer's ["ZigZag"
//! encoding] for signed integers and reverse-endian layout for floating-point.
//!
//! ["ZigZag" encoding]: https://protobuf.dev/programming-guides/encoding/#signed-ints

#![no_std]
#![warn(clippy::must_use_candidate)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![warn(missing_docs)]

use core::mem;

/// Returns the encoded length in a `vu128` prefix byte.
///
/// # Examples
///
/// ```
/// let mut buf = [0u8; 5];
/// let encoded_len = vu128::encode_u32(&mut buf, 12345);
/// assert_eq!(vu128::encoded_len(buf[0]), encoded_len);
/// ```
#[must_use]
pub const fn encoded_len(b: u8) -> usize {
	if b < 0b10000000 {
		return 1;
	}
	if b < 0b11000000 {
		return 2;
	}
	if b < 0b11100000 {
		return 3;
	}
	if b < 0b11110000 {
		return 4;
	}
	((b & 0x0F) + 2) as usize
}

/// Encodes a `u32` into a buffer, returning the encoded length.
///
/// The contents of the buffer beyond the returned length are unspecified.
///
/// # Examples
///
/// ```
/// let mut buf = [0u8; 5];
/// let encoded_len = vu128::encode_u32(&mut buf, 12345);
/// assert_eq!(&buf[..encoded_len], &[0xB9, 0xC0]);
/// ```
#[inline]
#[must_use]
pub fn encode_u32(buf: &mut [u8; 5], value: u32) -> usize {
	let mut x = value;
	if x < 0x80 {
		buf[0] = x as u8;
		return 1;
	}
	if x < 0x10000000 {
		if x < 0x00004000 {
			x <<= 2;
			buf[0] = 0x80 | ((x as u8) >> 2);
			buf[1] = (x >> 8) as u8;
			return 2;
		}
		if x < 0x00200000 {
			x <<= 3;
			buf[0] = 0xC0 | ((x as u8) >> 3);
			buf[1] = (x >> 8) as u8;
			buf[2] = (x >> 16) as u8;
			return 3;
		}
		x <<= 4;
		buf[0] = 0xE0 | ((x as u8) >> 4);
		buf[1] = (x >> 8) as u8;
		buf[2] = (x >> 16) as u8;
		buf[3] = (x >> 24) as u8;
		return 4;
	}

	// SAFETY: buf has a const length of `size_of::<u32>() + 1`.
	unsafe {
		ptr_from_mut::<[u8; mem::size_of::<u32>() + 1]>(buf)
			.cast::<u8>()
			.add(1)
			.cast::<u32>()
			.write_unaligned(x.to_le());
	}

	buf[0] = 0xF3;
	5
}

/// Encodes a `u64` into a buffer, returning the encoded length.
///
/// The contents of the buffer beyond the returned length are unspecified.
///
/// # Examples
///
/// ```
/// let mut buf = [0u8; 9];
/// let encoded_len = vu128::encode_u64(&mut buf, 12345);
/// assert_eq!(&buf[..encoded_len], &[0xB9, 0xC0]);
/// ```
#[inline]
#[must_use]
pub fn encode_u64(buf: &mut [u8; 9], value: u64) -> usize {
	let mut x = value;
	if x < 0x80 {
		buf[0] = x as u8;
		return 1;
	}
	if x < 0x10000000 {
		if x < 0x00004000 {
			x <<= 2;
			buf[0] = 0x80 | ((x as u8) >> 2);
			buf[1] = (x >> 8) as u8;
			return 2;
		}
		if x < 0x00200000 {
			x <<= 3;
			buf[0] = 0xC0 | ((x as u8) >> 3);
			buf[1] = (x >> 8) as u8;
			buf[2] = (x >> 16) as u8;
			return 3;
		}
		x <<= 4;
		buf[0] = 0xE0 | ((x as u8) >> 4);
		buf[1] = (x >> 8) as u8;
		buf[2] = (x >> 16) as u8;
		buf[3] = (x >> 24) as u8;
		return 4;
	}

	// SAFETY: buf has a const length of `size_of::<u64>() + 1`.
	unsafe {
		ptr_from_mut::<[u8; mem::size_of::<u64>() + 1]>(buf)
			.cast::<u8>()
			.add(1)
			.cast::<u64>()
			.write_unaligned(x.to_le());
	}

	const LEN_MASK: u8 = 0b111;
	let len = ((x.leading_zeros() >> 3) as u8) ^ LEN_MASK;
	buf[0] = 0xF0 | len;
	(len + 2) as usize
}

/// Encodes a `u128` into a buffer, returning the encoded length.
///
/// The contents of the buffer beyond the returned length are unspecified.
///
/// # Examples
///
/// ```
/// let mut buf = [0u8; 17];
/// let encoded_len = vu128::encode_u128(&mut buf, 12345);
/// assert_eq!(&buf[..encoded_len], &[0xB9, 0xC0]);
/// ```
#[inline]
#[must_use]
pub fn encode_u128(buf: &mut [u8; 17], value: u128) -> usize {
	if value < 0x80 {
		buf[0] = value as u8;
		return 1;
	}
	if value < 0x10000000 {
		// SAFETY: A `[u8; 17]` can be safely truncated to a `[u8; 5]`.
		let buf_u32 = unsafe {
			&mut *(ptr_from_mut::<[u8; 17]>(buf).cast::<[u8; 5]>())
		};
		return encode_u32(buf_u32, value as u32);
	}

	// SAFETY: buf has a const length of `size_of::<u128>() + 1`.
	unsafe {
		ptr_from_mut::<[u8; mem::size_of::<u128>() + 1]>(buf)
			.cast::<u8>()
			.add(1)
			.cast::<u128>()
			.write_unaligned(value.to_le());
	}

	const LEN_MASK: u8 = 0b1111;
	let len = ((value.leading_zeros() >> 3) as u8) ^ LEN_MASK;
	buf[0] = 0xF0 | len;
	(len + 2) as usize
}

/// Decodes a `u32` from a buffer, returning the value and encoded length.
///
/// # Examples
///
/// ```
/// let mut buf = [0u8; 5];
/// let encoded_len = vu128::encode_u32(&mut buf, 123);
/// assert_eq!(vu128::decode_u32(&buf), (123, encoded_len));
/// ```
#[inline]
#[must_use]
pub fn decode_u32(buf: &[u8; 5]) -> (u32, usize) {
	let buf0 = buf[0] as u32;
	if (buf0 & 0x80) == 0 {
		return (buf0, 1);
	}
	if (buf0 & 0b01000000) == 0 {
		let low = (buf0 as u8) & 0x3F;
		let value = ((buf[1] as u32) << 6) | (low as u32);
		return (value, 2);
	}
	if buf0 >= 0xF0 {
		let len = ((buf0 as u8) & 0x0F) + 2;
		let value = ((buf[4] as u32) << 24)
		          | ((buf[3] as u32) << 16)
		          | ((buf[2] as u32) << 8)
		          | (buf[1] as u32);
		return (value, len as usize);
	}
	if (buf0 & 0b00100000) == 0 {
		let low = (buf0 as u8) & 0x1F;
		let value = ((buf[2] as u32) << 13)
		          | ((buf[1] as u32) << 5)
		          | (low as u32);
		return (value, 3);
	}
	let value = ((buf[3] as u32) << 20)
	          | ((buf[2] as u32) << 12)
	          | ((buf[1] as u32) << 4)
	          | (buf0 & 0x0F);
	(value, 4)
}

/// Decodes a `u64` from a buffer, returning the value and encoded length.
///
/// # Examples
///
/// ```
/// let mut buf = [0u8; 9];
/// let encoded_len = vu128::encode_u64(&mut buf, 123);
/// assert_eq!(vu128::decode_u64(&buf), (123, encoded_len));
/// ```
#[inline]
#[must_use]
pub fn decode_u64(buf: &[u8; 9]) -> (u64, usize) {
	let buf0 = buf[0] as u64;
	if (buf0 & 0x80) == 0 {
		return (buf0, 1);
	}
	if buf0 < 0xF0 {
		if (buf0 & 0b01000000) == 0 {
			let low = (buf0 as u8) & 0b00111111;
			let value = ((buf[1] as u32) << 6) | (low as u32);
			return (value as u64, 2);
		}
		if (buf0 & 0b00100000) == 0 {
			let low = (buf0 as u8) & 0b00011111;
			let value = ((buf[2] as u32) << 13)
			          | ((buf[1] as u32) << 5)
			          | (low as u32);
			return (value as u64, 3);
		}
		let value = ((buf[3] as u32) << 20)
		          | ((buf[2] as u32) << 12)
		          | ((buf[1] as u32) << 4)
		          | ((buf0 as u32) & 0b00001111);
		return (value as u64, 4);
	}

	// SAFETY: buf has a const length of `size_of::<u64>() + 1`.
	let value = u64::from_le(unsafe {
		ptr_from_ref::<[u8; mem::size_of::<u64>() + 1]>(buf)
			.cast::<u8>()
			.add(1)
			.cast::<u64>()
			.read_unaligned()
	});

	const LEN_MASK: u8 = 0b111;
	let len = buf[0] & 0x0F;
	let mask = u64::MAX >> (((len & LEN_MASK) ^ LEN_MASK) * 8);
	(value & mask, (len + 2) as usize)
}

/// Decodes a `u128` from a buffer, returning the value and encoded length.
///
/// # Examples
///
/// ```
/// let mut buf = [0u8; 17];
/// let encoded_len = vu128::encode_u128(&mut buf, 123);
/// assert_eq!(vu128::decode_u128(&buf), (123, encoded_len));
/// ```
#[inline]
#[must_use]
pub fn decode_u128(buf: &[u8; 17]) -> (u128, usize) {
	if (buf[0] & 0x80) == 0 {
		return (buf[0] as u128, 1);
	}
	if buf[0] < 0xF0 {
		// SAFETY: A `[u8; 17]` can be safely truncated to a `[u8; 5]`.
		let buf_u32 = unsafe {
			&*(ptr_from_ref::<[u8; 17]>(buf).cast::<[u8; 5]>())
		};
		let (value, len) = decode_u32(buf_u32);
		return (value as u128, len);
	}

	// SAFETY: buf has a const length of `size_of::<u128>() + 1`.
	let value = u128::from_le(unsafe {
		ptr_from_ref::<[u8; mem::size_of::<u128>() + 1]>(buf)
			.cast::<u8>()
			.add(1)
			.cast::<u128>()
			.read_unaligned()
	});
	const LEN_MASK: u8 = 0b1111;
	let len = buf[0] & 0x0F;
	let mask = u128::MAX >> (((len & LEN_MASK) ^ LEN_MASK) * 8);
	(value & mask, (len + 2) as usize)
}

macro_rules! encode_iNN {
	($(#[$docs:meta])* $name:ident ( $it:ident, $ut:ident, $encode_fn:ident ) ) => {
		$(#[$docs])*
		#[inline]
		#[must_use]
		pub fn $name(buf: &mut [u8; mem::size_of::<$ut>()+1], value: $it) -> usize {
			const ZIGZAG_SHIFT: u8 = ($ut::BITS as u8) - 1;
			let zigzag = ((value >> ZIGZAG_SHIFT) as $ut) ^ ((value << 1) as $ut);
			$encode_fn(buf, zigzag)
		}
	};
}

macro_rules! decode_iNN {
	($(#[$docs:meta])* $name:ident ( $it:ident, $ut:ident, $decode_fn:ident ) ) => {
		$(#[$docs])*
		#[inline]
		#[must_use]
		pub fn $name(buf: &[u8; mem::size_of::<$ut>()+1]) -> ($it, usize) {
			let (zz, len) = $decode_fn(buf);
			let value = ((zz >> 1) as $it) ^ (-((zz & 1) as $it));
			(value, len)
		}
	};
}

encode_iNN! {
	/// Encodes an `i32` into a buffer, returning the encoded length.
	///
	/// The contents of the buffer beyond the returned length are unspecified.
	///
	/// # Examples
	///
	/// ```
	/// let mut buf = [0u8; 5];
	/// let encoded_len = vu128::encode_i32(&mut buf, 123);
	/// assert_eq!(&buf[..encoded_len], &[0xB6, 0x03]);
	/// ```
	encode_i32(i32, u32, encode_u32)
}

encode_iNN! {
	/// Encodes an `i64` into a buffer, returning the encoded length.
	///
	/// The contents of the buffer beyond the returned length are unspecified.
	///
	/// # Examples
	///
	/// ```
	/// let mut buf = [0u8; 9];
	/// let encoded_len = vu128::encode_i64(&mut buf, 123);
	/// assert_eq!(&buf[..encoded_len], &[0xB6, 0x03]);
	/// ```
	encode_i64(i64, u64, encode_u64)
}

encode_iNN! {
	/// Encodes an `i128` into a buffer, returning the encoded length.
	///
	/// The contents of the buffer beyond the returned length are unspecified.
	///
	/// # Examples
	///
	/// ```
	/// let mut buf = [0u8; 17];
	/// let encoded_len = vu128::encode_i128(&mut buf, 123);
	/// assert_eq!(&buf[..encoded_len], &[0xB6, 0x03]);
	/// ```
	encode_i128(i128, u128, encode_u128)
}

decode_iNN! {
	/// Decodes an `i32` from a buffer, returning the value and encoded length.
	///
	/// # Examples
	///
	/// ```
	/// let mut buf = [0u8; 5];
	/// let encoded_len = vu128::encode_i32(&mut buf, 123);
	/// assert_eq!(vu128::decode_i32(&buf), (123, encoded_len));
	/// ```
	decode_i32(i32, u32, decode_u32)
}

decode_iNN! {
	/// Decodes an `i64` from a buffer, returning the value and encoded length.
	///
	/// # Examples
	///
	/// ```
	/// let mut buf = [0u8; 9];
	/// let encoded_len = vu128::encode_i64(&mut buf, 123);
	/// assert_eq!(vu128::decode_i64(&buf), (123, encoded_len));
	/// ```
	decode_i64(i64, u64, decode_u64)
}

decode_iNN! {
	/// Decodes an `i128` from a buffer, returning the value and encoded length.
	///
	/// # Examples
	///
	/// ```
	/// let mut buf = [0u8; 17];
	/// let encoded_len = vu128::encode_i128(&mut buf, 123);
	/// assert_eq!(vu128::decode_i128(&buf), (123, encoded_len));
	/// ```
	decode_i128(i128, u128, decode_u128)
}

/// Encodes an `f32` into a buffer, returning the encoded length.
///
/// The contents of the buffer beyond the returned length are unspecified.
///
/// # Examples
///
/// ```
/// let mut buf = [0u8; 5];
/// let encoded_len = vu128::encode_f32(&mut buf, 2.5);
/// assert_eq!(&buf[..encoded_len], &[0x80, 0x81]);
/// ```
#[inline]
#[must_use]
pub fn encode_f32(buf: &mut [u8; 5], value: f32) -> usize {
	encode_u32(buf, value.to_bits().swap_bytes())
}

/// Encodes an `f64` into a buffer, returning the encoded length.
///
/// The contents of the buffer beyond the returned length are unspecified.
///
/// # Examples
///
/// ```
/// let mut buf = [0u8; 9];
/// let encoded_len = vu128::encode_f64(&mut buf, 2.5);
/// assert_eq!(&buf[..encoded_len], &[0x80, 0x11]);
/// ```
#[inline]
#[must_use]
pub fn encode_f64(buf: &mut [u8; 9], value: f64) -> usize {
	encode_u64(buf, value.to_bits().swap_bytes())
}

/// Decodes an `f32` from a buffer, returning the value and encoded length.
///
/// # Examples
///
/// ```
/// let mut buf = [0u8; 5];
/// let encoded_len = vu128::encode_f32(&mut buf, 2.5);
/// assert_eq!(vu128::decode_f32(&buf), (2.5, encoded_len));
/// ```
#[inline]
#[must_use]
pub fn decode_f32(buf: &[u8; 5]) -> (f32, usize) {
	let (swapped, len) = decode_u32(buf);
	(f32::from_bits(swapped.swap_bytes()), len)
}

/// Decodes an `f64` from a buffer, returning the value and encoded length.
///
/// # Examples
///
/// ```
/// let mut buf = [0u8; 9];
/// let encoded_len = vu128::encode_f64(&mut buf, 2.5);
/// assert_eq!(vu128::decode_f64(&buf), (2.5, encoded_len));
/// ```
#[inline]
#[must_use]
pub fn decode_f64(buf: &[u8; 9]) -> (f64, usize) {
	let (swapped, len) = decode_u64(buf);
	(f64::from_bits(swapped.swap_bytes()), len)
}

#[inline(always)]
const fn ptr_from_ref<T: ?Sized>(r: &T) -> *const T {
	r
}

#[inline(always)]
fn ptr_from_mut<T: ?Sized>(r: &mut T) -> *mut T {
	r
}
