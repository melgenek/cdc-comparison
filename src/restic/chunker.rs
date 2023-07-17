use crate::chunk_stream::{ChunkSizes, SplitPointFinder};
use crate::restic::polynomial::Pol;
use crate::util::logarithm2;

const WINDOW_SIZE: usize = 64;

fn update_digest(digest: u64, pol_shift: u64, tables: &Tables, b: u8) -> u64 {
    let index = digest >> pol_shift;
    let digest = digest << 8;
    let digest = digest | (b as u64);
    let digest = digest ^ tables.mods[index as usize].value();
    digest
}

struct Tables {
    out: [Pol; 256],
    mods: [Pol; 256],
}

impl Tables {
    fn new(pol: Pol) -> Tables {
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
            for _ in 0..(WINDOW_SIZE - 1) {
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

    fn append_byte(hash: Pol, b: u8, pol: Pol) -> Pol {
        let hash = hash << 8;
        let hash = hash | Pol::from(b);
        hash % pol
    }
}

pub struct ResticCdc {
    tables: Tables,
    pol_shift: u64,
    split_mask: u64,
}

impl ResticCdc {
    pub fn new(pol: Pol, chunk_sizes: ChunkSizes) -> Self {
        assert!(chunk_sizes.avg_size() <= u32::MAX as usize);
        let bits = logarithm2(chunk_sizes.avg_size() as u32);
        let split_mask = (1 << bits) - 1;
        let pol_shift = (pol.deg() - 8) as u64;
        if pol_shift > 53 - 8 {
            panic!("The polynomial must have a degree less than or equal 53")
        }
        let tables = Tables::new(pol);
        ResticCdc { tables, pol_shift, split_mask }
    }
}

impl SplitPointFinder for ResticCdc {
    fn find_split_point(&self, buf: &[u8], chunk_sizes: &ChunkSizes) -> usize {
        let mut window: [u8; WINDOW_SIZE] = [0; WINDOW_SIZE];
        let mut oldest_idx: usize = 0;

        let mut slide = |digest: u64, b: u8| -> u64 {
            let out = window[oldest_idx];
            window[oldest_idx] = b;
            oldest_idx = (oldest_idx + 1) % WINDOW_SIZE;
            let digest = digest ^ self.tables.out[out as usize].value();
            let digest = update_digest(digest, self.pol_shift, &self.tables, b);
            digest
        };
        let mut digest = slide(0, 1);

        let mut i = chunk_sizes.min_size() - WINDOW_SIZE;

        // todo the initialisation here might be wrong
        while i < chunk_sizes.min_size() {
            digest = slide(digest, buf[i as usize]);
            i += 1;
        }

        while i < buf.len() {
            if (digest & self.split_mask) == 0 {
                break;
            }
            digest = slide(digest, buf[i as usize]);
            i += 1;
        }
        i
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use data_encoding::{DecodeError, HEXLOWER};
    use crate::chunk_stream::{ChunkSizes, ChunkStream, SplitPointFinder};
    use crate::restic::chunker::ResticCdc;
    use crate::restic::polynomial::Pol;
    use crate::util::sha256;

    #[test]
    pub fn test1() {
        let pol = Pol::from(0x3DA3358B4DC173 as u64);
        let chunks: Vec<(usize, usize, &str)> = vec![
            (2163460, 0x000b98d4cdf00000, "4b94cb2cf293855ea43bf766731c74969b91aa6bf3c078719aabdd19860d590d"),
            (643703, 0x000d4e8364d00000, "5727a63c0964f365ab8ed2ccf604912f2ea7be29759a2b53ede4d6841e397407"),
            (1528956, 0x0015a25c2ef00000, "a73759636a1e7a2758767791c69e81b69fb49236c6929e5d1b654e06e37674ba"),
            (1955808, 0x00102a8242e00000, "c955fb059409b25f07e5ae09defbbc2aadf117c97a3724e06ad4abd2787e6824"),
            (2222372, 0x00045da878000000, "6ba5e9f7e1b310722be3627716cf469be941f7f3e39a4c3bcefea492ec31ee56"),
            (2538687, 0x00198a8179900000, "8687937412f654b5cfe4a82b08f28393a0c040f77c6f95e26742c2fc4254bfde"),
            (609606, 0x001d4e8d17100000, "5da820742ff5feb3369112938d3095785487456f65a8efc4b96dac4be7ebb259"),
            (1205738, 0x000a7204dd600000, "cc70d8fad5472beb031b1aca356bcab86c7368f40faa24fe5f8922c6c268c299"),
            (959742, 0x00183e71e1400000, "4065bdd778f95676c92b38ac265d361f81bff17d76e5d9452cf985a2ea5a4e39"),
            (4036109, 0x001fec043c700000, "b9cf166e75200eb4993fc9b6e22300a6790c75e6b0fc8f3f29b68a752d42f275"),
            (1525894, 0x000b1574b1500000, "2f238180e4ca1f7520a05f3d6059233926341090f9236ce677690c1823eccab3"),
            (1352720, 0x00018965f2e00000, "afd12f13286a3901430de816e62b85cc62468c059295ce5888b76b3af9028d84"),
            (811884, 0x00155628aa100000, "42d0cdb1ee7c48e552705d18e061abb70ae7957027db8ae8db37ec756472a70a"),
            (1282314, 0x001909a0a1400000, "819721c2457426eb4f4c7565050c44c32076a56fa9b4515a1c7796441730eb58"),
            (1318021, 0x001cceb980000000, "842eb53543db55bacac5e25cb91e43cc2e310fe5f9acc1aee86bdf5e91389374"),
            (948640, 0x0011f7a470a00000, "b8e36bf7019bb96ac3fb7867659d2167d9d3b3148c09fe0de45850b8fe577185"),
            (645464, 0x00030ce2d9400000, "5584bd27982191c3329f01ed846bfd266e96548dfa87018f745c33cfc240211d"),
            (533758, 0x0004435c53c00000, "4da778a25b72a9a0d53529eccfe2e5865a789116cb1800f470d8df685a8ab05d"),
            (1128303, 0x0000c48517800000, "08c6b0b38095b348d80300f0be4c5184d2744a17147c2cba5cc4315abf4c048f"),
            (800374, 0x000968473f900000, "820284d2c8fd243429674c996d8eb8d3450cbc32421f43113e980f516282c7bf"),
            (2453512, 0x001e197c92600000, "5fa870ed107c67704258e5e50abe67509fb73562caf77caa843b5f243425d853"),
            (2651975, 0x000ae6c868000000, "181347d2bbec32bef77ad5e9001e6af80f6abcf3576549384d334ee00c1988d8"),
            (237392, 0x0000000000000001, "fcd567f5d866357a8e299fd5b2359bb2c8157c30395229c4e9b0a353944a7978"),
        ];
        let chunk_sizes = ChunkSizes::new(512 * 1024, 1 * 1024 * 1024, 8 * 1024 * 1024);
        let restic: Box<dyn SplitPointFinder> = Box::new(ResticCdc::new(pol, chunk_sizes));
        let mut chunker = ChunkStream::new(File::open("data/test/restic_data1").unwrap(), &restic, chunk_sizes);

        let mut pos = 0;
        let mut count = 0;
        for (expected_length, _, expected_sha) in chunks {
            let chunk = chunker.next().unwrap().unwrap();
            println!("Chunk {} {}", chunk.offset, chunk.length);
            assert_eq!(chunk.offset, pos);
            assert_eq!(chunk.length, expected_length);
            assert_eq!(sha256(&chunk.data), expected_sha);
            pos += chunk.length;
        }
    }
}
