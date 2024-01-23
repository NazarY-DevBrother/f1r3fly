use crate::rspace::history::history::History;
use crate::rspace::history::history_action::HistoryAction;
use crate::rspace::history::radix_tree::{hash_node, EmptyNode, Node, RadixTreeImpl};
use crate::rspace::shared::key_value_store::{KeyValueStore, KeyValueStoreOps};
use crate::rspace::shared::key_value_typed_store::KeyValueTypedStore;
use bytes::Bytes;
use dashmap::DashMap;
use std::sync::Arc;
use std::sync::Mutex;

// See rspace/src/main/scala/coop/rchain/rspace/history/instances/RadixHistory.scala
pub struct RadixHistory {
    root_hash: blake3::Hash,
    root_node: Node,
    imple: RadixTreeImpl,
    store: Arc<Mutex<Box<dyn KeyValueTypedStore<Vec<u8>, Vec<u8>> + Sync>>>,
}

pub struct EmptyRootHash {
    pub hash: blake3::Hash,
}

impl EmptyRootHash {
    pub fn new() -> Self {
        let hash_bytes = hash_node(&EmptyNode::new().node).0;
        let hash_array: [u8; 32] = match hash_bytes.try_into() {
            Ok(array) => array,
            Err(_) => panic!("Radix_History: Expected a Blake3 hash of length 32"),
        };
        let hash = blake3::Hash::from_bytes(hash_array);

        EmptyRootHash { hash }
    }
}

impl RadixHistory {
    pub async fn create(
        root: blake3::Hash,
        store: Arc<Mutex<Box<dyn KeyValueTypedStore<Vec<u8>, Vec<u8>> + Sync>>>,
    ) -> RadixHistory {
        let imple = RadixTreeImpl {
            store: store.clone(),
            cache: DashMap::new(),
        };
        let node = imple.load_node(root.as_bytes().to_vec(), Some(true)).await;

        RadixHistory {
            root_hash: root,
            root_node: node,
            imple,
            store,
        }
    }

    pub fn create_store(
        store: Box<dyn KeyValueStore>,
    ) -> Arc<Mutex<Box<dyn KeyValueTypedStore<Vec<u8>, Vec<u8>> + Sync>>> {
        Arc::new(Mutex::new(Box::new(KeyValueStoreOps::to_typed_store::<Vec<u8>, Vec<u8>>(store))))
    }
}

impl History for RadixHistory {
    fn read(&self, key: Bytes) -> Option<Bytes> {
        todo!()
    }

    fn process(&self, actions: Vec<HistoryAction>) -> Box<dyn History> {
        todo!()
    }

    fn root(&self) -> blake3::Hash {
        todo!()
    }

    fn reset(&self, root: blake3::Hash) -> Box<dyn History> {
        todo!()
    }
}
