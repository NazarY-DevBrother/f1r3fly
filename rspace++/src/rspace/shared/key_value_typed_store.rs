use crate::rspace::shared::key_value_store::KeyValueStore;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::{collections::BTreeMap, marker::PhantomData};

use super::key_value_store::KvStoreError;

// See shared/src/main/scala/coop/rchain/store/KeyValueTypedStore.scala
#[async_trait]
pub trait KeyValueTypedStore<K: Debug + Clone + Send + Sync, V> {
    async fn get(&self, keys: Vec<K>) -> Result<Vec<Option<V>>, KvStoreError>;

    fn put(&self, kv_pairs: Vec<(K, V)>) -> ();

    fn delete(&self, keys: Vec<K>) -> i32;

    fn contains(&self, keys: Vec<K>) -> Vec<bool>;

    // def collect[T](pf: PartialFunction[(K, () => V), T]): F[Seq[T]]
    // TODO: Update this to match scala
    fn collect(&self) -> ();

    fn to_map(&self) -> BTreeMap<K, V>;

    // See shared/src/main/scala/coop/rchain/store/KeyValueTypedStoreSyntax.scala
    async fn get_one(&self, key: &K) -> Result<V, KvStoreError> {
        let mut values = self.get(vec![key.clone()]).await?;
        let first_value = values.remove(0);

        match first_value {
            Some(value) => Ok(value),
            None => {
                panic!("Key_Value_Store: key not found: {:?}", key);
            }
        }
    }
}

// See shared/src/main/scala/coop/rchain/store/KeyValueTypedStoreCodec.scala
#[derive(Clone)]
pub struct KeyValueTypedStoreInstance<K, V> {
    pub store: Box<dyn KeyValueStore>,
    pub _marker: PhantomData<(K, V)>,
}

#[async_trait]
impl<
        K: Debug + Clone + Serialize + Send + 'static + Sync,
        V: Send + Sync + for<'a> Deserialize<'a> + 'static,
    > KeyValueTypedStore<K, V> for KeyValueTypedStoreInstance<K, V>
{
    async fn get(&self, keys: Vec<K>) -> Result<Vec<Option<V>>, KvStoreError> {
        let keys_bytes = keys
            .into_iter()
            .map(|key| bincode::serialize(&key))
            .collect::<Result<Vec<_>, _>>()?;

        let values_bytes = self.store.get(keys_bytes).await?;
        let values: Vec<Option<V>> = values_bytes
            .into_iter()
            .map(|value_bytes_opt| match value_bytes_opt {
                Some(bytes) => bincode::deserialize(&bytes)
                    .expect("Radix Tree: Failed to deserialize value bytes"),
                None => None,
            })
            .collect();

        Ok(values)
    }

    fn put(&self, kv_pairs: Vec<(K, V)>) -> () {
        todo!()
    }

    fn delete(&self, keys: Vec<K>) -> i32 {
        todo!()
    }

    fn contains(&self, keys: Vec<K>) -> Vec<bool> {
        todo!()
    }

    fn collect(&self) -> () {
        todo!()
    }

    fn to_map(&self) -> BTreeMap<K, V> {
        todo!()
    }
}
