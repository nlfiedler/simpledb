//
// Copyright (c) 2022 Nathan Fiedler
//

//! A simple in-memory key/value with nested transactions and a function for
//! getting the number of occurrences of a particular value. Keys and values are
//! strings.

use std::collections::HashMap;

///
/// A simple key/value store that counts values.
///
#[derive(Clone)]
struct CountingStore {
    values: HashMap<String, Option<String>>,
    counts: HashMap<String, u32>,
}

impl CountingStore {
    /// Construct a new counting store.
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            counts: HashMap::new(),
        }
    }

    /// Returns true if the key exists at all, which includes deleted keys.
    pub fn contains(&self, name: &str) -> bool {
        self.values.contains_key(name)
    }

    /// Retrieve the value for the given key, if any.
    pub fn get(&self, name: &str) -> Option<String> {
        let cloned = self.values.get(name).cloned();
        cloned.flatten()
    }

    /// Save the value using the given key.
    pub fn set<T: Into<String>>(&mut self, name: T, value: T) {
        // update count for new value
        let name_str = name.into();
        let value_str = value.into();
        if let Some(c) = self.counts.get_mut(&value_str) {
            *c += 1;
        } else {
            self.counts.insert(value_str.clone(), 1);
        }
        // update count for the old value, if any
        self.delete(&name_str);
        self.values.insert(name_str, Some(value_str));
    }

    /// Removes the value with the given key from the store by overwriting it
    /// with a `None`.
    pub fn delete(&mut self, name: &str) {
        if let Some(v) = self.values.get_mut(name) {
            if let Some(value) = v.take() {
                if let Some(c) = self.counts.get_mut(&value) {
                    *c -= 1;
                }
            }
        } else {
            // for the sake of transactions, make the key disappear
            self.values.insert(name.into(), None);
        }
    }

    /// Returns the number of occurrences of the given value.
    pub fn count(&self, value: &str) -> u32 {
        *self.counts.get(value).or(Some(&0)).unwrap()
    }

    /// Removes all deleted entries.
    pub fn compact(&mut self) {
        self.values.retain(|_, v| v.is_some());
    }
}

///
/// Combination of a counting store and local metadata to track changes without
/// altering the parent transaction, if any.
///
#[derive(Clone)]
struct Transaction {
    store: CountingStore,
    parent: Option<Box<Transaction>>,
    counts: HashMap<String, i64>,
}

impl<'a> Transaction {
    /// Construct a new transaction.
    pub fn new() -> Self {
        Self {
            store: CountingStore::new(),
            parent: None,
            counts: HashMap::new(),
        }
    }

    /// Set the parent transaction.
    pub fn parent(mut self, parent: Transaction) -> Self {
        self.parent = Some(Box::new(parent));
        self
    }

    /// Retrieve the value for the given key, if any.
    pub fn get(&self, name: &str) -> Option<String> {
        if self.store.contains(name) {
            self.store.get(name)
        } else {
            if let Some(parent) = self.parent.as_ref() {
                parent.get(name)
            } else {
                None
            }
        }
    }

    /// Save the value using the given key in the transaction.
    pub fn set<T: Into<String>>(&mut self, name: T, value: T) {
        let name_str: String = name.into();
        self.delete(&name_str);
        let value_str: String = value.into();
        self.store.set(&name_str, &value_str)
    }

    /// Removes the value with the given key from the transaction.
    pub fn delete(&mut self, name: &str) {
        if !self.store.contains(name) {
            if let Some(parent) = self.parent.as_ref() {
                if let Some(value) = parent.get(name) {
                    if let Some(c) = self.counts.get_mut(&value) {
                        *c -= 1;
                    } else {
                        self.counts.insert(value.clone(), -1);
                    }
                }
            }
        }
        self.store.delete(name);
    }

    /// Returns the number of occurrences of the given value.
    pub fn count(&self, value: &str) -> u32 {
        let count = self.store.count(value);
        let local_count = *self.counts.get(value).or(Some(&0)).unwrap();
        let parent_count = if let Some(parent) = self.parent.as_ref() {
            parent.count(value)
        } else {
            0
        };
        std::cmp::max((count + parent_count) as i64 + local_count, 0) as u32
    }
}

///
/// In-memory key/value store that supports nested transactions.
///
pub struct Database {
    transaction: Transaction,
}

impl Database {
    /// Construct a new database.
    pub fn new() -> Self {
        Self {
            transaction: Transaction::new(),
        }
    }

    /// Retrieve the value for the given key, if any.
    pub fn get(&self, name: &str) -> Option<String> {
        self.transaction.get(name)
    }

    /// Save the value using the given key.
    pub fn set<T: Into<String>>(&mut self, name: T, value: T) {
        self.transaction.set(name, value)
    }

    /// Removes the value with the given key.
    pub fn delete(&mut self, name: &str) {
        self.transaction.delete(name)
    }

    /// Returns the number of occurrences of the given value.
    pub fn count(&self, value: &str) -> u32 {
        self.transaction.count(value)
    }

    /// Start a new transaction.
    pub fn begin(&mut self) {
        let mut transaction = Transaction::new();
        transaction = transaction.parent(self.transaction.clone());
        self.transaction = transaction;
    }

