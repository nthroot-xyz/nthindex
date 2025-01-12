use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

const ORIO_FOLDER_NAME: &str = "orio";
const LAST_HASH: &str = "QmUBS83qjRmXmSgEvZADVv2ch47137jkgNbqfVVxQep5Y1";  // harcoded for now


pub fn download_blooms(index: &String) {
    // TODO: I'm on this
}

pub fn check_index(config_dir: PathBuf) -> String{
    let mut orio_path: PathBuf = config_dir.clone();
    orio_path.push(ORIO_FOLDER_NAME);
    fs::create_dir_all(orio_path.clone()).unwrap();
    let mut index_file_path = orio_path.clone();
    index_file_path.push(format!("index_{}.json", LAST_HASH));

    if !index_file_path.exists() {
        let body: String = ureq::get(format!("https://ipfs.io/ipfs/{}", LAST_HASH).as_str())
            .call()
            .unwrap()
            .into_string().unwrap();

        let mut file = File::create(index_file_path).unwrap();
        file.write_all(body.as_bytes()).unwrap();
        return body;
    } else {
        return fs::read_to_string(index_file_path).unwrap();
    }
}
