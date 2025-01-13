mod index;

use crate::index as orio_index;

use std::fs::File;
use std::path::PathBuf;
use async_std::prelude::*;

use dirs;
use trueblocks::address::address_from_string;
use trueblocks::bloom::Bloom;
//use trueblocks::index::Index;


#[async_std::main]
async fn main() -> std::io::Result<()> {
    // Open the file 4945
    // let path =
    //     PathBuf::from("/home/yabirgb/Downloads/QmQfn7HkkyjiipBMYvnoQExp7G26NVv17a1pJZyPGpVuf6");
    // let bloom = bloom::Bloom::read_from_file(path).unwrap();
    // // E6c968B7d1b0f5FB6810036d6922aF3372Ffef11
    // // https://etherscan.io/tx/0x99e071789fd914f243fd4ba1fa88852f0429a50eae32a301c1fb4ff4adac0feb
    let raw_addr = "c37b40ABdB939635068d3c5f13E7faF686F03B65".to_string();
    // // non existing one
    // // let raw_addr = "1C5ABce0cAf0f92CF8b083c1b3e8bdda9AD24249".to_string();
    let addr = address_from_string(&raw_addr);
    // println!(
    //     "is 2228e5704B637131A3798A186CAF18366c146f74 in the bloom: {}",
    //     bloom.address_is_member(addr)
    // );
    //println!("{:?}", &bloom.bloom_filters[4].bytes);
    // println!("Checking appearances of 0x{}", raw_addr);
    // let index_path = PathBuf::from("./data/QmePxCpxtCQSDVcGTbQaXNARQjs1Us2WH6tXvixQEqZjCG");
    // let index_file = File::open(index_path.clone()).unwrap();
    // let header = index::HeaderRecord::read_from_file(index_path);
    // println!("{}", header.num_addresses);
    // let mut index = index::Index {
    //     file: index_file,
    //     header: header,
    //     address_table_start: 0,
    //     app_table_start: 0,
    // };

    // for app in index.read_apparences(&addr) {
    //     println!(
    //         "{} - {}; 0x{:x} - 0x{:x}",
    //         app.block, app.tx_index, app.block, app.tx_index
    //     );
    // }

    let config_path = dirs::config_dir().unwrap();
    let index = orio_index::check_index(config_path.clone()).await;

    for chunk in index.chunks[1..index.chunks.len()].into_iter() {
        let bloom_path = config_path.join("orio").join("blooms")
            .join(chunk.range.as_str());
        let bloom = match Bloom::read_from_file(bloom_path) {
                Ok(bloom) => bloom,
                Err(e) => panic!("Could not oppen bloom due to {:?}", e),
        };

        if bloom.address_is_member(addr) {
            println!("Address {:?} found in {}", raw_addr, chunk.range);
        }
    }
    Ok(())
}
