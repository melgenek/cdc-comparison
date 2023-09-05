// This code is ported from the https://github.com/nlfiedler/fastcdc-rs/tree/0f165fc5fd76e4c9b267bc4fa3a4ec6fcb78fe60
// The MIT License (MIT)
//
// Copyright (c) 2023 Nathan Fiedler
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

//! This module implements the canonical FastCDC algorithm as described in the
//! [paper](https://ieeexplore.ieee.org/document/9055082) by Wen Xia, et al., in 2020.
//!
//! The algorithm incorporates a simplified hash judgement using the fast Gear
//! hash, sub-minimum chunk cut-point skipping, normalized chunking to produce
//! chunks of a more consistent length, and "rolling two bytes each time".
//! According to the authors, this should be 30-40% faster than the 2016 version
//! while producing the same cut points.
use crate::chunkers::ported::fast_cdc2016::{
    create_fastcdc_mask, FAST_CDC_2016_TABLE, FAST_CDC_AVERAGE_MAX, FAST_CDC_AVERAGE_MIN,
};
use crate::chunkers::Chunker;
use crate::util::chunk_sizes::ChunkSizes;
use crate::util::logarithm2;

// GEAR table in which all values have been shifted left 1 bit, as per the
// FastCDC 2020 paper, section 3.7.
#[rustfmt::skip]
const GEAR_LS: [u64; 256] = [
    0x76ba78fa40fc6fb8, 0xf09ad1752224610c, 0x9aa5101f105ce530, 0xd59f1c9c33fb994e,
    0x863e70bbf7a2c656, 0x3abe4e003c4b57cc, 0x062617bc7935b322, 0x89644acedd36ec92,
    0x54653653c11d6932, 0x6cff97a43caefab0, 0x004f7555b4559ed6, 0xc7de5ab58e78444c,
    0x1121e49adda6256e, 0x5013c06d0a3af8fc, 0xe14dfcbc0027b036, 0x3a04c6088a59d828,
    0x070c8c64c91c491e, 0x9b559e7b9b257368, 0xebc025cc7830f0ac, 0x10c5f3a70438016c,
    0x505ee670ea1edf14, 0x3cb07b8d839616de, 0xf4628b6d2e874fe2, 0x57641fdc80900fd6,
    0x629679fc0f7074ba, 0x73b84f1315b7341e, 0x6e07ebd23754c57c, 0x9e1770cd02befb30,
    0x7b30cf883d53b9a6, 0x37c3f4ca8857e458, 0x28601b498aac63b2, 0xcd31d3978ca8b932,
    0x8ec620fd8c9d254a, 0x8cb043f8cad2d448, 0xec32d80c9045e16e, 0x90b7d083e6a4bc02,
    0xeca579390b2e9fea, 0x95b06a5f59d3c7d2, 0x54dda3b9d66bd31c, 0x8de90775b822d01e,
    0x79fb182fd39e25e2, 0x137078bc5d4ac8e2, 0x5cccb9fa49c72552, 0xd86789ca0997122a,
    0x7f7362bf93fce8a2, 0xaffa3fa328be432a, 0x635bc10a6887dfb0, 0x4abdf930c7c3c5a4,
    0x21d56c011aac859e, 0x8de090c64af59008, 0x4a5b8854f1344fa6, 0xb555bf39cef5eaca,
    0xd68f39aa7b0ffd76, 0xc613c5a7f27b6e5e, 0x8ae71ff7543ff3ac, 0xd7aafe7e4b5ef2d0,
    0xcec0a90db21a1494, 0xc29a172cc77f7b5c, 0x6f77b1b02dd60828, 0xbdf149e2d66b422c,
    0xcf265b0b555ffdac, 0x102c3975d219fa90, 0x0aaa0f7d6529e116, 0x22469d4dffa73364,
    0x5ae19e96486be604, 0xa51352eacb785a4e, 0x1cab086fff9533bc, 0x2da4e096e22b8080,
    0x1113779bf8cc1c82, 0xbc1a9ccfb924251a, 0xe553f122e0c7db2e, 0x8716d3a813c02dc6,
    0xbe9fdb14bb14872e, 0x01e83b9e83a807ee, 0x9029d6071ca4c07e, 0x3b5f7598b1c5aca2,
    0xfc5e8b1c97c2e15e, 0x88afd8829bed5280, 0x0dcc5e28a2246628, 0x7a2029a2e7752598,
    0xbde631c4bdaaeec0, 0x3fd41bd3bf950a4a, 0x8b3bc3ced840c496, 0x5dd8312c2fc5accc,
    0x24d4580d56b50796, 0x62642a646c1ec264, 0xca842a07b7680246, 0x5acf850fd4113566,
    0xd9277feb4ad7ad6c, 0x9ff6406d956db31a, 0x9cf6f0b637cf5a9e, 0xdb685dec313fa2c6,
    0xb920a510e07311ec, 0x6cbf383a58d23108, 0x8c80b06d01b337fc, 0x79a8c4980eb27d8c,
    0xfe3d51b0baf8b00a, 0x029085a9016ae292, 0x16c93796b5050d10, 0x79aae11daf3631e0,
    0xd30f90c5f7a5e5e0, 0x304e62ce3e19b058, 0x75e27d162db180c6, 0x1d4621397b2a3774,
    0xa28208b7f670b95a, 0x559783415e3efa60, 0xcc889b13be077fbe, 0x43198ee370311ff2,
    0x3068853b60387376, 0x4295ba0ffc10d43e, 0x1e0f83363ed67ff2, 0xad452f637e9ffcaa,
    0x29aab1c9278a9f8e, 0x817f8498ec8aa596, 0x2634e0df150a4196, 0x64453a64526b7aa4,
    0x4ac1a1ebb89fdf5e, 0x3b798906ab2d376c, 0x1fb038730b816ad4, 0xc0702fc2ac1e57b4,
    0x83769f03b12565aa, 0x61890c9e9c51a5ae, 0x7d9893f3b3ad84c6, 0xa260fd336a574cbc,
    0x15e56d11b5094ea4, 0xebae4a477236416c, 0xdb2bfe3fe8c6900c, 0xac5e42aaa8b06734,
    0x819c8ff11266c68c, 0x90f047ca113681b0, 0xc8e4f8fd78db2b24, 0xb51ee4efd292e174,
    0xf945e80d639176a0, 0xb63f152be2f220e0, 0xa6095f3f92050c0a, 0xa88156ff9261ce90,
    0x625a4bf794556b42, 0x21e949646949aaea, 0x20603aaa08fce8e6, 0x76c6e510d8c2b23c,
    0x5268056ef8889c0c, 0x8a3e6949a7d2fbae, 0x62b1b029b0378af6, 0x06928484b737b4d2,
    0xc4065ff3ca65b376, 0xc55c0cd71642f3ca, 0x2a8bc2185f1ae3b0, 0xffee907d65a47f80,
    0x0128bf9d7b931b0c, 0x0ec9777d3364d944, 0x363d8c4509817f86, 0xb1c1f989e1546c56,
    0xbe957df50f1a8b1a, 0xfae9585f2c0f8a32, 0x49c7f66fbf197f52, 0x7ed2fc87958ae9ca,
    0x0de2947ed1e99aa6, 0x49447a0ede39ca44, 0xce4b9b00910d1990, 0x7e78e53d648c86c4,
    0xb1ed9aaf67983db0, 0xc653ca484aa82aee, 0xc554d115ab5c3580, 0x14484acc4d37f08a,
    0x2d16348ea7594e96, 0xef135fdffe5cfc78, 0xd866c41276df99b6, 0x99d1ea17a5181364,
    0x00d45b72b5d15526, 0x2eb61ac4787a3518, 0x30c0ba726a6718b6, 0xb76dec26d95a78e2,
    0x1ea7440e03f1b14c, 0x5718b5a5cfd278ce, 0x816b58a24f595452, 0x18f7ec7840eb12be,
    0xf17b3efc029500b8, 0x6593d3e9f3918064, 0xdfac09304fd723e6, 0x57c8b3e90582df7a,
    0xb259c18ae8b55518, 0x15551f6531b2cb72, 0x566ff258d900762a, 0x18a94bd29c1e1cf0,
    0x2bf36dd218146064, 0xcf273f5486d8f0e8, 0xa2d7fd1ed5148192, 0x8930570c4c7fa5f4,
    0xc50bf673f309cb06, 0xef351bee5aec33a6, 0xe5af351bd1abba3c, 0xa206e6a9accd09c4,
    0x00990549ccd151ca, 0x63a814ecd16089cc, 0xae0af0a717a05822, 0xb68a8620f18be904,
    0x2ee24376fed4a35a, 0xe7ab997a69dff1ba, 0xc86f40fa6adc2f9a, 0x8f64f0408792ac4e,
    0x3f64a2827c83a934, 0x99ae16c0ca4a27a6, 0x392b663d14369364, 0x95ce7bfa37969836,
    0x69b3066363eb6e1e, 0xf09c73e44671b25e, 0x30c27a940c9be840, 0xe3b1b5c4be179d7c,
    0x67eef82b5d0abdf8, 0x7911677225d62138, 0x2ad45d92d75fdd4a, 0x35400b6bc15a1d0e,
    0xaa01ae0a4f89771c, 0xc6d8ae32c8439888, 0x2789a50d986ddc72, 0xaca9447b03165502,
    0xef63b827a2c357b8, 0xe69e89bcbf1abd6a, 0xc0e2fc2e94d91344, 0xa8fb2c924cd4423c,
    0xb6274864576d3d20, 0xeecd2c13f16bf878, 0x43cd58ab7db9b592, 0x36ad6c56c22cdbd4,
    0xe91ecd7272f2fd38, 0x6be665f381cd5d34, 0x98e67ed5350f1b60, 0x7b42c3c839821184,
    0x6fae95ca6b229aa2, 0x9a92761623a6c8d2, 0x9c4c9a3bf752e834, 0x53a3e5b8e86db80c,
    0xe0e7002cc098544e, 0x463a6dd2dd27e7aa, 0xeccd10232f071a32, 0x945506121555a818,
    0xe3cec2b22cd166ba, 0xe6c646c92fee614e, 0x602101c6e6f3ba9a, 0xa05bd452e304e084,
    0x858bd70b1e64c4be, 0xf0d5f73dbf5f7bfe, 0xb5dc1b0d09216548, 0xc2e6cd664d0c13ec,
    0x5c1c6b41fc8c2e7c, 0xa340fbd27d049e22, 0x0f371622bd499950, 0x275324e8ab1f5d76,
    0xf63cdc45c1140766, 0xd4c6bfb746d31ba0, 0x9ea6cb2650a074b8, 0x9bc7663cdfabaf00,
    0x1c7c8443a6c28826, 0xde29a1b0d7e34458, 0xc3b061a7e2d8bbb6, 0x557a56548a2a09c2
];

