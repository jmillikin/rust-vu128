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

use core::fmt;

const U32_TEST_CASES: &[(u32, &[u8])] = &[
	(0xABCDE, &[0xDE, 0xE6, 0x55]),
	(0x00000000, &[0x00000000]),
	(0x0000007F, &[0x0000007F]),
	(0x00000080, &[0b10000000, 0x02]),
	(0x00003FFF, &[0b10111111, 0xFF]),
	(0x00004000, &[0b11000000, 0x00, 0x02]),
	(0x001FFFFF, &[0b11011111, 0xFF, 0xFF]),
	(0x00200000, &[0b11100000, 0x00, 0x00, 0x02]),
	(0x0FFFFFFF, &[0b11101111, 0xFF, 0xFF, 0xFF]),
	(0x10000000, &[0b11110011, 0x00, 0x00, 0x00, 0x10]),
	(0xFFFFFFFF, &[0b11110011, 0xFF, 0xFF, 0xFF, 0xFF]),
];

const U64_TEST_CASES: &[(u64, &[u8])] = &[
	(0x00000000_00000000, &[0x00000000]),
	(0x00000000_0000007F, &[0x0000007F]),
	(0x00000000_00000080, &[0b10000000, 0x02]),
	(0x00000000_00003FFF, &[0b10111111, 0xFF]),
	(0x00000000_00004000, &[0b11000000, 0x00, 0x02]),
	(0x00000000_001FFFFF, &[0b11011111, 0xFF, 0xFF]),
	(0x00000000_00200000, &[0b11100000, 0x00, 0x00, 0x02]),
	(0x00000000_0FFFFFFF, &[0b11101111, 0xFF, 0xFF, 0xFF]),
	(0x00000000_10000000, &[0b11110011, 0x00, 0x00, 0x00, 0x10]),
	(0x00000000_FFFFFFFF, &[0b11110011, 0xFF, 0xFF, 0xFF, 0xFF]),
	(0x00000001_FFFFFFFF, &[0b11110100, 0xFF, 0xFF, 0xFF, 0xFF, 0x01]),
	(0x000000FF_FFFFFFFF, &[0b11110100, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]),
	(0x000001FF_FFFFFFFF, &[0b11110101, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01]),
	(0x0000FFFF_FFFFFFFF, &[0b11110101, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]),
	(0x0001FFFF_FFFFFFFF, &[0b11110110, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01]),
	(0x00FFFFFF_FFFFFFFF, &[0b11110110, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]),
	(0x01FFFFFF_FFFFFFFF, &[0b11110111, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01]),
	(0xFFFFFFFF_FFFFFFFF, &[0b11110111, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]),
];

const I32_TEST_CASES: &[(i32, &[u8])] = &[
	(0x00000000, &[0x00]),
	(0x0000007F, &[0xBE, 0x03]),
	(0x00000080, &[0x80, 0x04]),
	(0x000000FF, &[0xBE, 0x07]),
	(0x000001FF, &[0xBE, 0x0F]),
	(0x0000FFFF, &[0xDE, 0xFF, 0x0F]),
	(0x0001FFFF, &[0xDE, 0xFF, 0x1F]),
	(0x00FFFFFF, &[0xEE, 0xFF, 0xFF, 0x1F]),
	(0x01FFFFFF, &[0xEE, 0xFF, 0xFF, 0x3F]),

	(0xFFFFFFFFu32 as i32, &[0x01]),
	(0xFFFFFF00u32 as i32, &[0xBF, 0x07]),
	(0xFFFF0000u32 as i32, &[0xDF, 0xFF, 0x0F]),
	(0xFF000000u32 as i32, &[0xEF, 0xFF, 0xFF, 0x1F]),
	(0x80000000u32 as i32, &[0xF3, 0xFF, 0xFF, 0xFF, 0xFF]),
];

const I64_TEST_CASES: &[(i64, &[u8])] = &[
	(0x00000000_00000000, &[0x00]),
	(0x00000000_0000007F, &[0xBE, 0x03]),
	(0x00000000_00000080, &[0x80, 0x04]),
	(0x00000000_000000FF, &[0xBE, 0x07]),
	(0x00000000_000001FF, &[0xBE, 0x0F]),
	(0x00000000_0000FFFF, &[0xDE, 0xFF, 0x0F]),
	(0x00000000_0001FFFF, &[0xDE, 0xFF, 0x1F]),
	(0x00000000_00FFFFFF, &[0xEE, 0xFF, 0xFF, 0x1F]),
	(0x00000000_01FFFFFF, &[0xEE, 0xFF, 0xFF, 0x3F]),
	(0x00000000_FFFFFFFF, &[0xF4, 0xFE, 0xFF, 0xFF, 0xFF, 0x01]),
	(0x00000001_FFFFFFFF, &[0xF4, 0xFE, 0xFF, 0xFF, 0xFF, 0x03]),
	(0x000000FF_FFFFFFFF, &[0xF5, 0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0x01]),
	(0x000001FF_FFFFFFFF, &[0xF5, 0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0x03]),
	(0x0000FFFF_FFFFFFFF, &[0xF6, 0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01]),
	(0x0001FFFF_FFFFFFFF, &[0xF6, 0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x03]),
	(0x00FFFFFF_FFFFFFFF, &[0xF7, 0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01]),
	(0x01FFFFFF_FFFFFFFF, &[0xF7, 0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x03]),

	(0xFFFFFFFF_FFFFFFFFu64 as i64, &[0x01]),
	(0xFFFFFFFF_FFFFFF00u64 as i64, &[0xBF, 0x07]),
	(0xFFFFFFFF_FFFF0000u64 as i64, &[0xDF, 0xFF, 0x0F]),
	(0xFFFFFFFF_FF000000u64 as i64, &[0xEF, 0xFF, 0xFF, 0x1F]),
	(0xFFFFFFFF_00000000u64 as i64, &[0xF4, 0xFF, 0xFF, 0xFF, 0xFF, 0x01]),
	(0xFFFFFF00_00000000u64 as i64, &[0xF5, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01]),
	(0xFFFF0000_00000000u64 as i64, &[0xF6, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01]),
	(0xFF000000_00000000u64 as i64, &[0xF7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01]),
	(0x80000000_00000000u64 as i64, &[0xF7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]),
];