    /// Commit _all_ open transactions.
    pub fn commit(&mut self) {
        while let Some(mut transaction) = self.transaction.parent.take() {
            for (key, value) in self.transaction.store.values.iter() {
                if let Some(v) = value {
                    transaction.set(key, v);
                } else {
                    transaction.delete(key);
                }
            }
            self.transaction = *transaction;
        }
        self.transaction.store.compact();
    }

    /// Rollback the current transaction. Returns true if rollback was
    /// successful or false if there is no open tranaction.
    pub fn rollback(&mut self) -> bool {
        if let Some(transaction) = self.transaction.parent.take() {
            self.transaction = *transaction;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counting_store() {
        let mut store = CountingStore::new();
        assert_eq!(store.count("value"), 0);
        assert_eq!(store.get("name1"), None);
        store.set("name1", "value");
        assert_eq!(store.get("name1"), Some("value".into()));
        assert_eq!(store.count("value"), 1);
        store.set("name2", "value");
        assert_eq!(store.count("value"), 2);
        store.set("name3", "value");
        assert_eq!(store.count("value"), 3);
        store.delete("name3");
        assert_eq!(store.get("name3"), None);
        assert_eq!(store.count("value"), 2);
        store.delete("name2");
        assert_eq!(store.get("name2"), None);
        assert_eq!(store.count("value"), 1);
        store.delete("name1");
        assert_eq!(store.get("name1"), None);
        assert_eq!(store.count("value"), 0);
    }

    #[test]
    fn test_transactions() {
        let mut first = Transaction::new();
        first.set("name2", "value");
        first.set("name1", "value1");
        let mut second = Transaction::new();
        second = second.parent(first.clone());
        second.set("name1", "value2");
        second.set("name3", "value");
        assert_eq!(second.get("name1"), Some("value2".into()));
        assert_eq!(first.get("name1"), Some("value1".into()));
        assert_eq!(second.count("value"), 2);
        assert_eq!(first.count("value"), 1);
        second.delete("name3");
        assert_eq!(second.count("value"), 1);
        second.delete("name2");
        assert_eq!(second.count("value"), 0);
        assert_eq!(first.count("value"), 1);
    }

    #[test]
    fn test_example_1() {
        let mut db = Database::new();
        assert_eq!(db.get("a"), None);
        db.set("a", "foo");
        db.set("b", "foo");
        assert_eq!(db.count("foo"), 2);
        assert_eq!(db.count("bar"), 0);
        db.delete("a");
        assert_eq!(db.count("foo"), 1);
        db.set("b", "baz");
        assert_eq!(db.count("foo"), 0);
        assert_eq!(db.get("b"), Some("baz".into()));
        assert_eq!(db.get("B"), None);
    }

    #[test]
    fn test_example_2() {
        let mut db = Database::new();
        db.set("a", "foo");
        db.set("a", "foo");
        assert_eq!(db.count("foo"), 1);
        assert_eq!(db.get("a"), Some("foo".into()));
        db.delete("a");
        assert_eq!(db.get("a"), None);
        assert_eq!(db.count("foo"), 0);
    }

    #[test]
    fn test_example_3() {
        let mut db = Database::new();
        db.begin();
        db.set("a", "foo");
        assert_eq!(db.get("a"), Some("foo".into()));
        db.begin();
        db.set("a", "bar");
        assert_eq!(db.get("a"), Some("bar".into()));
        db.set("a", "baz");
        db.rollback();
        assert_eq!(db.get("a"), Some("foo".into()));
        db.rollback();
        assert_eq!(db.get("a"), None);
    }

    #[test]
    fn test_example_4() {
        let mut db = Database::new();
        db.set("a", "foo");
        db.set("b", "baz");
        db.begin();
        assert_eq!(db.get("a"), Some("foo".into()));
        db.set("a", "bar");
        assert_eq!(db.count("bar"), 1);
        db.begin();
        assert_eq!(db.count("bar"), 1);
        db.delete("a");
        assert_eq!(db.get("a"), None);
        assert_eq!(db.count("bar"), 0);
        db.rollback();
        assert_eq!(db.get("a"), Some("bar".into()));
        assert_eq!(db.count("bar"), 1);
        db.commit();
        assert_eq!(db.get("a"), Some("bar".into()));
        assert_eq!(db.get("b"), Some("baz".into()));
    }

    #[test]
    fn test_shadow_delete_count() {
        let mut db = Database::new();
        db.set("a", "foo");
        db.set("b", "foo");
        db.begin();
        assert_eq!(db.get("a"), Some("foo".into()));
        assert_eq!(db.get("b"), Some("foo".into()));
        assert_eq!(db.count("foo"), 2);
        db.set("a", "bar");
        assert_eq!(db.count("bar"), 1);
        assert_eq!(db.count("foo"), 1);
        db.delete("a");
        assert_eq!(db.count("bar"), 0);
        assert_eq!(db.count("foo"), 1);
        db.delete("a");
        assert_eq!(db.count("bar"), 0);
        assert_eq!(db.count("foo"), 1);
        db.delete("b");
        assert_eq!(db.count("bar"), 0);
        assert_eq!(db.count("foo"), 0);
    }
}
