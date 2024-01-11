use crate::rspace::internal::{Datum, WaitingContinuation};
use bytes::Bytes;

/**
* Reader for particular history (state verified on blockchain)
*
* @tparam Key type for hash of a channel
* @tparam C type for Channel => this is Par
* @tparam P type for Pattern => this is BindPattern
* @tparam A type for Abstraction => this is ListParWithRandom
* @tparam K type for Continuation => this is TaggedContinuation
*
* See rspace/src/main/scala/coop/rchain/rspace/history/HistoryReader.scala
*/
pub trait HistoryReader<Key, C, P, A, K> {
    // Get current root which reader reads from
    fn root(&self) -> Key;

    fn get_data_proj(&self, key: Key, proj: fn(Datum<A>, Bytes) -> Datum<A>) -> Vec<Datum<A>>;

    fn get_continuations_proj(
        &self,
        key: Key,
        proj: fn(WaitingContinuation<P, K>, Bytes) -> WaitingContinuation<P, K>,
    ) -> Vec<WaitingContinuation<P, K>>;

    fn get_joins_proj(&self, key: Key, proj: fn(Vec<C>, Bytes) -> Vec<C>) -> Vec<Vec<C>>;

    /**                                                                                                                                                                                                              
     * Defaults                                                                                                                                                                                                       
     */
    fn get_data(&self, key: Key) -> Vec<Datum<A>> {
        self.get_data_proj(key, |d, _| d)
    }

    fn get_continuations(&self, key: Key) -> Vec<WaitingContinuation<P, K>> {
        self.get_continuations_proj(key, |d, _| d)
    }

    fn get_joins(&self, key: Key) -> Vec<Vec<C>> {
        self.get_joins_proj(key, |d, _| d)
    }

    /**
     * Get reader which accepts non-serialized and hashed keys
     */
    fn base(&self) -> Box<dyn HistoryReaderBase<C, P, A, K>>;
}

/**
 * History reader base, version of a reader which accepts non-serialized and hashed keys
 */
pub trait HistoryReaderBase<C, P, A, K> {
    fn get_data_proj(&self, key: C, proj: fn(Datum<A>, Bytes) -> Datum<A>) -> Vec<Datum<A>>;

    fn get_continuations_proj(
        &self,
        key: Vec<C>,
        proj: fn(WaitingContinuation<P, K>, Bytes) -> WaitingContinuation<P, K>,
    ) -> Vec<WaitingContinuation<P, K>>;

    fn get_joins_proj(&self, key: C, proj: fn(Vec<C>, Bytes) -> Vec<C>) -> Vec<Vec<C>>;

    /**                                                                                                                                                                                                              
     * Defaults                                                                                                                                                                                                       
     */
    fn get_data(&self, key: C) -> Vec<Datum<A>> {
        self.get_data_proj(key, |d, _| d)
    }

    fn get_continuations(&self, key: Vec<C>) -> Vec<WaitingContinuation<P, K>> {
        self.get_continuations_proj(key, |d, _| d)
    }

    fn get_joins(&self, key: C) -> Vec<Vec<C>> {
        self.get_joins_proj(key, |d, _| d)
    }
}
