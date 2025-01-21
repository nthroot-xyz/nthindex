use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IndexFileConfig {
    pub apps_per_chunk: u64,
    pub snap_to_grid: u64,
    pub first_snap: u64,
    pub unripe_dist: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IndexFileChunk {
    pub bloom_hash: String,
    pub bloom_size: usize,
    pub index_hash: String,
    pub index_size: usize,
    pub range: String,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct IndexFile {
    pub version: String,
    pub chain: String,
    pub specification: String,
    pub config: IndexFileConfig,
    pub chunks: Vec<IndexFileChunk>,
}
