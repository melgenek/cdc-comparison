// This code is ported from the https://github.com/borgbackup/borg/blob/1c8da8f98ad118e363d1f90c805e26d5f284d077/src/borg/_chunker.c#L83
//
// Copyright (C) 2023 melgenek
// Copyright (C) 2015-2023 The Borg Collective (see AUTHORS file https://github.com/borgbackup/borg/blob/1c8da8f98ad118e363d1f90c805e26d5f284d077/AUTHORS)
// Copyright (C) 2010-2014 Jonas Borgström <jonas@borgstrom.se>
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
//
//  1. Redistributions of source code must retain the above copyright
//     notice, this list of conditions and the following disclaimer.
//  2. Redistributions in binary form must reproduce the above copyright
//     notice, this list of conditions and the following disclaimer in
//     the documentation and/or other materials provided with the
//     distribution.
//  3. The name of the author may not be used to endorse or promote
//     products derived from this software without specific prior
//     written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

//! /* Cyclic polynomial / buzhash
//!
//! http://www.serve.net/buz/Notes.1st.year/HTML/C6/rand.012.html (by "BUZ", the inventor)
//! http://www.dcs.gla.ac.uk/~hamer/cakes-talk.pdf (see buzhash slide)
//!
//! Some properties of buzhash / of this implementation:
//!
//! (1) the hash is designed for inputs <= 32 bytes, but the chunker uses it on a 4095 byte window;
//!     any repeating bytes at distance 32 within those 4095 bytes can cause cancellation within
//!     the hash function, e.g. in "X <any 31 bytes> X", the last X would cancel out the influence
//!     of the first X on the hash value.
//!
//! (2) the hash table is supposed to have (according to the BUZ) exactly a 50% distribution of
//!     0/1 bit values per position, but the hard coded table below doesn't fit that property.
//!
//! (3) if you would use a window size divisible by 64, the seed would cancel itself out completely.
//!     this is why we use a window size of 4095 bytes.
//!
//! Note: the 'seed' is used in the original implementation.
use crate::chunkers::chunker_with_normalization::{new_normalized_chunker, ChunkerWithMask};
use crate::hashes::buzhash::BuzHashBuilder;
use crate::util::chunk_sizes::ChunkSizes;
use crate::util::mask_builder::create_simple_mask;

