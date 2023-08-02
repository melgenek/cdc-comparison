// This code is ported from the https://github.com/restic/chunker/tree/db20dabc1bbeee68a21930061a3617d31fed1f29
//
// Copyright (c) 2014, Alexander Neumann <alexander@bumpern.de>
// Copyright (c) 2023, melgenek
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice, this
//    list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice,
//    this list of conditions and the following disclaimer in the documentation
//    and/or other materials provided with the distribution.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
// ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
// WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
// OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use std::cmp::Ordering;
use std::ops::{Add, BitAnd, BitOr, Div, Mul, Rem, Shl};

/// Pol is a polynomial from F_2\[X\].
#[derive(Copy, Clone, Debug)]
pub struct Pol(u64);

impl Pol {
    pub const ZERO: Pol = Pol(0);

    /// Returns a new random irreducible polynomial of degree 53 using.
    /// It is equivalent to calling [Pol::generate_random_from_seed] with seed `1`.
    pub fn generate_random() -> Pol {
        Self::generate_random_from_seed(1)
    }

    /// Returns an irreducible polynomial of degree 53 (largest prime number below 64-8) by generating random values.
    /// There are (2^53-2/53) irreducible polynomials of degree 53 in F_2\[X\],
    /// c.f. Michael O. Rabin (1981): "Fingerprinting by Random Polynomials", page 4.
    /// If no polynomial could be found in one million tries, the function panics.
    pub fn generate_random_from_seed(seed: u64) -> Pol {
        let mut rng = ChaCha20Rng::seed_from_u64(seed);
        for _ in 0..1_000_000 {
            let mut f: Pol = Pol(rng.gen::<u64>());
            // mask away bits above bit 53
            f = f & Pol((1 << 54) - 1);
            // set highest and lowest bit so that the degree is 53 and the
            // polynomial is not trivially reducible
            f = f | Pol((1 << 53) | 1);
            if f.is_irreducible() {
                return f;
            }
        }
        panic!("Unable to find new random irreducible polynomial")
    }

    pub fn value(&self) -> u64 {
        self.0
    }

    /// Returns the degree of the polynomial x. If x is zero, -1 is returned.
    pub fn deg(&self) -> i32 {
        63 as i32 - self.0.leading_zeros() as i32
    }

    /// Computes the Greatest Common Divisor x and f.
    fn gcd(x: Pol, f: Pol) -> Pol {
        if f == 0 {
            x
        } else if x == 0 {
            f
        } else if x.deg() < f.deg() {
            Self::gcd(x, f % x)
        } else {
            Self::gcd(f, x % f)
        }
    }

    /// computes x*f mod g
    fn mul_mod(self, f: Pol, g: Pol) -> Pol {
        if self == 0 || f == 0 {
            Pol(0)
        } else {
            let mut res = Pol(0);
            for i in 0..=f.deg() {
                if (f & Pol(1 << i)) > 0 {
                    let mut a = self;
                    for _ in 0..i {
                        a = (a * Pol(2)) % g;
                    }
                    res = (res + a) % g;
                }
            }
            res
        }
    }

    /// Irreducible returns true if x is irreducible over F_2.
    /// This function uses Ben Or's reducibility test.
    /// For details see "Tests and Constructions of Irreducible Polynomials over Finite Fields".
    pub fn is_irreducible(&self) -> bool {
        /// Computes the polynomial (x^(2^p)-x) mod g
        fn qp(p: i32, g: Pol) -> Pol {
            let num = 1 << p;
            let mut res = Pol(2);
            let mut i = 1;
            while i < num {
                res = res.mul_mod(res, g);
                i *= 2;
            }
            (res + Pol(2)) % g
        }

        for i in 1..=self.deg() / 2 {
            if Pol::gcd(*self, qp(i, *self)) != 1 {
                return false;
            }
        }
        true
    }

    /// Returns x / d = q, and remainder r,
    /// see https://en.wikipedia.org/wiki/Division_algorithm
    fn div_mod(self, d: Pol) -> (Pol, Pol) {
        if self == 0 {
            (Pol(0), Pol(0))
        } else if d == 0 {
            panic!("Division by zero")
        } else {
            let mut x = self;
            let d_deg = d.deg();
            let mut diff = x.deg() - d_deg;
            if diff < 0 {
                (Pol(0), x)
            } else {
                let mut q = Pol(0);
                while diff >= 0 {
                    let m = d << diff;
                    q = q | Pol(1 << diff);
                    x = x + m;
                    diff = x.deg() - d_deg;
                }
                (q, x)
            }
        }
    }
}

