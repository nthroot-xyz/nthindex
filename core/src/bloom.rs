use crate::errors::TrueblocksError;
use std::fmt;
use std::fs::File;
use std::io::{Read, Seek};
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;

use crate::address::Address;

// The number of bits in a single BloomByte structure
const BLOOM_WIDTH_IN_BITS: usize = 1048576;
// The number of bytes in a single BloomByte structure
const BLOOM_WIDTH_IN_BYTES: usize = BLOOM_WIDTH_IN_BITS / 8;
// The maximum number of addresses to add to a bloomBytes before creating a new one
const _MAX_ADDRS_IN_BLOOM: u16 = 50000;

#[derive(Debug)]
pub struct BloomHeader {
    pub magic: u16,
    pub hash: [u8; 32],
}

pub struct BloomFilter {
    count: usize,
    bytes: [u8; BLOOM_WIDTH_IN_BYTES],
}

pub struct Bloom {
    pub bloom_filters: Box<Vec<BloomFilter>>,
    pub header: BloomHeader,
}

impl BloomHeader {
    pub fn read(file: &mut File) -> BloomHeader {
        let mut magic_buffer = [0; 2];
        let mut hash_buffer = [0; 32];

        file.read_exact(&mut magic_buffer).unwrap();
        file.read_exact(&mut hash_buffer).unwrap();
        BloomHeader {
            magic: u16::from_be_bytes(magic_buffer),
            hash: hash_buffer,
        }
    }
}

impl fmt::Display for BloomHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Header(magic={}, hash=0x{})",
            self.magic,
            hex::encode(self.hash)
        )
    }
}

impl fmt::Display for BloomFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BloomFilter(count={})", self.count)
    }
}

impl BloomFilter {
    fn is_bit_in(&self, bit: u32) -> bool {
        let index = BLOOM_WIDTH_IN_BYTES - (bit / 8) as usize - 1;
        let res = self.bytes[index] & (1 << (bit % 8));
        res != 0
    }
}

impl Bloom {
    pub fn address_is_member(&self, address: Address) -> bool {
        let address_bits = address_to_bits(address);
        for bloom in self.bloom_filters.iter() {
            if self.bloom_contains_address(bloom, address_bits) {
                return true;
            }
        }

        false
    }

    fn bloom_contains_address(&self, bloom: &BloomFilter, add: [u32; 5]) -> bool {
        for bit in add {
            if !bloom.is_bit_in(bit) {
                return false;
            }
        }

        return true;
    }

    pub fn read_from_file(path: PathBuf) -> Result<Bloom, TrueblocksError> {
        let mut file = File::open(path).map_err(|e| TrueblocksError::BloomFilterError(e.to_string()))?;
        file.rewind().unwrap();
        if file.metadata().unwrap().size() == 0 {
            return Err(TrueblocksError::BloomFilterError("Empty file".to_string()));
        }

        let header = BloomHeader::read(&mut file);

        // read number of blooms
        let mut count_buffer = [0; 4]; // count is a uint32
        file.read_exact(&mut count_buffer).unwrap();
        let count = u32::from_le_bytes(count_buffer);

        // read blooms
        let mut blooms: Vec<BloomFilter> = Vec::new();

        // allocate once for all the blooms
        let mut bloom_count: [u8; 4] = [0; 4];
        let mut bloom = [0; BLOOM_WIDTH_IN_BYTES];
        for _ in 0..count as usize {
            file.read_exact(&mut bloom_count).unwrap();
            if let Err(e) = file.read_exact(&mut bloom) {
                return Err(TrueblocksError::BloomFilterError(e.to_string()));
            }

            blooms.push(BloomFilter {
                count: u32::from_le_bytes(bloom_count) as usize,
                bytes: bloom.clone(),
            });
        }

        Ok(Bloom {
            bloom_filters: Box::new(blooms),
            header,
        })
    }
}

/// addressToBits extracts five bits from a 20-byte address to determine its presence in the bloom filter.
/// It divides the address into five 4-byte segments, converts each to a 32-bit integer, and then takes the modulo
/// with the bloom array item width.
fn address_to_bits(address: Address) -> [u32; 5] {
    let mut output = [0; 5];
    for i in (0..20).step_by(4) {
        let bytes = &address[i..i + 4];
        output[i / 4] = u32::from_be_bytes(bytes.try_into().unwrap()) % BLOOM_WIDTH_IN_BITS as u32;
    }

    output
}
