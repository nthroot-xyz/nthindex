use std::fs::File;
use std::io::{Seek, Read};
use std::fmt;

pub type Address = [u8; 20];

// The number of bits in a single BloomByte structure
const BLOOM_WIDTH_IN_BITS: usize = 1048576;
// The number of bytes in a single BloomByte structure
const BLOOM_WIDTH_IN_BYTES: usize = BLOOM_WIDTH_IN_BITS / 8;
// The maximum number of addresses to add to a bloomBytes before creating a new one
const MAX_ADDRS_IN_BLOOM: u16 = 50000;

#[derive(Debug)]
pub struct Header{
    pub magic: u16,
    pub hash: [u8; 32]
}

struct BloomFilter {
    count: usize,
    bytes: [u8; BLOOM_WIDTH_IN_BYTES],
}

struct Bloom {
    bloom_filters: Box<Vec<BloomFilter>>,
    header: Header,
}

fn address_from_string(addr: &String) -> Address {
    let mut address_bytes: [u8; 20] = [0; 20];
    hex::decode_to_slice(addr, &mut address_bytes).expect("Decoding failed");
    address_bytes
}



/// addressToBits extracts five bits from a 20-byte address to determine its presence in the bloom filter.
/// It divides the address into five 4-byte segments, converts each to a 32-bit integer, and then takes the modulo
/// with the bloom array item width.
fn address_to_bits(address: Address) -> [u32; 5] {
    let mut output = [0; 5];
    for i in (0..20).step_by(4) {
        let bytes = &address[i..i+4];
        output[i / 4] = u32::from_be_bytes(bytes.try_into().unwrap()) % BLOOM_WIDTH_IN_BITS as u32;
    }

    // let mut i: usize = 0;
    // let mut cnt = 0;

    // while i < address.len() {
    //     output[cnt] = u32::from_be_bytes(address[i..i+4].try_into().unwrap()) % BLOOM_WIDTH_IN_BITS as u32;
    //     cnt += 1;
    //     i += 4;
    // }

    output
}


impl Header {
    pub fn read(file: &mut File) -> Header {
        let mut magic_buffer = [0; 2];
        let mut hash_buffer = [0; 32];

        file.read_exact(&mut magic_buffer).unwrap();
        file.read_exact(&mut hash_buffer).unwrap();
        Header{
            magic: u16::from_be_bytes(magic_buffer),
            hash: hash_buffer,
        }
    }
}

impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Header(magic={}, hash=0x{})", self.magic, hex::encode(self.hash))
    }
}


impl fmt::Display for BloomFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BloomFilter(count={})", self.count)
    }
}

impl BloomFilter {
    fn is_bit_in(&self, bit: u32) -> bool{
        let index = BLOOM_WIDTH_IN_BYTES - (bit / 8) as usize - 1;
        let res = self.bytes[index] & (1 << (bit % 8));
        res != 0
    }
}

impl Bloom {
    fn address_is_member(&self, address: Address) -> bool {
        let address_bits = address_to_bits(address);
        for bloom in self.bloom_filters.iter() {
            if self.is_bloom_in(bloom, address_bits){
                return true
            }
        }

        false
    }

    fn is_bloom_in(&self, bloom: &BloomFilter, add: [u32; 5]) -> bool {
        for bit in add {
            if !bloom.is_bit_in(bit) {
                return false
            }
        }

        return true
    }
}

fn main() -> std::io::Result<()> {
    // Open the file 4945
    let mut file = File::open("/Users/yabirgb/Downloads/QmQfn7HkkyjiipBMYvnoQExp7G26NVv17a1pJZyPGpVuf6")?;
    file.rewind().unwrap();
    let header = Header::read(&mut file);
    println!("{}", header);

    // read number of blooms
    let mut count_buffer = [0; 4];  // count is a uint32
    file.read_exact(&mut count_buffer).unwrap();
    let count = u32::from_le_bytes(count_buffer);
    println!("Total number of bloom filters: {}", count);

    // read blooms
    let mut blooms: Vec<BloomFilter> = Vec::new();

    // allocate once for all the blooms
    let mut bloom_count: [u8; 4] = [0; 4];
    let mut bloom = [0; BLOOM_WIDTH_IN_BYTES];
    for _ in 0..count as usize {
        file.read_exact(&mut bloom_count).unwrap();
        file.read_exact(&mut bloom).unwrap();

        blooms.push(BloomFilter{
            count: u32::from_le_bytes(bloom_count) as usize,
            bytes: bloom.clone(),
        });
    }

    let bloom = Bloom{bloom_filters: Box::new(blooms), header};
    // E6c968B7d1b0f5FB6810036d6922aF3372Ffef11
    // https://etherscan.io/tx/0x99e071789fd914f243fd4ba1fa88852f0429a50eae32a301c1fb4ff4adac0feb
    let raw_addr = "2228e5704B637131A3798A186CAF18366c146f74".to_string();
    // non existing one
    // let raw_addr = "1C5ABce0cAf0f92CF8b083c1b3e8bdda9AD24249".to_string();
    let addr = address_from_string(&raw_addr);
    println!("{:?}", address_to_bits(addr));
    println!("is 2228e5704B637131A3798A186CAF18366c146f74 in the bloom: {}", bloom.address_is_member(addr));
    //println!("{:?}", &bloom.bloom_filters[4].bytes);
    Ok(())
}