#[rustfmt::skip]
const BORG_TABLE: [u32; 256] = [
    0xe7f831ec, 0xf4026465, 0xafb50cae, 0x6d553c7a, 0xd639efe3, 0x19a7b895, 0x9aba5b21, 0x5417d6d4,
    0x35fd2b84, 0xd1f6a159, 0x3f8e323f, 0xb419551c, 0xf444cebf, 0x21dc3b80, 0xde8d1e36, 0x84a32436,
    0xbeb35a9d, 0xa36f24aa, 0xa4e60186, 0x98d18ffe, 0x3f042f9e, 0xdb228bcd, 0x096474b7, 0x5c20c2f7,
    0xf9eec872, 0xe8625275, 0xb9d38f80, 0xd48eb716, 0x22a950b4, 0x3cbaaeaa, 0xc37cddd3, 0x8fea6f6a,
    0x1d55d526, 0x7fd6d3b3, 0xdaa072ee, 0x4345ac40, 0xa077c642, 0x8f2bd45b, 0x28509110, 0x55557613,
    0xffc17311, 0xd961ffef, 0xe532c287, 0xaab95937, 0x46d38365, 0xb065c703, 0xf2d91d0f, 0x92cd4bb0,
    0x4007c712, 0xf35509dd, 0x505b2f69, 0x557ead81, 0x310f4563, 0xbddc5be8, 0x9760f38c, 0x701e0205,
    0x00157244, 0x14912826, 0xdc4ca32b, 0x67b196de, 0x5db292e8, 0x8c1b406b, 0x01f34075, 0xfa2520f7,
    0x73bc37ab, 0x1e18bc30, 0xfe2c6cb3, 0x20c522d0, 0x5639e3db, 0x942bda35, 0x899af9d1, 0xced44035,
    0x98cc025b, 0x255f5771, 0x70fefa24, 0xe928fa4d, 0x2c030405, 0xb9325590, 0x20cb63bd, 0xa166305d,
    0x80e52c0a, 0xa8fafe2f, 0x1ad13f7d, 0xcfaf3685, 0x6c83a199, 0x7d26718a, 0xde5dfcd9, 0x79cf7355,
    0x8979d7fb, 0xebf8c55e, 0xebe408e4, 0xcd2affba, 0xe483be6e, 0xe239d6de, 0x5dc1e9e0, 0x0473931f,
    0x851b097c, 0xac5db249, 0x09c0f9f2, 0xd8d2f134, 0xe6f38e41, 0xb1c71bf1, 0x52b6e4db, 0x07224424,
    0x6cf73e85, 0x4f25d89c, 0x782a7d74, 0x10a68dcd, 0x3a868189, 0xd570d2dc, 0x69630745, 0x9542ed86,
    0x331cd6b2, 0xa84b5b28, 0x07879c9d, 0x38372f64, 0x7185db11, 0x25ba7c83, 0x01061523, 0xe6792f9f,
    0xe5df07d1, 0x4321b47f, 0x7d2469d8, 0x1a3a4f90, 0x48be29a3, 0x669071af, 0x8ec8dd31, 0x0810bfbf,
    0x813a06b4, 0x68538345, 0x65865ddc, 0x43a71b8e, 0x78619a56, 0x5a34451d, 0x5bdaa3ed, 0x71edc7e9,
    0x17ac9a20, 0x78d10bfa, 0x6c1e7f35, 0xd51839d9, 0x240cbc51, 0x33513cc1, 0xd2b4f795, 0xccaa8186,
    0x0babe682, 0xa33cf164, 0x18c643ea, 0xc1ca105f, 0x9959147a, 0x6d3d94de, 0x0b654fbe, 0xed902ca0,
    0x7d835cb5, 0x99ba1509, 0x6445c922, 0x495e76c2, 0xf07194bc, 0xa1631d7e, 0x677076a5, 0x89fffe35,
    0x1a49bcf3, 0x8e6c948a, 0x0144c917, 0x8d93aea1, 0x16f87ddf, 0xc8f25d49, 0x1fb11297, 0x27e750cd,
    0x2f422da1, 0xdee89a77, 0x1534c643, 0x457b7b8b, 0xaf172f7a, 0x6b9b09d6, 0x33573f7f, 0xf14e15c4,
    0x526467d5, 0xaf488241, 0x87c3ee0d, 0x33be490c, 0x95aa6e52, 0x43ec242e, 0xd77de99b, 0xd018334f,
    0x5b78d407, 0x498eb66b, 0xb1279fa8, 0xb38b0ea6, 0x90718376, 0xe325dee2, 0x8e2f2cba, 0xcaa5bdec,
    0x9d652c56, 0xad68f5cb, 0xa77591af, 0x88e37ee8, 0xf8faa221, 0xfcbbbe47, 0x4f407786, 0xaf393889,
    0xf444a1d9, 0x15ae1a2f, 0x40aa7097, 0x6f9486ac, 0x29d232a3, 0xe47609e9, 0xe8b631ff, 0xba8565f4,
    0x11288749, 0x46c9a838, 0xeb1b7cd8, 0xf516bbb1, 0xfb74fda0, 0x010996e6, 0x4c994653, 0x1d889512,
    0x53dcd9a3, 0xdd074697, 0x1e78e17c, 0x637c98bf, 0x930bb219, 0xcf7f75b0, 0xcb9355fb, 0x9e623009,
    0xe466d82c, 0x28f968d3, 0xfeb385d9, 0x238e026c, 0xb8ed0560, 0x0c6a027a, 0x3d6fec4b, 0xbb4b2ec2,
    0xe715031c, 0xeded011d, 0xcdc4d3b9, 0xc456fc96, 0xdd0eea20, 0xb3df8ec9, 0x12351993, 0xd9cbb01c,
    0x603147a2, 0xcf37d17d, 0xf7fcd9dc, 0xd8556fa3, 0x104c8131, 0x13152774, 0xb4715811, 0x6a72c2c9,
    0xc5ae37bb, 0xa76ce12a, 0x8150d8f3, 0x2ec29218, 0xa35f0984, 0x48c0647e, 0x0b5ff98c, 0x71893f7b
];

const WINDOW_SIZE: usize = 4095;

pub struct Borg;

impl Borg {
    pub fn new(chunk_sizes: ChunkSizes) -> ChunkerWithMask<u32, BuzHashBuilder<u32>, u32> {
        new_normalized_chunker(
            chunk_sizes,
            BuzHashBuilder::new(BORG_TABLE, WINDOW_SIZE),
            Box::new(create_simple_mask),
            0,
        )
    }
}
