#![doc(html_root_url = "https://docs.rs/nilsimsa/0.1.0")]
#![feature(test)]

//! Implementation of the Nilsimsa locality-sensitive hashing algorithm.
//!
//! Compared to "traditional" hash functions (cryptographic or not), a small modification to the input does not
//! substantially change the resulting hash. This crate contains the [Nilsimsa](Nilsimsa) utility to calculate Nilsimsa
//! hash digests, as well as a [compare](comparison) function for given digests.
//!
//! ```rust
//! # use nilsimsa::*;
//! # fn main() {
//! let mut hasher = Nilsimsa::new();
//! hasher.update("test string");
//! let digest = hasher.digest();
//! assert_eq!(
//!     digest,
//!     "42c82c184080082040001004000000084e1043b0c0925829003e84c860410010"
//! );
//! # }
//! ```

const TRAN: [u8; 256] = [
    0x02, 0xd6, 0x9e, 0x6f, 0xf9, 0x1d, 0x04, 0xab, 0xd0, 0x22, 0x16, 0x1f, 0xd8, 0x73, 0xa1, 0xac, 0x3b, 0x70, 0x62,
    0x96, 0x1e, 0x6e, 0x8f, 0x39, 0x9d, 0x05, 0x14, 0x4a, 0xa6, 0xbe, 0xae, 0x0e, 0xcf, 0xb9, 0x9c, 0x9a, 0xc7, 0x68,
    0x13, 0xe1, 0x2d, 0xa4, 0xeb, 0x51, 0x8d, 0x64, 0x6b, 0x50, 0x23, 0x80, 0x03, 0x41, 0xec, 0xbb, 0x71, 0xcc, 0x7a,
    0x86, 0x7f, 0x98, 0xf2, 0x36, 0x5e, 0xee, 0x8e, 0xce, 0x4f, 0xb8, 0x32, 0xb6, 0x5f, 0x59, 0xdc, 0x1b, 0x31, 0x4c,
    0x7b, 0xf0, 0x63, 0x01, 0x6c, 0xba, 0x07, 0xe8, 0x12, 0x77, 0x49, 0x3c, 0xda, 0x46, 0xfe, 0x2f, 0x79, 0x1c, 0x9b,
    0x30, 0xe3, 0x00, 0x06, 0x7e, 0x2e, 0x0f, 0x38, 0x33, 0x21, 0xad, 0xa5, 0x54, 0xca, 0xa7, 0x29, 0xfc, 0x5a, 0x47,
    0x69, 0x7d, 0xc5, 0x95, 0xb5, 0xf4, 0x0b, 0x90, 0xa3, 0x81, 0x6d, 0x25, 0x55, 0x35, 0xf5, 0x75, 0x74, 0x0a, 0x26,
    0xbf, 0x19, 0x5c, 0x1a, 0xc6, 0xff, 0x99, 0x5d, 0x84, 0xaa, 0x66, 0x3e, 0xaf, 0x78, 0xb3, 0x20, 0x43, 0xc1, 0xed,
    0x24, 0xea, 0xe6, 0x3f, 0x18, 0xf3, 0xa0, 0x42, 0x57, 0x08, 0x53, 0x60, 0xc3, 0xc0, 0x83, 0x40, 0x82, 0xd7, 0x09,
    0xbd, 0x44, 0x2a, 0x67, 0xa8, 0x93, 0xe0, 0xc2, 0x56, 0x9f, 0xd9, 0xdd, 0x85, 0x15, 0xb4, 0x8a, 0x27, 0x28, 0x92,
    0x76, 0xde, 0xef, 0xf8, 0xb2, 0xb7, 0xc9, 0x3d, 0x45, 0x94, 0x4b, 0x11, 0x0d, 0x65, 0xd5, 0x34, 0x8b, 0x91, 0x0c,
    0xfa, 0x87, 0xe9, 0x7c, 0x5b, 0xb1, 0x4d, 0xe5, 0xd4, 0xcb, 0x10, 0xa2, 0x17, 0x89, 0xbc, 0xdb, 0xb0, 0xe2, 0x97,
    0x88, 0x52, 0xf7, 0x48, 0xd3, 0x61, 0x2c, 0x3a, 0x2b, 0xd1, 0x8c, 0xfb, 0xf1, 0xcd, 0xe4, 0x6a, 0xe7, 0xa9, 0xfd,
    0xc4, 0x37, 0xc8, 0xd2, 0xf6, 0xdf, 0x58, 0x72, 0x4e,
];

