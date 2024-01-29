use crate::rspace::hashing::blake3_hash::Blake3Hash;

// See rspace/src/main/scala/coop/rchain/rspace/history/HistoryAction.scala
type KeyPath = Vec<u8>;

pub enum HistoryAction {
    Insert(InsertAction),
    Delete(DeleteAction),
}

trait HistoryActionTrait {
    fn key(&self) -> &KeyPath;
}

pub struct InsertAction {
    pub key: KeyPath,
    pub hash: Blake3Hash,
}

impl HistoryActionTrait for InsertAction {
    fn key(&self) -> &KeyPath {
        &self.key
    }
}

pub struct DeleteAction {
    pub key: KeyPath,
}

impl HistoryActionTrait for DeleteAction {
    fn key(&self) -> &KeyPath {
        &self.key
    }
}
