use std::path::PathBuf;

use async_std::fs::{File, create_dir_all, read_to_string};
use async_std::io::{copy, Cursor, WriteExt};
use serde_json;

use trueblocks::index::index_file::IndexFile;

const ORIO_FOLDER_NAME: &str = "orio";
const LAST_HASH: &str = "QmUBS83qjRmXmSgEvZADVv2ch47137jkgNbqfVVxQep5Y1";  // harcoded for now


pub async fn download_blooms(base_path: PathBuf, index: &IndexFile) {
    let blooms_folder = base_path.join("blooms");
    if blooms_folder.join(index.chunks.last().unwrap().range.as_str()).exists() {
        return
    }

    for bloom_file in index.chunks.iter() {
        let path = blooms_folder.join(bloom_file.range.as_str());
        println!("Downloading {} into {:?}", bloom_file.bloom_hash, path);
        let mut file = File::create(path).await.unwrap();

        let file_content = surf::get(format!("https://ipfs.io/ipfs/{}", bloom_file.bloom_hash).as_str())
            .await
            .unwrap()
            .body_bytes().await.unwrap();
        let mut content = Cursor::new(file_content);
        copy(&mut content, &mut file).await.unwrap();
    };
}

pub async fn check_index(config_dir: PathBuf) -> IndexFile{
    let orio_path: PathBuf = config_dir.join(ORIO_FOLDER_NAME);
    create_dir_all(orio_path.clone()).await.unwrap();
    let index_file_path = orio_path.join(format!("index_{}.json", LAST_HASH));

    let body: String;
    if !index_file_path.exists() {
        body = surf::get(format!("https://ipfs.io/ipfs/{}", LAST_HASH).as_str())
            .await
            .unwrap()
            .body_string().await.unwrap();

        let mut file = File::create(index_file_path).await.unwrap();
        file.write_all(body.as_bytes()).await.unwrap();
    } else {
        body = read_to_string(index_file_path).await.unwrap();
    }

    let index_file: IndexFile = match serde_json::from_str(body.as_str()) {
        Ok(file) => file,
        Err(e) => {panic!("Failed to read index due to {}", e)},
    };

    download_blooms(orio_path, &index_file).await;
    index_file
}