const POPC: [u8; 256] = [
    0x00, 0x01, 0x01, 0x02, 0x01, 0x02, 0x02, 0x03, 0x01, 0x02, 0x02, 0x03, 0x02, 0x03, 0x03, 0x04, 0x01, 0x02, 0x02,
    0x03, 0x02, 0x03, 0x03, 0x04, 0x02, 0x03, 0x03, 0x04, 0x03, 0x04, 0x04, 0x05, 0x01, 0x02, 0x02, 0x03, 0x02, 0x03,
    0x03, 0x04, 0x02, 0x03, 0x03, 0x04, 0x03, 0x04, 0x04, 0x05, 0x02, 0x03, 0x03, 0x04, 0x03, 0x04, 0x04, 0x05, 0x03,
    0x04, 0x04, 0x05, 0x04, 0x05, 0x05, 0x06, 0x01, 0x02, 0x02, 0x03, 0x02, 0x03, 0x03, 0x04, 0x02, 0x03, 0x03, 0x04,
    0x03, 0x04, 0x04, 0x05, 0x02, 0x03, 0x03, 0x04, 0x03, 0x04, 0x04, 0x05, 0x03, 0x04, 0x04, 0x05, 0x04, 0x05, 0x05,
    0x06, 0x02, 0x03, 0x03, 0x04, 0x03, 0x04, 0x04, 0x05, 0x03, 0x04, 0x04, 0x05, 0x04, 0x05, 0x05, 0x06, 0x03, 0x04,
    0x04, 0x05, 0x04, 0x05, 0x05, 0x06, 0x04, 0x05, 0x05, 0x06, 0x05, 0x06, 0x06, 0x07, 0x01, 0x02, 0x02, 0x03, 0x02,
    0x03, 0x03, 0x04, 0x02, 0x03, 0x03, 0x04, 0x03, 0x04, 0x04, 0x05, 0x02, 0x03, 0x03, 0x04, 0x03, 0x04, 0x04, 0x05,
    0x03, 0x04, 0x04, 0x05, 0x04, 0x05, 0x05, 0x06, 0x02, 0x03, 0x03, 0x04, 0x03, 0x04, 0x04, 0x05, 0x03, 0x04, 0x04,
    0x05, 0x04, 0x05, 0x05, 0x06, 0x03, 0x04, 0x04, 0x05, 0x04, 0x05, 0x05, 0x06, 0x04, 0x05, 0x05, 0x06, 0x05, 0x06,
    0x06, 0x07, 0x02, 0x03, 0x03, 0x04, 0x03, 0x04, 0x04, 0x05, 0x03, 0x04, 0x04, 0x05, 0x04, 0x05, 0x05, 0x06, 0x03,
    0x04, 0x04, 0x05, 0x04, 0x05, 0x05, 0x06, 0x04, 0x05, 0x05, 0x06, 0x05, 0x06, 0x06, 0x07, 0x03, 0x04, 0x04, 0x05,
    0x04, 0x05, 0x05, 0x06, 0x04, 0x05, 0x05, 0x06, 0x05, 0x06, 0x06, 0x07, 0x04, 0x05, 0x05, 0x06, 0x05, 0x06, 0x06,
    0x07, 0x05, 0x06, 0x06, 0x07, 0x06, 0x07, 0x07, 0x08,
];

/// Utility to calculate Nilsimsa hash digests for arbitrarily long string inputs. See the crate-level documentation for
/// an example of use.
#[derive(Debug, Clone)]
pub struct Nilsimsa {
    num_char: usize,
    acc: Vec<u8>,
    window: Vec<u8>,
}

impl Default for Nilsimsa {
    fn default() -> Self {
        Self {
            num_char: 0,
            acc: vec![0; 256],
            window: Vec::new(),
        }
    }
}

impl Nilsimsa {
    /// Returns a new Nilsimsa hash digest utility.
    pub fn new() -> Self {
        Default::default()
    }

