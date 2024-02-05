use crate::rspace::hashing::blake3_hash::Blake3Hash;
use crate::rspace::history::roots_store::{RootsStore, RootsStoreInstances};
use crate::rspace::shared::key_value_store::KvStoreError;
use crate::rspace::shared::key_value_typed_store::KeyValueTypedStore;
use crate::rspace::shared::trie_exporter::{TrieExporter, TrieNode};
use crate::rspace::state::rspace_exporter::RSpaceExporterInstance;
use crate::rspace::{
    shared::key_value_store::KeyValueStore, state::rspace_exporter::RSpaceExporter,
};

// See rspace/src/main/scala/coop/rchain/rspace/state/instances/RSpaceExporterStore.scala
pub struct RSpaceExporterStore;

impl RSpaceExporterStore {
    pub fn create(
        history_store: Box<dyn KeyValueStore>,
        value_store: Box<dyn KeyValueStore>,
        roots_store: Box<dyn KeyValueStore>,
    ) -> impl RSpaceExporter<
        KeyHash = Blake3Hash,
        NodePath = Vec<(Blake3Hash, Option<u8>)>,
        Value = Vec<u8>,
    > {
        RSpaceExporterImpl {
            source_history_store: history_store,
            source_value_store: value_store,
            source_roots_store: roots_store,
        }
    }
}

struct RSpaceExporterImpl {
    source_history_store: Box<dyn KeyValueStore>,
    source_value_store: Box<dyn KeyValueStore>,
    source_roots_store: Box<dyn KeyValueStore>,
}

impl RSpaceExporterImpl {
    fn get_items(
        &self,
        store: Box<dyn KeyValueStore>,
        keys: Vec<Blake3Hash>,
    ) -> Result<Vec<(Blake3Hash, <RSpaceExporterImpl as TrieExporter>::Value)>, KvStoreError> {
        let loaded = store.get(keys.iter().map(|key| key.bytes()).collect())?;
        Ok(keys
            .into_iter()
            .zip(loaded.into_iter())
            .filter_map(|(key, value_option)| value_option.map(|value| (key, value)))
            .collect())
    }
}

impl RSpaceExporter for RSpaceExporterImpl {
    fn get_root(&self) -> Blake3Hash {
        let roots_store = RootsStoreInstances::roots_store(self.source_roots_store.clone());
        let maybe_root = roots_store.current_root();
        match maybe_root {
            Some(root) => root,
            None => panic!("RSpace Exporter Store: No root found"),
        }
    }
}

impl TrieExporter for RSpaceExporterImpl {
    type KeyHash = Blake3Hash;

    type NodePath = Vec<(Self::KeyHash, Option<u8>)>;

    type Value = Vec<u8>;

    fn get_nodes(
        &self,
        start_path: Self::NodePath,
        skip: usize,
        take: usize,
    ) -> Vec<TrieNode<Self::KeyHash>> {
        let nodes = RSpaceExporterInstance::traverse_history::<
            Vec<u8>,
            Vec<u8>,
            Box<dyn KeyValueTypedStore<Vec<u8>, Vec<u8>>>,
        >(start_path, skip, take, |store, key| store.get_one(key));
        nodes
    }

    fn get_history_items(
        &self,
        keys: Vec<Blake3Hash>,
    ) -> Result<Vec<(Self::KeyHash, Self::Value)>, KvStoreError> {
        self.get_items(self.source_history_store.clone(), keys)
    }

    fn get_data_items(
        &self,
        keys: Vec<Blake3Hash>,
    ) -> Result<Vec<(Self::KeyHash, Self::Value)>, KvStoreError> {
        self.get_items(self.source_value_store.clone(), keys)
    }
}
