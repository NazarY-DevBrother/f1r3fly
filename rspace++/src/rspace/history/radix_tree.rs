use crate::rspace::shared::key_value_typed_store::KeyValueTypedStore;
use crate::rspace::Byte;
use crate::rspace::ByteVector;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::Mutex;

use super::history_action::HistoryAction;

// See rspace/src/main/scala/coop/rchain/rspace/history/RadixTree.scala
#[derive(Clone, Serialize, Deserialize)]
pub enum Item {
    EmptyItem,

    Leaf {
        prefix: ByteVector,
        value: ByteVector,
    },

    NodePtr {
        prefix: ByteVector,
        ptr: ByteVector,
    },
}

pub type Node = Vec<Item>;

const NUM_ITEMS: usize = 256;

pub struct EmptyNode {
    pub node: Node,
}

impl EmptyNode {
    pub fn new() -> EmptyNode {
        EmptyNode {
            node: vec![Item::EmptyItem; NUM_ITEMS],
        }
    }
}

/** Deserialization [[ByteVector]] to [[Node]]
 */
fn decode(bv: ByteVector) -> Node {
    todo!()
}

fn common_prefix(b1: ByteVector, b2: ByteVector) -> (ByteVector, ByteVector, ByteVector) {
    fn go(
        common: ByteVector,
        l: ByteVector,
        r: ByteVector,
    ) -> (ByteVector, ByteVector, ByteVector) {
        if r.is_empty() || l.is_empty() {
            (common, l, r)
        } else {
            let (l_head, l_tail) = l.split_first().unwrap();
            let (r_head, r_tail) = r.split_first().unwrap();
            if l_head == r_head {
                let mut new_common = common.clone();
                new_common.push(*l_head);
                go(new_common, l_tail.to_vec(), r_tail.to_vec())
            } else {
                (common, l, r)
            }
        }
    }
    go(Vec::new(), b1, b2)
}

pub fn hash_node(node: &Node) -> (ByteVector, ByteVector) {
    let bytes = bincode::serialize(node).unwrap();
    let hash = blake3::hash(&bytes);
    (hash.as_bytes().to_vec(), bytes)
}

fn byte_to_int(b: u8) -> usize {
    b as usize & 0xff
}

/**
 * Data returned after export
 *
 * @param nodePrefixes Node prefixes
 * @param nodeKeys Node KVDB keys
 * @param nodeValues Node KVDB values
 * @param leafPrefixes Leaf prefixes
 * @param leafValues Leaf values (it's pointer for data in datastore)
 */
pub struct ExportData {
    pub node_prefixes: Vec<ByteVector>,
    pub node_keys: Vec<ByteVector>,
    pub node_values: Vec<ByteVector>,
    pub leaf_prefixes: Vec<ByteVector>,
    pub leaf_values: Vec<ByteVector>,
}

/**
 * Settings for [[ExportData]]
 *
 * If false - data will not be exported.
 */
pub struct ExportDataSettings {
    pub flag_node_prefixes: bool,
    pub flag_node_keys: bool,
    pub flag_node_values: bool,
    pub flag_leaf_prefixes: bool,
    pub flag_leaf_values: bool,
}

/**
 * Sequential export algorithm
 *
 * @param rootHash Root node hash, starting point
 * @param lastPrefix Describes the path of root to last processed element (if None - start from root)
 * @param skipSize Describes how many elements to skip
 * @param takeSize Describes how many elements to take
 * @param getNodeDataFromStore Function to get data from storage
 * @param settings [[ExportDataSettings]]
 *
 * @return
 * Return the data and prefix of the last processed item.
 * If all bonds in the tree are processed, returns None as prefix.
 * {{{
 * prefix - Prefix that describes the path of root to node
 * decoded - Deserialized data (from parsing)
 * lastItemIndex - Last processed item index
 * }}}
 */
