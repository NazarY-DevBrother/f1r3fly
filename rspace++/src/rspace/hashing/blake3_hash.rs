use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

// See src/firefly/f1r3fly/rspace/src/main/scala/coop/rchain/rspace/hashing/Blake2b256Hash.scala
// The 'Hash' macro is needed here for test util function 'check_same_elements'
#[derive(Eq, Clone, Debug, Serialize, Deserialize, Hash)]
pub struct Blake3Hash(pub Vec<u8>);

impl Blake3Hash {
    pub fn new(data: &[u8]) -> Self {
        let hash = blake3::hash(data);
        Blake3Hash(hash.as_bytes().to_vec())
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Blake3Hash(bytes)
    }

    pub fn bytes(&self) -> Vec<u8> {
        self.0.clone()
    }
}

impl Ord for Blake3Hash {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for Blake3Hash {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Blake3Hash {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
