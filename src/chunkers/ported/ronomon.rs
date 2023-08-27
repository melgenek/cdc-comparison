// This code is ported from the https://github.com/nlfiedler/fastcdc-rs/tree/0f165fc5fd76e4c9b267bc4fa3a4ec6fcb78fe60
// The MIT License (MIT)
//
// Copyright (c) 2017 Ronomon
// Copyright (c) 2020 Nathan Fiedler
// Copyright (c) 2023 melgenek
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

//! This module implements a variation of the FastCDC algorithm using
//! 31-integers and right shifts instead of left shifts.
//!
//! The explanation below is copied from
//! [ronomon/deduplication](https://github.com/ronomon/deduplication) since this
//! module is little more than a translation of that implementation:
//!
//! > The following optimizations and variations on FastCDC are involved in the
//! > chunking algorithm:
//! > * 31 bit integers to avoid 64 bit integers for the sake of the Javascript
//! >   reference implementation.
//! > * A right shift instead of a left shift to remove the need for an
//! >   additional modulus operator, which would otherwise have been necessary
//! >   to prevent overflow.
//! > * Masks are no longer zero-padded since a right shift is used instead of a
//! >   left shift.
//! > * A more adaptive threshold based on a combination of average and minimum
//! >   chunk size (rather than just average chunk size) to decide the pivot
//! >   point at which to switch masks. A larger minimum chunk size now switches
//! >   from the strict mask to the eager mask earlier.
//! > * Masks use 1 bit of chunk size normalization instead of 2 bits of chunk
//! >   size normalization.
use crate::chunkers::chunker_with_normalization::{new_normalized_chunker_with_center, ChunkerWithMask};
use crate::hashes::right_gearhash::RightGearHashBuilder;
use crate::util::chunk_sizes::ChunkSizes;
use crate::util::logarithm2;
use crate::util::mask_builder::create_simple_mask;
use crate::util::unsigned_integer::UnsignedInteger;
use aes::cipher::{generic_array::GenericArray, KeyIvInit, StreamCipher};
use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;

pub const MINIMUM_MIN: usize = 64;
pub const AVERAGE_MIN: usize = 64;
pub const AVERAGE_MAX: usize = 268435456;

/// https://github.com/ronomon/deduplication/blob/master/binding.cc#L20
#[rustfmt::skip]
const RONOMON_TABLE: [u32; 256] = [
    0x5c95c078, 0x22408989, 0x2d48a214, 0x12842087, 0x530f8afb, 0x474536b9, 0x2963b4f1, 0x44cb738b,
    0x4ea7403d, 0x4d606b6e, 0x074ec5d3, 0x3af39d18, 0x726003ca, 0x37a62a74, 0x51a2f58e, 0x7506358e,
    0x5d4ab128, 0x4d4ae17b, 0x41e85924, 0x470c36f7, 0x4741cbe1, 0x01bb7f30, 0x617c1de3, 0x2b0c3a1f,
    0x50c48f73, 0x21a82d37, 0x6095ace0, 0x419167a0, 0x3caf49b0, 0x40cea62d, 0x66bc1c66, 0x545e1dad,
    0x2bfa77cd, 0x6e85da24, 0x5fb0bdc5, 0x652cfc29, 0x3a0ae1ab, 0x2837e0f3, 0x6387b70e, 0x13176012,
    0x4362c2bb, 0x66d8f4b1, 0x37fce834, 0x2c9cd386, 0x21144296, 0x627268a8, 0x650df537, 0x2805d579,
    0x3b21ebbd, 0x7357ed34, 0x3f58b583, 0x7150ddca, 0x7362225e, 0x620a6070, 0x2c5ef529, 0x7b522466,
    0x768b78c0, 0x4b54e51e, 0x75fa07e5, 0x06a35fc6, 0x30b71024, 0x1c8626e1, 0x296ad578, 0x28d7be2e,
    0x1490a05a, 0x7cee43bd, 0x698b56e3, 0x09dc0126, 0x4ed6df6e, 0x02c1bfc7, 0x2a59ad53, 0x29c0e434,
    0x7d6c5278, 0x507940a7, 0x5ef6ba93, 0x68b6af1e, 0x46537276, 0x611bc766, 0x155c587d, 0x301ba847,
    0x2cc9dda7, 0x0a438e2c, 0x0a69d514, 0x744c72d3, 0x4f326b9b, 0x7ef34286, 0x4a0ef8a7, 0x6ae06ebe,
    0x669c5372, 0x12402dcb, 0x5feae99d, 0x76c7f4a7, 0x6abdb79c, 0x0dfaa038, 0x20e2282c, 0x730ed48b,
    0x069dac2f, 0x168ecf3e, 0x2610e61f, 0x2c512c8e, 0x15fb8c06, 0x5e62bc76, 0x69555135, 0x0adb864c,
    0x4268f914, 0x349ab3aa, 0x20edfdb2, 0x51727981, 0x37b4b3d8, 0x5dd17522, 0x6b2cbfe4, 0x5c47cf9f,
    0x30fa1ccd, 0x23dedb56, 0x13d1f50a, 0x64eddee7, 0x0820b0f7, 0x46e07308, 0x1e2d1dfd, 0x17b06c32,
    0x250036d8, 0x284dbf34, 0x68292ee0, 0x362ec87c, 0x087cb1eb, 0x76b46720, 0x104130db, 0x71966387,
    0x482dc43f, 0x2388ef25, 0x524144e1, 0x44bd834e, 0x448e7da3, 0x3fa6eaf9, 0x3cda215c, 0x3a500cf3,
    0x395cb432, 0x5195129f, 0x43945f87, 0x51862ca4, 0x56ea8ff1, 0x201034dc, 0x4d328ff5, 0x7d73a909,
    0x6234d379, 0x64cfbf9c, 0x36f6589a, 0x0a2ce98a, 0x5fe4d971, 0x03bc15c5, 0x44021d33, 0x16c1932b,
    0x37503614, 0x1acaf69d, 0x3f03b779, 0x49e61a03, 0x1f52d7ea, 0x1c6ddd5c, 0x062218ce, 0x07e7a11a,
    0x1905757a, 0x7ce00a53, 0x49f44f29, 0x4bcc70b5, 0x39feea55, 0x5242cee8, 0x3ce56b85, 0x00b81672,
    0x46beeccc, 0x3ca0ad56, 0x2396cee8, 0x78547f40, 0x6b08089b, 0x66a56751, 0x781e7e46, 0x1e2cf856,
    0x3bc13591, 0x494a4202, 0x520494d7, 0x2d87459a, 0x757555b6, 0x42284cc1, 0x1f478507, 0x75c95dff,
    0x35ff8dd7, 0x4e4757ed, 0x2e11f88c, 0x5e1b5048, 0x420e6699, 0x226b0695, 0x4d1679b4, 0x5a22646f,
    0x161d1131, 0x125c68d9, 0x1313e32e, 0x4aa85724, 0x21dc7ec1, 0x4ffa29fe, 0x72968382, 0x1ca8eef3,
    0x3f3b1c28, 0x39c2fb6c, 0x6d76493f, 0x7a22a62e, 0x789b1c2a, 0x16e0cb53, 0x7deceeeb, 0x0dc7e1c6,
    0x5c75bf3d, 0x52218333, 0x106de4d6, 0x7dc64422, 0x65590ff4, 0x2c02ec30, 0x64a9ac67, 0x59cab2e9,
    0x4a21d2f3, 0x0f616e57, 0x23b54ee8, 0x02730aaa, 0x2f3c634d, 0x7117fc6c, 0x01ac6f05, 0x5a9ed20c,
    0x158c4e2a, 0x42b699f0, 0x0c7c14b3, 0x02bd9641, 0x15ad56fc, 0x1c722f60, 0x7da1af91, 0x23e0dbcb,
    0x0e93e12b, 0x64b2791d, 0x440d2476, 0x588ea8dd, 0x4665a658, 0x7446c418, 0x1877a774, 0x5626407e,
    0x7f63bd46, 0x32d2dbd8, 0x3c790f4a, 0x772b7239, 0x6f8b2826, 0x677ff609, 0x0dc82c11, 0x23ffe354,
    0x2eac53a6, 0x16139e09, 0x0afd0dbc, 0x2a4d4237, 0x56a368c7, 0x234325e4, 0x2dce9187, 0x32e8ea7e
];

