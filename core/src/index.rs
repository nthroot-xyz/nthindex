pub mod index_file;

use std::io::{self, Read, Seek, SeekFrom};
use std::{fs::File, path::PathBuf};

use crate::address::Address;

const HEADER_WIDTH: usize = 44;
const ADDRESS_RECORD_WIDTH: usize = 28;
const APP_RECORD_WIDTH: usize = 8;

pub struct HeaderRecord {
    pub magic: u32,
    pub version: [u8; 32],
    pub num_addresses: u32,
    pub num_appearances: u32,
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

pub struct AddressRecord {
    pub address: Address,
    pub offset: u32,
    pub count: u32,
}

#[derive(Debug)]
pub struct AppearanceRecord {
    pub block: u32,
    pub tx_index: u32,
}

pub struct Index {
    pub file: File,
    pub header: HeaderRecord,
    pub address_table_start: u64,
    pub app_table_start: u64,
}

impl Index {
    fn binary_search(&mut self, target_address: &Address) -> io::Result<usize> {
        let address_count = self.header.num_addresses as usize;

        // we don't know the size of the slice at comp time so we need to use
        // a vec. binary_search_by is not implemented for ranges.
        let pos = (0..address_count)
            .collect::<Vec<usize>>()
            .binary_search_by(|&pos| {
                // Handle edge cases
                if pos == 0 {
                    return std::cmp::Ordering::Less;
                }
                if pos == address_count {
                    return std::cmp::Ordering::Equal;
                }

                // Calculate read position
                let read_location = (HEADER_WIDTH + pos * ADDRESS_RECORD_WIDTH) as u64;
                if let Err(e) = self.file.seek(SeekFrom::Start(read_location)) {
                    eprintln!("Seek error: {}", e);
                    return std::cmp::Ordering::Less;
                }

                // Read address record
                let mut addr_record = [0; 32];
                match self.file.read_exact(&mut addr_record) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("Read error: {}", e);
                        return std::cmp::Ordering::Less;
                    }
                }

                // Compare addresses
                addr_record.as_slice().cmp(target_address)
            });

        match pos {
            Ok(index) => Ok(index),
            Err(index) => Ok(index), // Return insertion point if not found
        }
    }

    pub fn search_for_address_record(&mut self, address: &Address) -> AddressRecord {
        let pos = self
            .binary_search(address)
            .expect("Address not found in index");
        let read_location = HEADER_WIDTH + pos * ADDRESS_RECORD_WIDTH;
        self.file
            .seek(SeekFrom::Start(read_location as u64))
            .unwrap();

        let mut address_buf = [0; 20];
        let mut offset_buf = [0; 4];
        let mut count_buf = [0; 4];

        self.file.read_exact(&mut address_buf).unwrap();
        self.file.read_exact(&mut offset_buf).unwrap();
        self.file.read_exact(&mut count_buf).unwrap();

        AddressRecord {
            address: address_buf,
            offset: u32::from_le_bytes(offset_buf),
            count: u32::from_le_bytes(count_buf),
        }
    }

    pub fn read_apparences(&mut self, address: &Address) -> Vec<AppearanceRecord> {
        let mut results: Vec<AppearanceRecord> = Vec::new();
        let record = self.search_for_address_record(address);

        let read_location = (HEADER_WIDTH
            + ADDRESS_RECORD_WIDTH * self.header.num_addresses as usize
            + APP_RECORD_WIDTH * record.offset as usize) as u64;

        self.file.seek(SeekFrom::Start(read_location)).unwrap();

        let mut block_buf = [0; 4];
        let mut index_buf = [0; 4];

        for _ in 0..record.count {
            self.file.read_exact(&mut block_buf).unwrap();
            self.file.read_exact(&mut index_buf).unwrap();
            results.push(AppearanceRecord {
                block: u32::from_le_bytes(block_buf),
                tx_index: u32::from_le_bytes(index_buf),
            });
        }
        results
    }
}
