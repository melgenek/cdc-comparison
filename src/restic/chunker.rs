use crate::chunk_sizes::ChunkSizes;
use crate::chunker::Chunker;
use crate::restic::polynomial::Pol;
use crate::util::logarithm2;

const WINDOW_SIZE: usize = 64;

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

    fn append_byte(digest: Pol, b: u8, pol: Pol) -> Pol {
        let digest = digest << 8;
        let digest = digest | Pol::from(b);
        digest % pol
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

    fn update_digest(&self, digest: u64, b: u8) -> u64 {
        let index = digest >> self.pol_shift;
        let digest = digest << 8;
        let digest = digest | (b as u64);
        let digest = digest ^ self.tables.mods[index as usize].value();
        digest
    }
}

impl Chunker for ResticCdc {
    fn find_split_point(&self, buf: &[u8], chunk_sizes: &ChunkSizes) -> usize {
        let mut window: [u8; WINDOW_SIZE] = [0; WINDOW_SIZE];
        let mut oldest_idx: usize = 0;

        let mut slide = |digest: u64, new_byte: u8| -> u64 {
            let old_byte = window[oldest_idx];
            window[oldest_idx] = new_byte;
            oldest_idx = (oldest_idx + 1) % WINDOW_SIZE;
            let digest = digest ^ self.tables.out[old_byte as usize].value();
            let digest = self.update_digest(digest, new_byte);
            digest
        };

        let mut digest = slide(0, 1);
        let mut i = chunk_sizes.min_size() - WINDOW_SIZE;

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
    use std::io::Read;

    use crate::chunk_sizes::ChunkSizes;
    use crate::chunk_stream::ChunkStream;
    use crate::chunker::Chunker;
    use crate::restic::chunker::ResticCdc;
    use crate::restic::polynomial::Pol;
    use crate::util::sha256;
    use crate::{KB, MB};

    #[test]
    pub fn should_split_random_data1() -> std::io::Result<()> {
        let chunks = vec![
            (2163460, "4b94cb2cf293855ea43bf766731c74969b91aa6bf3c078719aabdd19860d590d"),
            (643703, "5727a63c0964f365ab8ed2ccf604912f2ea7be29759a2b53ede4d6841e397407"),
            (1528956, "a73759636a1e7a2758767791c69e81b69fb49236c6929e5d1b654e06e37674ba"),
            (1955808, "c955fb059409b25f07e5ae09defbbc2aadf117c97a3724e06ad4abd2787e6824"),
            (2222372, "6ba5e9f7e1b310722be3627716cf469be941f7f3e39a4c3bcefea492ec31ee56"),
            (2538687, "8687937412f654b5cfe4a82b08f28393a0c040f77c6f95e26742c2fc4254bfde"),
            (609606, "5da820742ff5feb3369112938d3095785487456f65a8efc4b96dac4be7ebb259"),
            (1205738, "cc70d8fad5472beb031b1aca356bcab86c7368f40faa24fe5f8922c6c268c299"),
            (959742, "4065bdd778f95676c92b38ac265d361f81bff17d76e5d9452cf985a2ea5a4e39"),
            (4036109, "b9cf166e75200eb4993fc9b6e22300a6790c75e6b0fc8f3f29b68a752d42f275"),
            (1525894, "2f238180e4ca1f7520a05f3d6059233926341090f9236ce677690c1823eccab3"),
            (1352720, "afd12f13286a3901430de816e62b85cc62468c059295ce5888b76b3af9028d84"),
            (811884, "42d0cdb1ee7c48e552705d18e061abb70ae7957027db8ae8db37ec756472a70a"),
            (1282314, "819721c2457426eb4f4c7565050c44c32076a56fa9b4515a1c7796441730eb58"),
            (1318021, "842eb53543db55bacac5e25cb91e43cc2e310fe5f9acc1aee86bdf5e91389374"),
            (948640, "b8e36bf7019bb96ac3fb7867659d2167d9d3b3148c09fe0de45850b8fe577185"),
            (645464, "5584bd27982191c3329f01ed846bfd266e96548dfa87018f745c33cfc240211d"),
            (533758, "4da778a25b72a9a0d53529eccfe2e5865a789116cb1800f470d8df685a8ab05d"),
            (1128303, "08c6b0b38095b348d80300f0be4c5184d2744a17147c2cba5cc4315abf4c048f"),
            (800374, "820284d2c8fd243429674c996d8eb8d3450cbc32421f43113e980f516282c7bf"),
            (2453512, "5fa870ed107c67704258e5e50abe67509fb73562caf77caa843b5f243425d853"),
            (2651975, "181347d2bbec32bef77ad5e9001e6af80f6abcf3576549384d334ee00c1988d8"),
            (237392, "fcd567f5d866357a8e299fd5b2359bb2c8157c30395229c4e9b0a353944a7978"),
        ];
        let chunk_sizes = ChunkSizes::new(512 * KB, 1 * MB, 8 * MB);

        verify_chunks_for_data(File::open("data/test/restic_data1")?, chunks, chunk_sizes);
        Ok(())
    }

    #[test]
    pub fn should_split_random_data2() -> std::io::Result<()> {
        let chunks = vec![
            (1491586, "4c008237df602048039287427171cef568a6cb965d1b5ca28dc80504a24bb061"),
            (671874, "fa8a42321b90c3d4ce9dd850562b2fd0c0fe4bdd26cf01a24f22046a224225d3"),
            (643703, "5727a63c0964f365ab8ed2ccf604912f2ea7be29759a2b53ede4d6841e397407"),
            (1284146, "16d04cafecbeae9eaedd49da14c7ad7cdc2b1cc8569e5c16c32c9fb045aa899a"),
            (823366, "48662c118514817825ad4761e8e2e5f28f9bd8281b07e95dcafc6d02e0aa45c3"),
            (810134, "f629581aa05562f97f2c359890734c8574c5575da32f9289c5ba70bfd05f3f46"),
            (567118, "d4f0797c56c60d01bac33bfd49957a4816b6c067fc155b026de8a214cab4d70a"),
            (821315, "8ebd0fd5db0293bd19140da936eb8b1bbd3cd6ffbec487385b956790014751ca"),
            (1401057, "001360af59adf4871ef138cfa2bb49007e86edaf5ac2d6f0b3d3014510991848"),
            (2311122, "8276d489b566086d9da95dc5c5fe6fc7d72646dd3308ced6b5b6ddb8595f0aa1"),
            (608723, "518db33ba6a79d4f3720946f3785c05b9611082586d47ea58390fc2f6de9449e"),
            (980456, "0121b1690738395e15fecba1410cd0bf13fde02225160cad148829f77e7b6c99"),
            (1140278, "28ca7c74804b5075d4f5eeb11f0845d99f62e8ea3a42b9a05c7bd5f2fca619dd"),
            (2015542, "6fe8291f427d48650a5f0f944305d3a2dbc649bd401d2655fc0bdd42e890ca5a"),
            (904752, "62af1f1eb3f588d18aff28473303cc4731fc3cafcc52ce818fee3c4c2820854d"),
            (713072, "4bda9dc2e3031d004d87a5cc93fe5207c4b0843186481b8f31597dc6ffa1496c"),
            (675937, "5299c8c5acec1b90bb020cd75718aab5e12abb9bf66291465fd10e6a823a8b4a"),
            (1525894, "2f238180e4ca1f7520a05f3d6059233926341090f9236ce677690c1823eccab3"),
            (1352720, "afd12f13286a3901430de816e62b85cc62468c059295ce5888b76b3af9028d84"),
            (811884, "42d0cdb1ee7c48e552705d18e061abb70ae7957027db8ae8db37ec756472a70a"),
            (1282314, "819721c2457426eb4f4c7565050c44c32076a56fa9b4515a1c7796441730eb58"),
            (1093738, "5dddfa7a241b68f65d267744bdb082ee865f3c2f0d8b946ea0ee47868a01bbff"),
            (962003, "0cb5c9ebba196b441c715c8d805f6e7143a81cd5b0d2c65c6aacf59ca9124af9"),
            (856384, "7734b206d46f3f387e8661e81edf5b1a91ea681867beb5831c18aaa86632d7fb"),
            (533758, "4da778a25b72a9a0d53529eccfe2e5865a789116cb1800f470d8df685a8ab05d"),
            (1128303, "08c6b0b38095b348d80300f0be4c5184d2744a17147c2cba5cc4315abf4c048f"),
            (800374, "820284d2c8fd243429674c996d8eb8d3450cbc32421f43113e980f516282c7bf"),
            (2453512, "5fa870ed107c67704258e5e50abe67509fb73562caf77caa843b5f243425d853"),
            (665901, "deceec26163842fdef6560311c69bf8a9871a56e16d719e2c4b7e4d668ceb61f"),
            (1986074, "64cd64bf3c3bc389eb20df8310f0427d1c36ab2eaaf09e346bfa7f0453fc1a18"),
            (237392, "fcd567f5d866357a8e299fd5b2359bb2c8157c30395229c4e9b0a353944a7978"),
        ];
        let chunk_sizes = ChunkSizes::new(512 * KB, 512 * KB, 8 * MB);

        verify_chunks_for_data(File::open("data/test/restic_data1")?, chunks, chunk_sizes);
        Ok(())
    }

    #[test]
    pub fn should_split_min_multiple() {
        let min_size = 512 * KB;
        let chunks = vec![
            (min_size, "07854d2fef297a06ba81685e660c332de36d5d18d546927d30daad6d7fda1541"),
            (min_size, "07854d2fef297a06ba81685e660c332de36d5d18d546927d30daad6d7fda1541"),
            (min_size, "07854d2fef297a06ba81685e660c332de36d5d18d546927d30daad6d7fda1541"),
            (min_size, "07854d2fef297a06ba81685e660c332de36d5d18d546927d30daad6d7fda1541"),
        ];
        let chunk_sizes = ChunkSizes::new(min_size, 1 * MB, 8 * MB);
        let zeros = vec![0u8; chunks.len() * min_size];

        verify_chunks_for_data(zeros.as_slice(), chunks, chunk_sizes);
    }

    fn verify_chunks_for_data<R: Read>(input: R, chunks: Vec<(usize, &str)>, chunk_sizes: ChunkSizes) {
        let expected_chunks_count = chunks.len();
        let pol = Pol::from(0x3DA3358B4DC173 as u64);
        let restic: Box<dyn Chunker> = Box::new(ResticCdc::new(pol, chunk_sizes));
        let mut chunker = ChunkStream::new(input, restic, chunk_sizes);

        let mut offset = 0;
        let mut chunk_count = 0;
        for (expected_length, expected_sha) in chunks {
            let chunk = chunker.next().unwrap().unwrap();
            assert_eq!(chunk.offset, offset);
            assert_eq!(chunk.length, expected_length);
            assert_eq!(sha256(&chunk.data), expected_sha);
            offset += chunk.length;
            chunk_count += 1;
        }
        assert_eq!(expected_chunks_count, chunk_count);
    }
}