pub fn sequential_export(
    root_hash: ByteVector,
    last_prefix: Option<ByteVector>,
    skip_size: usize,
    take_size: usize,
    get_node_data_from_store: Arc<dyn Fn(&ByteVector) -> Option<ByteVector>>,
    settings: ExportDataSettings,
) -> (ExportData, Option<ByteVector>) {
    #[derive(Clone)]
    struct NodeData {
        prefix: ByteVector,
        decoded: Node,
        last_item_index: Option<Byte>,
    }

    type Path = Vec<NodeData>; // Sequence used in recursions

    struct NodePathData {
        hash: ByteVector,        // Hash of node for load
        node_prefix: ByteVector, // Prefix of this node
        rest_prefix: ByteVector, // Prefix that describes the rest of the Path
        path: Path,              // Return path
    }

    // Create path from root to lastPrefix node
    let init_node_path = |p: NodePathData| {
        let process_child_item = |node: Node| {
            let item_idx = byte_to_int(*p.rest_prefix.first().unwrap());
            match node.get(item_idx) {
                Some(Item::NodePtr {
                    prefix: ptr_prefix,
                    ptr,
                }) => {
                    let (_, rest_prefix_tail) = p.rest_prefix.split_first().unwrap();
                    let (mut prefix_common, prefix_rest, ptr_prefix_rest) =
                        common_prefix(rest_prefix_tail.to_vec(), ptr_prefix.to_vec());

                    if ptr_prefix_rest.is_empty() {
                        let mut node_prefix_appended = p.node_prefix.clone();
                        node_prefix_appended.push(*p.rest_prefix.first().unwrap());
                        node_prefix_appended.append(&mut prefix_common);

                        Ok(NodePathData {
                            hash: ptr.to_vec(),
                            node_prefix: node_prefix_appended,
                            rest_prefix: prefix_rest,
                            path: {
                                let mut new_path = Vec::new();
                                new_path.push(NodeData {
                                    prefix: p.node_prefix.clone(),
                                    decoded: node,
                                    last_item_index: p.rest_prefix.first().copied(),
                                });

                                new_path.extend(p.path.clone());
                                new_path
                            },
                        })
                    } else {
                        let mut node_prefix_cloned = p.node_prefix.clone();
                        let mut rest_prefix_cloned = p.rest_prefix.clone();
                        node_prefix_cloned.append(&mut rest_prefix_cloned);

                        println!(
                            "Radix Tree - Export error: node with prefix {} not found.",
                            hex::encode(node_prefix_cloned)
                        );
                        Err(Vec::<NodeData>::new())
                    }
                }
                _ => {
                    let mut node_prefix_cloned = p.node_prefix.clone();
                    let mut rest_prefix_cloned = p.rest_prefix.clone();
                    node_prefix_cloned.append(&mut rest_prefix_cloned);

                    println!(
                        "Radix Tree - Export error: node with prefix {} not found.",
                        hex::encode(node_prefix_cloned)
                    );
                    Err(Vec::<NodeData>::new())
                }
            }
        };

        let node_opt = get_node_data_from_store(&p.hash);
        if node_opt.is_none() {
            println!("Radix Tree - Export error: node with key {} not found.", {
                hex::encode(p.hash)
            })
        }
        let decoded_node = decode(node_opt.unwrap());
        if p.rest_prefix.is_empty() {
            let mut new_path = Vec::new();
            new_path.push(NodeData {
                prefix: p.node_prefix,
                decoded: decoded_node,
                last_item_index: None,
            });

            new_path.extend(p.path);
            Ok(new_path) // Happy end
        } else {
            Err(process_child_item(decoded_node)) // Go dipper
        }
    };

    /*
     * Find next non-empty item.
     *
     * @param node Node to look for
     * @param lastIdxOpt Last found index (if this node was not searched - [[None]])
     * @param settings ExportDataSettings from outer scope
     * @return [[Some]](idxItem, [[Item]]) if item found, [[None]] if non-empty item not found
     */
    fn find_next_non_empty_item(
        node: Node,
        last_idx_opt: Option<Byte>,
        settings: ExportDataSettings,
    ) -> Option<(Byte, Item)> {
        if last_idx_opt == Some(0xFF) {
            None
        } else {
            let cur_idx_int = last_idx_opt.map(|b| byte_to_int(b) + 1).unwrap_or(0);
            let cur_item = node.get(cur_idx_int).unwrap();
            let cur_idx = cur_idx_int as u8;
            match cur_item {
                Item::EmptyItem => find_next_non_empty_item(node, Some(cur_idx), settings),
                Item::Leaf {
                    prefix: _,
                    value: _,
                } => {
                    if settings.flag_leaf_prefixes || settings.flag_leaf_values {
                        Some((cur_idx, cur_item.clone()))
                    } else {
                        find_next_non_empty_item(node, Some(cur_idx), settings)
                    }
                }
                Item::NodePtr { prefix: _, ptr: _ } => Some((cur_idx, cur_item.clone())),
            }
        }
    }

    struct StepData {
        path: Path,           // Path of node from current to root
        skip: usize,          // Skip counter
        take: usize,          // Take counter
        exp_data: ExportData, // Result of export
    }

    let add_leaf = |p: StepData,
                    leaf_prefix: ByteVector,
                    leaf_value: ByteVector,
                    item_index: Byte,
                    curr_node_prefix: ByteVector,
                    new_path: Vec<NodeData>| {
        if p.skip > 0 {
            StepData {
                path: new_path,
                skip: p.skip,
                take: p.take,
                exp_data: p.exp_data,
            }
        } else {
            let new_lp = if settings.flag_leaf_prefixes {
                let mut new_single_lp = curr_node_prefix;
                let mut leaf_prefix_copy = leaf_prefix;
                new_single_lp.push(item_index);
                new_single_lp.append(&mut leaf_prefix_copy);

                let mut leaf_prefixes_copy = p.exp_data.leaf_prefixes;
                leaf_prefixes_copy.push(new_single_lp);
                leaf_prefixes_copy
            } else {
                Vec::new()
            };

            let new_lv = if settings.flag_leaf_values {
                let mut leaf_values_copied = p.exp_data.leaf_values;
                leaf_values_copied.push(leaf_value);
                leaf_values_copied
            } else {
                Vec::new()
            };

            let new_export_data = ExportData {
                node_prefixes: p.exp_data.node_prefixes,
                node_keys: p.exp_data.node_keys,
                node_values: p.exp_data.node_values,
                leaf_prefixes: new_lp,
                leaf_values: new_lv,
            };

            StepData {
                path: new_path,
                skip: p.skip,
                take: p.take,
                exp_data: new_export_data,
            }
        }
    };

    todo!()
}