type Aes256Ctr64BE = ctr::Ctr64BE<aes::Aes256>;

/// https://github.com/nlfiedler/fastcdc-rs/blob/0f165fc5fd76e4c9b267bc4fa3a4ec6fcb78fe60/examples/table32.rs
fn ronomon64_table() -> [u64; 256] {
    let max_value: u64 = 1u64 << 63;
    let mut table = [0u8; 2048];
    let key = GenericArray::from([0u8; 32]);
    let nonce = GenericArray::from([0u8; 16]);
    let mut cipher = Aes256Ctr64BE::new(&key, &nonce);
    cipher.apply_keystream(&mut table);
    let mut rdr = Cursor::new(&table[..]);
    let mut result = [0u64; 256];
    for i in 0..256 {
        let mut num: u64 = rdr.read_u64::<BigEndian>().unwrap();
        num %= max_value;
        assert!(num < max_value);
        result[i] = num;
    }
    result
}

pub struct RonomonCdc;

impl RonomonCdc {
    pub fn new_original(
        chunk_sizes: ChunkSizes,
        normalization_level: u32,
    ) -> ChunkerWithMask<u32, RightGearHashBuilder<u32>, u32> {
        Self::new(RONOMON_TABLE, chunk_sizes, normalization_level)
    }

    pub fn new_u64(
        chunk_sizes: ChunkSizes,
        normalization_level: u32,
    ) -> ChunkerWithMask<u64, RightGearHashBuilder<u64>, u64> {
        Self::new(ronomon64_table(), chunk_sizes, normalization_level)
    }

    pub fn new<T: UnsignedInteger>(
        table: [T; 256],
        chunk_sizes: ChunkSizes,
        normalization_level: u32,
    ) -> ChunkerWithMask<T, RightGearHashBuilder<T>, T> {
        assert!(chunk_sizes.min_size() >= MINIMUM_MIN);
        assert!(chunk_sizes.avg_size() >= AVERAGE_MIN && chunk_sizes.avg_size() <= AVERAGE_MAX);
        let bits = logarithm2(chunk_sizes.avg_size() as u32);
        assert!(bits - normalization_level >= 5);
        assert!(bits + normalization_level <= 26);
        new_normalized_chunker_with_center(
            chunk_sizes,
            RightGearHashBuilder::new(table),
            Box::new(create_simple_mask),
            normalization_level,
            ronomon_center_finder,
        )
    }
}

/// Integer division that rounds up instead of down.
fn ceil_div(x: usize, y: usize) -> usize {
    (x + y - 1) / y
}

/// Find the middle of the desired chunk size, or what the FastCDC paper refers
/// to as the "normal size".
fn ronomon_center_finder(chunk_sizes: &ChunkSizes, buf_size: usize) -> usize {
    let mut offset: usize = chunk_sizes.min_size() + ceil_div(chunk_sizes.min_size(), 2);
    if offset > chunk_sizes.avg_size() {
        offset = chunk_sizes.avg_size();
    }
    let size: usize = chunk_sizes.avg_size() - offset;
    if size > buf_size {
        buf_size
    } else {
        size
    }
}