impl From<u8> for Pol {
    fn from(value: u8) -> Self {
        Self(value as u64)
    }
}

impl From<u64> for Pol {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl PartialEq<u64> for Pol {
    fn eq(&self, other: &u64) -> bool {
        &self.0 == other
    }
}

impl PartialEq for Pol {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialOrd<u64> for Pol {
    fn partial_cmp(&self, other: &u64) -> Option<Ordering> {
        if &self.0 < other {
            Some(Ordering::Less)
        } else if &self.0 > other {
            Some(Ordering::Greater)
        } else {
            Some(Ordering::Equal)
        }
    }
}

impl Add for Pol {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl Shl<i32> for Pol {
    type Output = Self;

    fn shl(self, rhs: i32) -> Self::Output {
        Self(self.0 << rhs)
    }
}

/// Returns the integer division result x / d.
impl Div for Pol {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        self.div_mod(rhs).0
    }
}

/// Returns x*y. When an overflow occurs, Mul panics.
impl Mul for Pol {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Pol(0), Pol(0)) => Pol(0),
            (Pol(1), y) => y,
            (x, Pol(1)) => x,
            (x, Pol(2)) => {
                if x & Pol(1 << 63) != 0 {
                    panic!("multiplication would overflow u64")
                }
                return x << 1;
            }
            (x, y) => {
                let mut res = Pol(0);
                for i in 0..=y.deg() {
                    if (y & Pol(1 << i)) > 0 {
                        res = res + (x << i);
                    }
                }
                if res / y != x {
                    panic!("multiplication would overflow u64")
                }
                res
            }
        }
    }
}

/// Returns the remainder of x / d
impl Rem for Pol {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        self.div_mod(rhs).1
    }
}

impl BitAnd for Pol {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitOr for Pol {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

#[cfg(test)]
mod tests {
    use super::Pol;

    #[test]
    fn should_get_degree() {
        assert_eq!(Pol(0).deg(), -1);
        assert_eq!(Pol(1).deg(), 0);
        for i in 0..64 {
            let x = 1 << i;
            assert_eq!(Pol(x).deg(), i);
        }
        assert_eq!(Pol(0x3af4b284899).deg(), 41);
    }

    #[test]
    fn should_add() {
        assert_eq!(Pol(23) + Pol(16), 23 ^ 16);
        assert_eq!(Pol(0x9a7e30d1e855e0a0) + Pol(0x670102a1f4bcd414), 0xfd7f32701ce934b4);
        assert_eq!(Pol(0x9a7e30d1e855e0a0) + Pol(0x9a7e30d1e855e0a0), 0);
    }

    #[test]
    fn should_divide() {
        assert_eq!(Pol(10) / Pol(50), 0);
        assert_eq!(Pol(0) / Pol(1), 0);
        assert_eq!(Pol(0b101101000) / Pol(0b1010), 0b100100);
        assert_eq!(Pol(2) / Pol(2), Pol(1));
        assert_eq!(Pol(0x8000000000000000) / Pol(0x8000000000000000), 1);
        assert_eq!(Pol(0b1100) / Pol(0b100), 0b11);
        assert_eq!(Pol(0b1100001111) / Pol(0b10011), 0b110101);
    }

    #[test]
    fn should_find_remainder() {
        assert_eq!(Pol(10) % Pol(50), 10);
        assert_eq!(Pol(0) % Pol(1), 0);
        assert_eq!(Pol(0b101101001) % Pol(0b1010), 0b1);
        assert_eq!(Pol(2) % Pol(2), Pol(0));
        assert_eq!(Pol(0x8000000000000000) % Pol(0x8000000000000000), 0);
        assert_eq!(Pol(0b1100) % Pol(0b100), 0b0);
        assert_eq!(Pol(0b1100001111) % Pol(0b10011), 0b0);
    }

