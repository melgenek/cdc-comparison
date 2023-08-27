pub mod polynomial;

use crate::hashes::polynomial_hash::polynomial::Pol;
use crate::hashes::{RollingHash, RollingHashBuilder};
use crate::util::unsigned_integer::UnsignedInteger;

struct Tables {
    out: [Pol; 256],
    mods: [Pol; 256],
}

impl Tables {
    fn new(pol: Pol, window_size: usize) -> Tables {
        let mut out = [Pol::ZERO; 256];
        let mut mods = [Pol::ZERO; 256];

        // calculate table for sliding out bytes. The byte to slide out is used as
        // the index for the table, the value contains the following:
        // out_table[b] = Hash(b || 0 ||        ...        || 0)
        //                          \ windowsize-1 zero bytes /
        // To slide out byte b_0 for window size w with known hash
        // H := H(b_0 || ... || b_w), it is sufficient to add out_table[b_0]:
        //    H(b_0 || ... || b_w) + H(b_0 || 0 || ... || 0)
        //  = H(b_0 + b_0 || b_1 + 0 || ... || b_w + 0)
        //  = H(    0     || b_1 || ...     || b_w)
        //
        // Afterwards a new byte can be shifted in.
        for b in 0..256 {
            let mut h = Tables::append_byte(Pol::ZERO, b as u8, pol);
            for _ in 0..(window_size - 1) {
                h = Tables::append_byte(h, 0, pol);
            }
            out[b] = h;
        }

        // calculate table for reduction mod Polynomial
        let k = pol.deg();
        for b in 0..256 {
            // for b := 0; b < 256; b++ {
            // mod_table[b] = A | B, where A = (b(x) * x^k mod pol) and  B = b(x) * x^k
            //
            // The 8 bits above deg(Polynomial) determine what happens next and so
            // these bits are used as a lookup to this table. The value is split in
            // two parts: Part A contains the result of the modulus operation, part
            // B is used to cancel out the 8 top bits so that one XOR operation is
            // enough to reduce modulo Polynomial
            mods[b] = (Pol::from((b as u64) << k) % pol) | (Pol::from(b as u64) << k)
        }
        Tables { out, mods }
    }

    fn append_byte(digest: Pol, b: u8, pol: Pol) -> Pol {
        let digest = digest << 8;
        let digest = digest | Pol::from(b);
        digest % pol
    }
}

pub struct PolynomialHashBuilder {
    tables: Tables,
    window_size: usize,
    pol_shift: u64,
}

impl PolynomialHashBuilder {
    pub fn new(pol: Pol, window_size: usize) -> Self {
        let pol_shift = (pol.deg() - 8) as u64;
        if pol_shift > 53 - 8 {
            panic!("The polynomial must have a degree less than or equal 53")
        }

        let tables = Tables::new(pol, window_size);
        Self { tables, window_size, pol_shift }
    }
}

impl RollingHashBuilder<u64> for PolynomialHashBuilder {
    type RH<'a> = PolynomialHash<'a>;

    fn prepare_bytes_count(&self) -> usize {
        self.window_size
    }

    fn new_hash(&self, buffer: &[u8]) -> Self::RH<'_> {
        PolynomialHash::new(self, buffer)
    }
}

pub struct PolynomialHash<'a> {
    builder: &'a PolynomialHashBuilder,
    digest: u64,
    window: Vec<u8>,
    window_idx: usize,
}

impl<'a> PolynomialHash<'a> {
    fn new(builder: &'a PolynomialHashBuilder, buffer: &[u8]) -> Self {
        let mut hash = Self { builder, digest: 0, window: vec![0; builder.window_size], window_idx: 0 };

        for new_byte in buffer {
            hash.roll(*new_byte);
        }

        hash
    }

    fn replace_and_return_oldest_window_byte(&mut self, new_byte: u8) -> u8 {
        let old_byte = self.window[self.window_idx];
        self.window[self.window_idx] = new_byte;
        self.window_idx = (self.window_idx + 1) % self.builder.window_size;
        old_byte
    }
}

impl<'a> RollingHash<'a, u64> for PolynomialHash<'a> {
    fn roll(&mut self, new_byte: u8) {
        let old_byte = self.replace_and_return_oldest_window_byte(new_byte);
        self.digest = self.digest ^ self.builder.tables.out[old_byte as usize].value();
        let index = self.digest >> self.builder.pol_shift;
        self.digest = ((self.digest << 8) | (new_byte as u64)) ^ self.builder.tables.mods[index as usize].value();
    }

    fn digest(&self) -> u64 {
        self.digest
    }
}