const F32_TEST_CASES: &[(f32, &[u8])] = &[
	( 0.0, &[0x00]),
	(-0.0, &[0x80, 0x02]),
];

const F64_TEST_CASES: &[(f64, &[u8])] = &[
	( 0.0, &[0x00]),
	(-0.0, &[0x80, 0x02]),
];

macro_rules! assert_expected {
	($test_name:ident, $value:expr, $expect:expr, $got:expr) => {
		if $got != $expect {
			panic!(
				"{}({}): expected {}, got {}",
				stringify!($test_name),
				($value).arg_fmt(),
				($expect).arg_fmt(),
				($got).arg_fmt(),
			);
		}
	};
}

#[test]
fn test_encode_u32() {
	for (value, expect) in U32_TEST_CASES {
		let mut buf = [0u8; 5];
		let len = vu128::encode_u32(&mut buf, *value);
		assert_expected!(encode_u32, *value, *expect, &buf[..len]);
	}
}

#[test]
fn test_decode_u32() {
	for (expect, encoded_value) in U32_TEST_CASES {
		let mut buf = [0u8; 5];
		(&mut buf[0..encoded_value.len()]).copy_from_slice(encoded_value);
		let got = vu128::decode_u32(&buf);
		let expect = (*expect, encoded_value.len());
		assert_expected!(decode_u32, encoded_value, expect, got);
	}
}

#[test]
fn test_encode_u64() {
	for (value, expect) in U64_TEST_CASES {
		let mut buf = [0u8; 9];
		let len = vu128::encode_u64(&mut buf, *value);
		assert_expected!(encode_u64, *value, *expect, &buf[..len]);
	}
}

#[test]
fn test_decode_u64() {
	for (expect, encoded_value) in U64_TEST_CASES {
		let mut buf = [0u8; 9];
		(&mut buf[0..encoded_value.len()]).copy_from_slice(encoded_value);
		let got = vu128::decode_u64(&buf);
		let expect = (*expect, encoded_value.len());
		assert_expected!(decode_u64, encoded_value, expect, got);
	}
}

#[test]
fn test_encode_u128() {
	for (value, expect) in U32_TEST_CASES {
		let value = *value as u128;
		let mut buf = [0u8; 17];
		let len = vu128::encode_u128(&mut buf, value);
		assert_expected!(encode_u128, value, *expect, &buf[..len]);
	}
	for (value, expect) in U64_TEST_CASES {
		let value = *value as u128;
		let mut buf = [0u8; 17];
		let len = vu128::encode_u128(&mut buf, value);
		assert_expected!(encode_u128, value, *expect, &buf[..len]);
	}
}

#[test]
fn test_decode_u128() {
	for (expect, encoded_value) in U32_TEST_CASES {
		let mut buf = [0u8; 17];
		(&mut buf[0..encoded_value.len()]).copy_from_slice(encoded_value);
		let got = vu128::decode_u128(&buf);
		let expect = (*expect as u128, encoded_value.len());
		assert_expected!(decode_u128, encoded_value, expect, got);
	}
	for (expect, encoded_value) in U64_TEST_CASES {
		let mut buf = [0u8; 17];
		(&mut buf[0..encoded_value.len()]).copy_from_slice(encoded_value);
		let got = vu128::decode_u128(&buf);
		let expect = (*expect as u128, encoded_value.len());
		assert_expected!(decode_u128, encoded_value, expect, got);
	}
}

#[test]
fn test_encode_i32() {
	for (value, expect) in I32_TEST_CASES {
		let mut buf = [0u8; 5];
		let len = vu128::encode_i32(&mut buf, *value);
		assert_expected!(encode_i32, *value, *expect, &buf[..len]);
	}
}