    /// Updates the digest with a given string.
    pub fn update(&mut self, s: &str) {
        for c in s.bytes() {
            self.num_char += 1;

            let window_len = self.window.len();
            if window_len > 1 {
                self.acc[tran_hash(c, self.window[0], self.window[1], 0) as usize] += 1;
            }

            if window_len > 2 {
                self.acc[tran_hash(c, self.window[0], self.window[2], 1) as usize] += 1;
                self.acc[tran_hash(c, self.window[1], self.window[2], 2) as usize] += 1;
            }

            if window_len > 3 {
                self.acc[tran_hash(c, self.window[0], self.window[3], 3) as usize] += 1;
                self.acc[tran_hash(c, self.window[1], self.window[3], 4) as usize] += 1;
                self.acc[tran_hash(c, self.window[2], self.window[3], 5) as usize] += 1;

                self.acc[tran_hash(self.window[3], self.window[0], c, 6) as usize] += 1;
                self.acc[tran_hash(self.window[3], self.window[2], c, 7) as usize] += 1;
            }

            self.window.insert(0, c);
            if self.window.len() > 4 {
                self.window.remove(4);
            }
        }
    }

    /// Finalise and consume the digest and return the computed Nilsimsa hash digest as a hex string.
    pub fn digest(self) -> String {
        let num_trigrams = match self.num_char {
            0..=2 => 0,
            3 => 1,
            4 => 4,
            n => 8 * n - 28,
        };

        let threshold = num_trigrams / 256;
        let mut digest = [0u8; 32];

        for i in 0..256 {
            if self.acc[i] as usize > threshold {
                digest[i >> 3] += 1 << (i & 7);
            }
        }

        digest.reverse();
        hex::encode(digest)
    }
}

/// Compare two hex digests with a Hamming distance calculation. Returns an unsigned 8-bit integer in the range `[0,
/// 128]` representing the similarity of the two input digests, where 0 is most dissimilar and 128 is most similar, or
/// equal. The input strings must be of the same length.
///
/// ```rust
/// # use nilsimsa::*;
/// # fn main() {
/// let similar = compare(
///     "42c82c184080082040001004000000084e1043b0c0925829003e84c860410010",
///     "00480cba20810802408000000400000a481091b088b21e21003e840a20011016",
/// );
/// assert_eq!(similar, 90);
///
/// let very_dissimilar = compare(
///     "51613b08c286b8054e09847c51928935289e623b63308db6b1606b0883804264",
///     "1db4dd17fb93907f2dbb52a5d7dddc268f15545be7da0f75efcb0f9df7cc65b3",
/// );
/// assert_eq!(very_dissimilar, 1);
/// # }
/// ```
pub fn compare(digest_a: &str, digest_b: &str) -> u8 {
    assert!(digest_a.len() == digest_b.len());

    let hex_a = hex::decode(digest_a).expect("failed to decode digest A into hex");
    let hex_b = hex::decode(digest_b).expect("failed to decode digest B into hex");
    let mut bits = 0;

    for (a, b) in hex_a.into_iter().zip(hex_b) {
        bits += POPC[(a ^ b) as usize] as u8;
    }

    128 - bits
}

fn tran_hash(a: u8, b: u8, c: u8, n: u8) -> u8 {
    (TRAN[(a.wrapping_add(n)) as usize] ^ (TRAN[b as usize].wrapping_mul(n.wrapping_add(n).wrapping_add(1))))
        .wrapping_add(TRAN[(c ^ TRAN[n as usize]) as usize])
}

#[cfg(test)]
mod tests {
    extern crate test;

    use super::*;
    use test::Bencher;

    #[bench]
    fn expected_output(b: &mut Bencher) {
        b.iter(|| {
            let mut hash = Nilsimsa::default();
            hash.update("test string");
            let output = hash.digest();

            assert_eq!(
                output,
                "42c82c184080082040001004000000084e1043b0c0925829003e84c860410010"
            );
        })
    }

    #[bench]
    fn compare_equal(b: &mut Bencher) {
        b.iter(|| {
            let hash_a = String::from("42c82c184080082040001004000000084e1043b0c0925829003e84c860410010");
            let hash_b = hash_a.clone();

            assert_eq!(compare(&hash_a, &hash_b), 128);
        });
    }

