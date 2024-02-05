use crate::rspace::shared::key_value_store::KeyValueStore;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::{collections::BTreeMap, marker::PhantomData};

use super::key_value_store::KvStoreError;

// See shared/src/main/scala/coop/rchain/store/KeyValueTypedStore.scala
#[async_trait]
pub trait KeyValueTypedStore<K, V>: Send + Sync
where
    K: Debug + Clone + Send + Sync,
    V: Debug + Send + Sync,
{
    fn get(&self, keys: Vec<K>) -> Result<Vec<Option<V>>, KvStoreError>;

    fn put(&self, kv_pairs: Vec<(K, V)>) -> Result<(), KvStoreError>;

    fn delete(&self, keys: Vec<K>) -> Result<usize, KvStoreError>;

    fn contains(&self, keys: &Vec<K>) -> Result<Vec<bool>, KvStoreError>;

    // def collect[T](pf: PartialFunction[(K, () => V), T]): F[Seq[T]]
    // TODO: Update this to match scala
    fn collect(&self) -> ();

    fn to_map(&self) -> BTreeMap<K, V>;

    // See shared/src/main/scala/coop/rchain/store/KeyValueTypedStoreSyntax.scala
    fn get_one(&self, key: &K) -> Result<Option<V>, KvStoreError> {
        let mut values = self.get(vec![key.clone()])?;
        let first_value = values.remove(0);

        match first_value {
            Some(value) => Ok(Some(value)),
            None => Ok(None),
        }
    }

    fn put_if_absent(&self, kv_pairs: Vec<(K, V)>) -> Result<(), KvStoreError> {
        let keys: Vec<K> = kv_pairs.iter().map(|(k, _)| k.clone()).collect();
        let if_absent = self.contains(&keys)?;
        let kv_if_absent: Vec<_> = kv_pairs.into_iter().zip(if_absent).collect();
        let kv_absent: Vec<_> = kv_if_absent
            .into_iter()
            .filter(|(_, is_present)| !is_present)
            .map(|(kv, _)| kv)
            .collect();

        self.put(kv_absent)
    }
}

// See shared/src/main/scala/coop/rchain/store/KeyValueTypedStoreCodec.scala
#[derive(Clone)]
pub struct KeyValueTypedStoreInstance<K, V> {
    pub store: Box<dyn KeyValueStore>,
    pub _marker: PhantomData<(K, V)>,
}

impl<K, V> KeyValueTypedStore<K, V> for KeyValueTypedStoreInstance<K, V>
where
    K: Debug + Clone + Send + Sync + Serialize + 'static,
    V: Debug + Send + Sync + for<'a> Deserialize<'a> + 'static + Serialize,
{
    fn get(&self, keys: Vec<K>) -> Result<Vec<Option<V>>, KvStoreError> {
        let keys_bytes = keys
            .into_iter()
            .map(|key| bincode::serialize(&key))
            .collect::<Result<Vec<_>, _>>()?;

        let values_bytes = self.store.get(keys_bytes)?;
        let values: Vec<Option<V>> = values_bytes
            .into_iter()
            .map(|value_bytes_opt| match value_bytes_opt {
                Some(bytes) => bincode::deserialize(&bytes)
                    .expect("Key Value Typed Store: Failed to deserialize value bytes"),
                None => None,
            })
            .collect();

        Ok(values)
    }

    fn put(&self, kv_pairs: Vec<(K, V)>) -> Result<(), KvStoreError> {
        let pairs_bytes: Vec<(Vec<u8>, Vec<u8>)> = kv_pairs
            .iter()
            .map(|(k, v)| {
                let serialized_key =
                    bincode::serialize(k).expect("Key Value Typed Store: Failed to serialize key");
                let serialized_value = bincode::serialize(v)
                    .expect("Key Value Typed Store: Failed to serialize value");
                (serialized_key, serialized_value)
            })
            .collect();

        Ok(self.store.put(pairs_bytes)?)
    }

    fn delete(&self, keys: Vec<K>) -> Result<usize, KvStoreError> {
        let keys_bytes: Vec<Vec<u8>> = keys
            .iter()
            .map(|k| {
                let serialized_key =
                    bincode::serialize(k).expect("Key Value Typed Store: Failed to serialize key");
                serialized_key
            })
            .collect();

        let deleted_count = self.store.delete(keys_bytes);
        Ok(deleted_count?)
    }

    fn contains(&self, keys: &Vec<K>) -> Result<Vec<bool>, KvStoreError> {
        let keys_bytes: Vec<Vec<u8>> = keys
            .iter()
            .map(|k| {
                let serialized_key =
                    bincode::serialize(k).expect("Key Value Typed Store: Failed to serialize key");
                serialized_key
            })
            .collect();

        let results = self.store.get(keys_bytes)?;
        Ok(results
            .into_iter()
            .map(|result| !result.is_none())
            .collect())
    }

    // Not implemented because unsure if used in 'rspace' code
    fn collect(&self) -> () {
        todo!()
    }

    // Not implemented because unsure if used in 'rspace' code
    fn to_map(&self) -> BTreeMap<K, V> {
        todo!()
    }
}