    #[test]
    fn should_multiply() {
        let operands_and_result = [
            (Pol(1), Pol(2), Pol(2)),
            (Pol(0b1101), Pol(0b10), Pol(0b11010)),
            (Pol(0b1101), Pol(0b11), Pol(0b10111)),
            (Pol(0x40000000), Pol(0x40000000), Pol(0x1000000000000000)),
            (Pol(0b1010), Pol(0b100100), Pol(0b101101000)),
            (Pol(0b100), Pol(0b11), Pol(0b1100)),
            (Pol(0b11), Pol(0b110101), Pol(0b1011111)),
            (Pol(0b10011), Pol(0b110101), Pol(0b1100001111)),
        ];

        for (a, b, result) in operands_and_result {
            let res1 = a * b;
            let res2 = b * a;
            assert_eq!(res1, result);
            assert_eq!(res1, res2);
        }
    }

    #[test]
    #[should_panic]
    fn should_panic_when_mul_overflow() {
        println!("{:?}", Pol(1 << 63) * Pol(2));
    }

    #[test]
    fn should_mul_mod() {
        assert_eq!(Pol(0x1230).mul_mod(Pol(0x230), Pol(0x55)), 0x22);
        assert_eq!(Pol(0x0eae8c07dbbb3026).mul_mod(Pol(0xd5d6db9de04771de), Pol(0xdd2bda3b77c9)), 0x425ae8595b7a);
    }

    #[test]
    fn should_check_if_irreducible() {
        assert!(!Pol(0x38f1e565e288df).is_irreducible());
        assert!(Pol(0x3DA3358B4DC173).is_irreducible());
        assert!(!Pol(0x30a8295b9d5c91).is_irreducible());
        assert!(!Pol(0x255f4350b962cb).is_irreducible());
        assert!(!Pol(0x267f776110a235).is_irreducible());
        assert!(!Pol(0x2f4dae10d41227).is_irreducible());
        assert!(Pol(0x2482734cacca49).is_irreducible());
        assert!(!Pol(0x312daf4b284899).is_irreducible());
        assert!(!Pol(0x29dfb6553d01d1).is_irreducible());
        assert!(!Pol(0x3548245eb26257).is_irreducible());
        assert!(!Pol(0x3199e7ef4211b3).is_irreducible());
        assert!(!Pol(0x362f39017dae8b).is_irreducible());
        assert!(!Pol(0x200d57aa6fdacb).is_irreducible());
        assert!(!Pol(0x35e0a4efa1d275).is_irreducible());
        assert!(!Pol(0x2ced55b026577f).is_irreducible());
        assert!(!Pol(0x260b012010893d).is_irreducible());
        assert!(!Pol(0x2df29cbcd59e9d).is_irreducible());
        assert!(!Pol(0x3f2ac7488bd429).is_irreducible());
        assert!(!Pol(0x3e5cb1711669fb).is_irreducible());
        assert!(!Pol(0x226d8de57a9959).is_irreducible());
        assert!(!Pol(0x3c8de80aaf5835).is_irreducible());
        assert!(!Pol(0x2026a59efb219b).is_irreducible());
        assert!(!Pol(0x39dfa4d13fb231).is_irreducible());
        assert!(!Pol(0x3143d0464b3299).is_irreducible());
    }

    #[test]
    fn should_generate_random() {
        assert!(Pol::generate_random() > 0);
    }

    #[test]
    fn should_find_gcd() {
        assert_eq!(Pol::gcd(Pol(10), Pol(50)), 2);
        assert_eq!(Pol::gcd(Pol(0), Pol(1)), 1);
        assert_eq!(Pol::gcd(Pol(0b101101001), Pol(0b1010)), 1);
        assert_eq!(Pol::gcd(Pol(2), Pol(2)), 2);
        assert_eq!(Pol::gcd(Pol(0b1010), Pol(0b11)), 0b11);
        assert_eq!(Pol::gcd(Pol(0x8000000000000000), Pol(0x8000000000000000)), 0x8000000000000000);
        assert_eq!(Pol::gcd(Pol(0b1100), Pol(0b101)), 0b11);
        assert_eq!(Pol::gcd(Pol(0b1100001111), Pol(0b10011)), 0b10011);
        assert_eq!(Pol::gcd(Pol(0x3da3358b4dc173), Pol(0x3da3358b4dc173)), 0x3da3358b4dc173);
        assert_eq!(Pol::gcd(Pol(0x3da3358b4dc173), Pol(0x230d2259defd)), 1);
        assert_eq!(Pol::gcd(Pol(0x230d2259defd), Pol(0x51b492b3eff2)), 0b10011);
    }
}
