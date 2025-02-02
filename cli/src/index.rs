use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use futures::future::join_all;
use futures::StreamExt;

use tokio::fs::{self, File, create_dir_all, read_to_string, remove_file};
use tokio::io::AsyncWriteExt;  // for write all
use tokio::task::JoinHandle;
use serde_json;

use trueblocks::index::index_file::{IndexFile, IndexFileChunk};

const ORIO_FOLDER_NAME: &str = "orio";
const LAST_HASH: &str = "QmUBS83qjRmXmSgEvZADVv2ch47137jkgNbqfVVxQep5Y1";  // harcoded for now


pub async fn download_blooms(base_path: PathBuf, index: &IndexFile) {
    let blooms_folder = base_path.join("blooms");

    for bloom_chunk in index.chunks.chunks(20) {
        let mut current_group = Vec::new();

        for bloom_file in bloom_chunk {
            let bloom_hash = bloom_file.bloom_hash.clone();
            let range = bloom_file.range.clone();
            let blooms_folder = blooms_folder.clone();

            let handle = tokio::spawn(async move {
                let path = blooms_folder.join(range.as_str());

                if path.exists(){
                    // check the size and if it isn't 0 continue
                    if fs::metadata(path.clone()).await.unwrap().size() != 0 {
                        return
                    }
                    // otherwise remove the file and download it again
                    remove_file(path.clone()).await.unwrap();
                }
                println!("Downloading {} into {:?}", bloom_hash, path);
                let mut file = File::create(path).await.unwrap();
                let mut file_content = reqwest::get(format!("https://ipfs.io/ipfs/{}", bloom_hash).as_str())
                    .await
                    .unwrap()
                    .bytes_stream();

                while let Some(item) = file_content.next().await {
                    tokio::io::copy(&mut item.unwrap().as_ref(), &mut file).await.unwrap();
                }
            });

            current_group.push(handle);
        }

        join_all(current_group).await;
    };

}

pub async fn check_index(config_dir: PathBuf) -> IndexFile{
    let orio_path: PathBuf = config_dir.join(ORIO_FOLDER_NAME);
    create_dir_all(orio_path.clone()).await.unwrap();
    let index_file_path = orio_path.join(format!("index_{}.json", LAST_HASH));

    let body: String;
    if !index_file_path.exists() {
        body = reqwest::get(format!("https://ipfs.io/ipfs/{}", LAST_HASH).as_str())
            .await
            .unwrap()
            .json().await.unwrap();

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

pub async fn download_chunk_index(
    orio_path: PathBuf,
    chunk: &IndexFileChunk,
) -> Result<PathBuf, PathBuf> {
    let index_folder = orio_path.join("index");
    let file_path = index_folder.join(chunk.range.clone());
    if file_path.exists(){
        return Ok(file_path)
    }

    let mut body = reqwest::get(format!("https://ipfs.io/ipfs/{}", chunk.index_hash).as_str())
        .await
        .unwrap()
        .bytes_stream();

    let mut file = File::create(file_path.clone()).await.unwrap();
    while let Some(item) = body.next().await {
        tokio::io::copy(&mut item.unwrap().as_ref(), &mut file).await.unwrap();
    }

    return Ok(file_path);

}