#[test]
fn test_decode_i32() {
	for (expect, encoded_value) in I32_TEST_CASES {
		let mut buf = [0u8; 5];
		(&mut buf[0..encoded_value.len()]).copy_from_slice(encoded_value);
		let got = vu128::decode_i32(&buf);
		let expect = (*expect, encoded_value.len());
		assert_expected!(decode_i32, encoded_value, expect, got);
	}
}

#[test]
fn test_encode_i64() {
	for (value, expect) in I64_TEST_CASES {
		let mut buf = [0u8; 9];
		let len = vu128::encode_i64(&mut buf, *value);
		assert_expected!(encode_i64, *value, *expect, &buf[..len]);
	}
}

#[test]
fn test_decode_i64() {
	for (expect, encoded_value) in I64_TEST_CASES {
		let mut buf = [0u8; 9];
		(&mut buf[0..encoded_value.len()]).copy_from_slice(encoded_value);
		let got = vu128::decode_i64(&buf);
		let expect = (*expect, encoded_value.len());
		assert_expected!(decode_i64, encoded_value, expect, got);
	}
}

#[test]
fn test_encode_i128() {
	for (value, expect) in I32_TEST_CASES {
		let value = *value as i128;
		let mut buf = [0u8; 17];
		let len = vu128::encode_i128(&mut buf, value);
		assert_expected!(encode_i128, value, *expect, &buf[..len]);
	}
	for (value, expect) in I64_TEST_CASES {
		let value = *value as i128;
		let mut buf = [0u8; 17];
		let len = vu128::encode_i128(&mut buf, value);
		assert_expected!(encode_i128, value, *expect, &buf[..len]);
	}
}

#[test]
fn test_decode_i128() {
	for (expect, encoded_value) in I32_TEST_CASES {
		let mut buf = [0u8; 17];
		(&mut buf[0..encoded_value.len()]).copy_from_slice(encoded_value);
		let got = vu128::decode_i128(&buf);
		let expect = (*expect as i128, encoded_value.len());
		assert_expected!(decode_i128, encoded_value, expect, got);
	}
	for (expect, encoded_value) in I64_TEST_CASES {
		let mut buf = [0u8; 17];
		(&mut buf[0..encoded_value.len()]).copy_from_slice(encoded_value);
		let got = vu128::decode_i128(&buf);
		let expect = (*expect as i128, encoded_value.len());
		assert_expected!(decode_i128, encoded_value, expect, got);
	}
}

#[test]
fn test_encode_f32() {
	for (value, expect) in F32_TEST_CASES {
		let mut buf = [0u8; 5];
		let len = vu128::encode_f32(&mut buf, *value);
		assert_expected!(encode_f32, *value, *expect, &buf[..len]);
	}
}

#[test]
fn test_decode_f32() {
	for (expect, encoded_value) in F32_TEST_CASES {
		let mut buf = [0u8; 5];
		(&mut buf[0..encoded_value.len()]).copy_from_slice(encoded_value);
		let got = vu128::decode_f32(&buf);
		let expect = (*expect, encoded_value.len());
		assert_expected!(decode_f32, encoded_value, expect, got);
	}
}

#[test]
fn test_encode_f64() {
	for (value, expect) in F64_TEST_CASES {
		let mut buf = [0u8; 9];
		let len = vu128::encode_f64(&mut buf, *value);
		assert_expected!(encode_f64, *value, *expect, &buf[..len]);
	}
}

#[test]
fn test_decode_f64() {
	for (expect, encoded_value) in F64_TEST_CASES {
		let mut buf = [0u8; 9];
		(&mut buf[0..encoded_value.len()]).copy_from_slice(encoded_value);
		let got = vu128::decode_f64(&buf);
		let expect = (*expect, encoded_value.len());
		assert_expected!(decode_f64, encoded_value, expect, got);
	}
}

trait ArgFmt: fmt::Debug {
	fn arg_fmt(&self) -> String {
		format!("{:?}", self)
	}
}

impl ArgFmt for u32 {
	fn arg_fmt(&self) -> String {
		format!("0x{:08X?}", self)
	}
}

impl ArgFmt for i32 {}

impl ArgFmt for u64 {
	fn arg_fmt(&self) -> String {
		format!("0x{:016X?}", self)
	}
}
impl ArgFmt for i64 {}

impl ArgFmt for u128 {
	fn arg_fmt(&self) -> String {
		format!("0x{:032X?}", self)
	}
}
impl ArgFmt for i128 {}

impl ArgFmt for &[u8] {
	fn arg_fmt(&self) -> String {
		format!("{:?}", HexArray(self))
	}
}

impl ArgFmt for f32 {}

impl ArgFmt for f64 {}

impl ArgFmt for usize {}

impl<T1: ArgFmt, T2: ArgFmt> ArgFmt for (T1, T2) {
	fn arg_fmt(&self) -> String {
		format!("({}, {})", self.0.arg_fmt(), self.1.arg_fmt())
	}
}

struct HexArray<'a>(&'a [u8]);

impl fmt::Debug for HexArray<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		write!(fmt, "[")?;
		let mut comma = "";
		for byte in self.0 {
			write!(fmt, "{}0x{:02X}", comma, byte)?;
			comma = ", ";
		}
		write!(fmt, "]")
	}
}
