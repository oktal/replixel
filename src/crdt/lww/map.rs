//! Module that defines a `Map` of [`LWWRegister`] values

use std::borrow::Borrow;
use std::collections::{hash_map, HashMap};
use std::hash::Hash;

use crate::crdt::{CRDTExt, CRDT};

use super::register::LWWRegister;

pub enum Entry<V> {
    Occupied(V),
    Tombstoned,
}

impl<V> Entry<V> {
    /// Return a reference to the current value that this entry holds
    /// Return [`None`] if the current entry is [`Self::Tombstoned`]
    fn get(&self) -> Option<&V> {
        let Self::Occupied(v) = self else {
            return None;
        };

        Some(v)
    }

    /// Take the entry if the current entry is [`Self::Occupied`]
    /// Return [`None`] if the current entry is [`Self::Tombstoned`]
    fn take(self) -> Option<V> {
        let Entry::Occupied(v) = self else {
            return None;
        };

        Some(v)
    }

    /// Returns `true` if the current entry is [`Tombstoned`]
    fn is_tombstoned(&self) -> bool {
        matches!(self, Self::Tombstoned)
    }
}

pub struct MapState<K, V> {
    inner: HashMap<K, LWWRegister<Entry<V>>>,
}

/// A map of [`LWWRegister`] values
pub struct LWWMap<K, V> {
    state: MapState<K, V>,
}

impl<K, V> LWWMap<K, V> {
    /// Create a new, empty map
    pub fn new() -> Self {
        Self {
            state: MapState {
                inner: HashMap::new(),
            },
        }
    }
}

impl<K, V> LWWMap<K, V>
where
    K: Eq + Hash,
{
    /// Returns a reference to the value corresponding to the key.
    pub fn get<Q>(&self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.state.inner.get(k).and_then(|reg| reg.value().get())
    }

    /// Inserts a key-value pair into the map.
    /// If the map did not have this key present, [`None`] is returned.
    /// If the map did have this key present, the register holding the value is updated, and the old value is returned.
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        match self.state.inner.entry(k) {
            hash_map::Entry::Occupied(mut e) => e.get_mut().update(Entry::Occupied(v)).take(),
            hash_map::Entry::Vacant(e) => {
                e.insert(LWWRegister::new(Entry::Occupied(v)));
                None
            }
        }
    }

    pub fn remove<Q>(&mut self, k: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.state
            .inner
            .get_mut(k)
            .and_then(|e| e.update(Entry::Tombstoned).take())
    }

    /// Returns `true` if the map contains a value for the specified key.
    pub fn contains_key<Q>(&self, k: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.state
            .inner
            .get(k)
            .map(|e| !e.value().is_tombstoned())
            .unwrap_or(false)
    }
}

impl<K, V> CRDT for LWWMap<K, V>
where
    K: Eq + Hash,
{
    type State = MapState<K, V>;

    fn merge(&mut self, other: Self::State) {
        for (k, v) in other.inner {
            match self.state.inner.entry(k) {
                hash_map::Entry::Occupied(mut e) => v.merge_into(e.get_mut()),
                hash_map::Entry::Vacant(e) => {
                    if let Some(entry) = v.take() {
                        e.insert(LWWRegister::new(entry));
                    }
                }
            }
        }
    }

    fn take(self) -> Self::State {
        self.state
    }
}

impl<K, V> FromIterator<(K, V)> for LWWMap<K, V>
where
    K: Eq + Hash,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let iter = iter
            .into_iter()
            .map(|(k, v)| (k, LWWRegister::new(Entry::Occupied(v))));

        Self {
            state: MapState {
                inner: iter.collect(),
            },
        }
    }
}
