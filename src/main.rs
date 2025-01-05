use std::path::PathBuf;

use missuri::address::address_from_string;
use missuri::bloom;

fn main() -> std::io::Result<()> {
    // Open the file 4945
    let path =
        PathBuf::from("/home/yabirgb/Downloads/QmQfn7HkkyjiipBMYvnoQExp7G26NVv17a1pJZyPGpVuf6");
    let bloom = bloom::Bloom::read_from_file(path).unwrap();
    // E6c968B7d1b0f5FB6810036d6922aF3372Ffef11
    // https://etherscan.io/tx/0x99e071789fd914f243fd4ba1fa88852f0429a50eae32a301c1fb4ff4adac0feb
    let raw_addr = "2228e5704B637131A3798A186CAF18366c146f74".to_string();
    // non existing one
    // let raw_addr = "1C5ABce0cAf0f92CF8b083c1b3e8bdda9AD24249".to_string();
    let addr = address_from_string(&raw_addr);
    println!(
        "is 2228e5704B637131A3798A186CAF18366c146f74 in the bloom: {}",
        bloom.address_is_member(addr)
    );
    //println!("{:?}", &bloom.bloom_filters[4].bytes);
    Ok(())
}
