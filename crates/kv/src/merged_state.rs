use std::collections::HashMap;
use std::hash::Hash;


use crate::bit_cask_key::BitCaskKey;
use crate::entry::MappedStoredEntry;

pub struct MergedState<Key: BitCaskKey> {
    pub value_by_key: HashMap<Key, MappedStoredEntry<Key>>,
    pub deleted_keys: HashMap<Key, MappedStoredEntry<Key>>,
}

impl<Key: BitCaskKey + Eq + Hash + Clone> MergedState<Key> {
    // Equivalent to NewMergedState
    pub fn new() -> Self {
        MergedState {
            value_by_key: HashMap::new(),
            deleted_keys: HashMap::new(),
        }
    }

    // Equivalent to merge method
    pub fn merge(
        &mut self,
        entries: Vec<MappedStoredEntry<Key>>,
        other_entries: Vec<MappedStoredEntry<Key>>,
    ) {
        self.take_all(entries);
        self.merge_with(other_entries);
    }

    // Equivalent to takeAll method
    pub fn take_all(&mut self, mapped_entries: Vec<MappedStoredEntry<Key>>) {
        for entry in mapped_entries {
            if entry.deleted {
                self.deleted_keys.insert(entry.key.clone(), entry);
            } else {
                self.value_by_key.insert(entry.key.clone(), entry);
            }
        }
    }

    // Equivalent to mergeWith method
    pub fn merge_with(&mut self, mapped_entries: Vec<MappedStoredEntry<Key>>) {
        for new_entry in mapped_entries {
            if let Some(existing) = self.value_by_key.get(&new_entry.key) {
                self.maybe_update(&existing.clone(), new_entry);
            } else if let Some(deleted_entry) = self.deleted_keys.remove(&new_entry.key) {
                self.maybe_update(&deleted_entry, new_entry.clone());
                if !new_entry.deleted {
                    self.value_by_key.insert(new_entry.key.clone(), new_entry);
                }
            } else {
                self.value_by_key.insert(new_entry.key.clone(), new_entry);
            }
        }
    }

    // Equivalent to maybeUpdate method
    fn maybe_update(&mut self, existing_entry: &MappedStoredEntry<Key>, new_entry: MappedStoredEntry<Key>) {
        if new_entry.timestamp > existing_entry.timestamp {
            if new_entry.deleted {
                self.value_by_key.remove(&existing_entry.key);
            } else {
                self.value_by_key.insert(existing_entry.key.clone(), new_entry);
            }
        }
    }
}