pub struct FastCdc2020 {
    mask_s: u64,
    mask_l: u64,
    mask_s_ls: u64,
    mask_l_ls: u64,
}

impl FastCdc2020 {
    pub fn new(chunk_sizes: ChunkSizes, normalization_level: u32) -> Self {
        assert!(chunk_sizes.avg_size() >= FAST_CDC_AVERAGE_MIN && chunk_sizes.avg_size() <= FAST_CDC_AVERAGE_MAX);
        let mask_s = create_fastcdc_mask(chunk_sizes.avg_size() << normalization_level);
        let mask_l = create_fastcdc_mask(chunk_sizes.avg_size() >> normalization_level);
        Self { mask_s, mask_l, mask_s_ls: mask_s << 1, mask_l_ls: mask_l << 1 }
    }
}

impl Chunker for FastCdc2020 {
    fn find_split_point(&self, buf: &[u8], chunk_sizes: &ChunkSizes) -> usize {
        let buf_length = buf.len();
        let center = if buf_length < chunk_sizes.avg_size() { buf_length } else { chunk_sizes.avg_size() };

        let mut index = chunk_sizes.min_size() / 2;
        let mut hash: u64 = 0;
        while index < center / 2 {
            let a = index * 2;
            hash = (hash << 2).wrapping_add(GEAR_LS[buf[a] as usize]);
            if (hash & self.mask_s_ls) == 0 {
                return a;
            }
            hash = hash.wrapping_add(FAST_CDC_2016_TABLE[buf[a + 1] as usize]);
            if (hash & self.mask_s) == 0 {
                return a + 1;
            }
            index += 1;
        }

        while index < buf_length / 2 {
            let a = index * 2;
            hash = (hash << 2).wrapping_add(GEAR_LS[buf[a] as usize]);
            if (hash & self.mask_l_ls) == 0 {
                return a;
            }
            hash = hash.wrapping_add(FAST_CDC_2016_TABLE[buf[a + 1] as usize]);
            if (hash & self.mask_l) == 0 {
                return a + 1;
            }
            index += 1;
        }

        index
    }
}
