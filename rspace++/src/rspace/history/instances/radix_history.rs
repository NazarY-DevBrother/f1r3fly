use crate::rspace::hashing::blake3_hash::Blake3Hash;
use crate::rspace::history::history::History;
use crate::rspace::history::history::HistoryError;
use crate::rspace::history::history_action::HistoryAction;
use crate::rspace::history::history_action::HistoryActionTrait;
use crate::rspace::history::radix_tree::empty_node;
use crate::rspace::history::radix_tree::{hash_node, Node, RadixTreeImpl};
use crate::rspace::shared::key_value_store::{KeyValueStore, KeyValueStoreOps};
use crate::rspace::shared::key_value_typed_store::KeyValueTypedStore;
use crate::rspace::ByteVector;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;

// See rspace/src/main/scala/coop/rchain/rspace/history/instances/RadixHistory.scala
pub struct RadixHistory {
    root_hash: Blake3Hash,
    root_node: Node,
    imple: RadixTreeImpl,
    store: Arc<Mutex<Box<dyn KeyValueTypedStore<ByteVector, ByteVector>>>>,
}

impl RadixHistory {
    pub fn create(
        root: Blake3Hash,
        store: Arc<Mutex<Box<dyn KeyValueTypedStore<ByteVector, ByteVector>>>>,
    ) -> Result<RadixHistory, HistoryError> {
        let imple = RadixTreeImpl::new(store.clone());
        let node = imple.load_node(root.bytes(), Some(true))?;

        // println!("Radix Tree in Radix History create: {:?}", imple.print_tree(&node, vec![]));

        Ok(RadixHistory {
            root_hash: root,
            root_node: node,
            imple,
            store,
        })
    }

    pub fn create_store(
        store: Arc<Mutex<Box<dyn KeyValueStore>>>,
    ) -> Arc<Mutex<Box<dyn KeyValueTypedStore<ByteVector, ByteVector>>>> {
        Arc::new(Mutex::new(Box::new(KeyValueStoreOps::to_typed_store::<ByteVector, ByteVector>(
            store,
        ))))
    }

    pub fn empty_root_node_hash() -> Blake3Hash {
        let node_hash_bytes = hash_node(&empty_node()).0;
        // let hash_array: [u8; 32] = match hash_bytes.try_into() {
        //     Ok(array) => array,
        //     Err(_) => panic!("Radix_History: Expected a Blake3 hash of length 32"),
        // };
        let node_hash = Blake3Hash::from_bytes(node_hash_bytes);
        node_hash
    }

    fn has_no_duplicates(&self, actions: &Vec<HistoryAction>) -> bool {
        let keys: HashSet<_> = actions.iter().map(|action| action.key()).collect();
        keys.len() == actions.len()
    }
}

impl History for RadixHistory {
    fn read(&self, key: ByteVector) -> Result<Option<ByteVector>, HistoryError> {
        let read_result = self.imple.read(self.root_node.clone(), key)?;
        println!("read_result: {:?}", read_result);
        Ok(read_result)
    }

    fn process(&self, actions: Vec<HistoryAction>) -> Result<Box<dyn History>, HistoryError> {
        if !self.has_no_duplicates(&actions) {
            return Err(HistoryError::ActionError(
                "Cannot process duplicate actions on one key.".to_string(),
            ));
        }

        // println!("\nhit process");

        let new_root_node_opt = self.imple.make_actions(self.root_node.clone(), actions)?;

        // println!("\nnew_root_node_opt: {:?}", new_root_node_opt);

        match new_root_node_opt {
            Some(new_root_node) => {
                println!("\nnew_root_node in process: {:?}", new_root_node);
                let node_hash_bytes = self.imple.save_node(new_root_node.clone());
                // println!("\nhash_node_bytes: {:?}", hash_node_bytes);
                // println!(
                //     "\nRadix Tree after Radix History save_node: {:?}",
                //     self.imple.print_tree(&new_root_node, vec![])
                // );
                let root_hash = Blake3Hash::from_bytes(node_hash_bytes);
                // println!("\nHash in process: {:?}", root_hash);
                let new_history = RadixHistory {
                    root_hash,
                    root_node: new_root_node.clone(),
                    imple: RadixTreeImpl::new(self.store.clone()),
                    store: self.store.clone(),
                };
                // println!("\ncache_w before clear: {:?}", self.imple.cache_w);
                self.imple.commit()?;

                // let store_lock = self
                //     .store
                //     .lock()
                //     .expect("Radix History: Failed to acquire store lock");
                // println!("\nstore after commit: {:?}", store_lock.to_map());
                // drop(store_lock);

                self.imple.clear_write_cache();
                self.imple.clear_read_cache();

                // println!(
                //     "\nRadix Tree after Radix History commit: {:?}",
                //     self.imple.print_tree(&new_root_node, vec![])
                // );

                // println!("\ncache_w after clear: {:?}", self.imple.cache_w);

                Ok(Box::new(new_history))
            }
            None => {
                // println!("\nNone in process");
                self.imple.clear_write_cache();
                self.imple.clear_read_cache();
                Ok(Box::new(RadixHistory {
                    root_hash: self.root_hash.clone(),
                    root_node: self.root_node.clone(),
                    imple: self.imple.clone(),
                    store: self.store.clone(),
                }))
            }
        }
    }

    fn root(&self) -> Blake3Hash {
        self.root_hash.clone()
    }

    fn reset(&self, root: &Blake3Hash) -> Box<dyn History> {
        let imple = RadixTreeImpl::new(self.store.clone());

      // println!("")

        let node = imple
            .load_node(root.bytes(), Some(true))
            .expect("Radix History: Unable to call load_node");

        println!("\nroot_node in reset: {:?}", node);

        // Box::new(RadixHistory {
        //     root_hash: root.clone(),
        //     root_node: node,
        //     imple: RadixTreeImpl::new(self.store.clone()),
        //     store: self.store.clone(),
        // })
        Box::new(RadixHistory {
            root_hash: root.clone(),
            root_node: node,
            imple,
            store: self.store.clone(),
        })
    }
}