/**
 * Radix Tree implementation
 */
#[derive(Clone)]
pub struct RadixTreeImpl {
    pub store: Arc<Mutex<Box<dyn KeyValueTypedStore<ByteVector, ByteVector>>>>,
    /**
     * Cache for storing read and decoded nodes.
     *
     * Cache stores kv-pairs (hash, node).
     * Where hash - Blake2b256Hash of serializing nodes data,
     *       node - deserialized data of this node.
     */
    pub cache_r: DashMap<ByteVector, Node>,
    /**
     * Cache for storing serializing nodes. For subsequent unloading in KVDB
     *
     * Cache stores kv-pairs (hash, bytes).
     * Where hash -  Blake2b256Hash of bytes,
     *       bytes - serializing data of nodes.
     */
    pub cache_w: DashMap<ByteVector, ByteVector>,
}

impl RadixTreeImpl {
    pub fn new(store: Arc<Mutex<Box<dyn KeyValueTypedStore<ByteVector, ByteVector>>>>) -> Self {
        RadixTreeImpl {
            store,
            cache_r: DashMap::new(),
            cache_w: DashMap::new(),
        }
    }

    /**
     * Load and decode serializing data from KVDB.
     */
    fn load_node_from_store(&self, node_ptr: &ByteVector) -> Option<Node> {
        let store_lock = self
            .store
            .lock()
            .expect("Radix Tree: Failed to acquire lock on store");
        let get_result = store_lock.get_one(&node_ptr);

        match get_result {
            Ok(bytes_opt) => match bytes_opt {
                Some(bytes) => {
                    let deserialized = bincode::deserialize(&bytes)
                        .expect("Radix Tree: Failed to deserialize node bytes");
                    deserialized
                }
                None => None,
            },
            Err(err) => {
                println!("Radix Tree: {}", err);
                None
            }
        }
    }

    /**
     * Load one node from [[cacheR]].
     *
     * If there is no such record in cache - load and decode from KVDB, then save to cacheR.
     * If there is no such record in KVDB - execute assert (if set noAssert flag - return emptyNode).
     */
    pub fn load_node(&self, node_ptr: ByteVector, no_assert: Option<bool>) -> Node {
        let no_assert = no_assert.unwrap_or(false);

        let error_msg = |node_ptr: &[u8]| {
            assert!(
                no_assert,
                "Missing node in database. ptr={:?}",
                node_ptr
                    .iter()
                    .map(|byte| format!("{:02x}", byte))
                    .collect::<Vec<_>>()
            );
        };

        let cache_miss = |node_ptr: ByteVector| {
            let store_node_opt = self.load_node_from_store(&node_ptr);

            let node_opt = store_node_opt
                .map(|node| self.cache_r.insert(node_ptr.clone(), node))
                .unwrap_or_else(|| {
                    error_msg(&node_ptr);
                    None
                });

            match node_opt {
                Some(node) => node,
                None => EmptyNode::new().node,
            }
        };

        let cache_node_opt = self.cache_r.get(&node_ptr);
        match cache_node_opt {
            Some(node) => node.to_vec(),
            None => cache_miss(node_ptr),
        }
    }

    /**
     * Clear [[cacheR]] (cache for storing read nodes).
     */
    pub fn clear_read_cache(&self) -> () {
        self.cache_r.clear()
    }

    /**
     * Serializing and hashing one [[Node]].
     *
     * Serializing data load in [[cacheW]].
     * If detected collision with older cache data - executing assert
     */
    pub fn save_node(&self, node: &Node) -> ByteVector {
        todo!()
    }

    /**
     * Save all [[cacheW]] to [[store]]
     *
     * If detected collision with older KVDB data - execute Exception
     */
    pub fn commit(&self) -> () {
        todo!()
    }

    /**
     * Clear [[cacheW]] (cache for storing data to write in KVDB).
     */
    pub fn clear_write_cache(&self) -> () {
        self.cache_w.clear()
    }

    /**
     * Read leaf data with prefix. If data not found, returned [[None]]
     */
    pub fn read(&self, start_node: Node, start_prefix: ByteVector) -> Option<ByteVector> {
        todo!()
    }

    /**
     * Parallel processing of [[HistoryAction]]s in this part of tree (start from curNode).
     *
     * New data load to [[cacheW]].
     * @return Updated curNode. if no action was taken - return [[None]].
     */
    pub fn make_actions(&self, curr_node: Node, actions: Vec<HistoryAction>) -> Option<Node> {
        todo!()
    }
}