    #[bench]
    fn compare_almost_equal(b: &mut Bencher) {
        b.iter(|| {
            // input: test string
            let hash_a = String::from("42c82c184080082040001004000000084e1043b0c0925829003e84c860410010");
            // input: best strong
            let hash_b = String::from("00480cba20810802408000000400000a481091b088b21e21003e840a20011016");

            assert_eq!(compare(&hash_a, &hash_b), 90);
        });
    }

    #[bench]
    fn compare_very_dissimilar(b: &mut Bencher) {
        b.iter(|| {
            // input: Dear Bill, Please be ready to receive the money.
            let hash_a = String::from("51613b08c286b8054e09847c51928935289e623b63308db6b1606b0883804264");
            // input: Dear Mark, I hope you are okay.
            let hash_b = String::from("1db4dd17fb93907f2dbb52a5d7dddc268f15545be7da0f75efcb0f9df7cc65b3");

            assert_eq!(compare(&hash_a, &hash_b), 1);
        });
    }

    #[bench]
    fn long_string(b: &mut Bencher) {
        b.iter(|| {
            let mut hash = Nilsimsa::default();
            hash.update(
                "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Suspendisse dictum odio id massa rhoncus, \
                 nec congue ante hendrerit. Donec elementum sollicitudin arcu, ut ultricies libero ultrices sed. \
                 Phasellus hendrerit urna quis tellus porta, pharetra congue risus elementum. Vivamus finibus \
                 malesuada mollis. Nulla mollis sit amet est ac commodo. Integer ac lacus in tellus condimentum \
                 tempus. Quisque sed ligula eget felis lobortis tempor nec vel neque. Etiam nisi urna, malesuada at \
                 rhoncus et, pharetra in ligula. Pellentesque venenatis efficitur magna vel consequat. Duis a \
                 sollicitudin mi. Pellentesque rutrum placerat consequat. Ut tristique, neque in dignissim aliquet, \
                 enim est luctus nisi, nec mollis lacus risus eu quam. Suspendisse potenti. Mauris pellentesque purus \
                 et neque vehicula, nec tempor purus ornare. Mauris pharetra turpis vel nulla ultrices, non imperdiet \
                 ante egestas. Sed rhoncus dolor non maximus gravida. Nam tristique ante sit amet consectetur \
                 tincidunt. Ut vitae scelerisque neque. Nulla nec tristique mauris. Mauris elementum turpis at purus \
                 venenatis congue. Donec pellentesque congue arcu, ac suscipit massa aliquet quis. Aenean tincidunt \
                 tempor ultrices. Sed vel ultrices magna. Etiam viverra accumsan neque, id gravida justo egestas \
                 vitae. Aliquam et libero magna. Etiam eu semper elit, ut eleifend orci. Curabitur volutpat suscipit \
                 tincidunt. Suspendisse id molestie enim. Sed vitae vehicula tellus, et pulvinar risus. Curabitur \
                 ornare vel ligula sed pulvinar. Praesent faucibus erat massa, ac pulvinar lacus faucibus sed. Sed \
                 hendrerit nec arcu sit amet luctus. Donec mollis ligula lacus, eget mollis augue dictum eget. Donec \
                 vitae dui vel ligula pellentesque pulvinar a pulvinar nulla. Nam nec nulla quam. Morbi vel sodales \
                 nisi. Proin vitae mattis dui, id accumsan lacus. Nullam rhoncus fermentum nunc at tempus. In hac \
                 habitasse platea dictumst. Curabitur vel molestie augue.Nam et elementum risus. Sed in turpis non \
                 augue tempus dictum. Duis eu arcu eu tortor mollis blandit. Nam feugiat felis eu varius scelerisque. \
                 Donec venenatis, ex sit amet fermentum fringilla, lorem tellus dictum turpis, sit amet tristique \
                 quam nunc at lorem. Nam tincidunt leo non vulputate feugiat. Pellentesque ut porttitor massa. \
                 Pellentesque habitant morbi tristique senectus et netus et malesuada fames ac turpis egestas. \
                 Integer bibendum diam sed turpis hendrerit sodales. Ut hendrerit auctor enim, volutpat bibendum \
                 risus dapibus in.",
            );
            let output = hash.digest();

            assert_eq!(
                output,
                "9b8c8a910218eb47d0f283c5ac948ba12c0ba8112513eae8291befdca3f4e066"
            );
        })
    }
}
