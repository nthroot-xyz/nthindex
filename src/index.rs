use std::{fs::File, io::Read, path::PathBuf};

use crate::address::Address;

const HEADER_WIDTH: u64 = 44;

struct HeaderRecord {
    magic: u32,
    version: [u8; 32],
    num_addresses: u32,
    num_appearances: u32,
}

impl HeaderRecord {
    pub fn read_from_file(path: PathBuf) -> HeaderRecord {
        let mut magic_buffer = [0; 4];
        let mut hash_buffer = [0; 32];
        let mut address_count = [0; 4];
        let mut appearances_count = [0; 4];

        let mut file = File::open(path).unwrap();
        file.read_exact(&mut magic_buffer).unwrap();
        file.read_exact(&mut hash_buffer).unwrap();
        file.read_exact(&mut address_count).unwrap();
        file.read_exact(&mut appearances_count).unwrap();

        HeaderRecord {
            magic: u32::from_le_bytes(magic_buffer),
            version: hash_buffer,
            num_addresses: u32::from_le_bytes(address_count),
            num_appearances: u32::from_le_bytes(appearances_count),
        }
    }
}

struct AddressRecord {
    address: Address,
    offset: u32,
    count: u32,
}

struct AppearanceRecord {
    block: u32,
    tx_index: u32,
}

pub struct Index {
    file: File,
    header: HeaderRecord,
    address_table_start: u64,
    app_table_start: u64,
}
