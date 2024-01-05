use crate::rspace::history::history::History;
use crate::rspace::hot_store_action::HotStoreAction;
use crate::rspace::hot_store_trie_action::HotStoreTrieAction;
use crate::rspace::state::rspace_exporter::RSpaceExporter;

// See rspace/src/main/scala/coop/rchain/rspace/history/HistoryRepository.scala
pub trait HistoryRepository<C, P, A, K> {
    fn checkpoint(
        &self,
        actions: Vec<HotStoreAction<C, P, A, K>>,
    ) -> dyn HistoryRepository<C, P, A, K>;

    fn do_checkpoint(
        &self,
        actions: Vec<HotStoreTrieAction<C, P, A, K>>,
    ) -> dyn HistoryRepository<C, P, A, K>;

    fn reset(&self, root: blake3::Hash) -> dyn HistoryRepository<C, P, A, K>;

    fn history(&self) -> dyn History;

    fn exporter(
        &self,
    ) -> dyn RSpaceExporter<
        KeyHash = blake3::Hash,
        NodePath = Vec<(blake3::Hash, Option<u8>)>,
        Value = Vec<u8>,
    >;

    // fn importer(&self) -> dyn RSpacePlusPlusImporter;

    // fn get_history_reader(&self, state_hash: blake3::Hash) -> dyn HistoryReader;

    // def getSerializeC: Serialize[C]

    fn root(&self) -> blake3::Hash;
}

// struct HistoryRepository<C, P, A, K> {}
