use thiserror::Error;

#[derive(Error, Debug)]
pub enum TrueblocksError {
    #[error("Error with bloom filter")]
    BloomFilterError,
}
