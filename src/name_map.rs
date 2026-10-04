//! Registries keyed by `Name`. A spelling becomes a `Name` once, here.
//!
//! Text that is not a name — a rendered parametric form — stays in `rendered`
//! under that text. It is not forced into a `Name`.

use std::collections::{BTreeMap, HashMap};

use crate::scope::Name;

#[derive(Clone, Debug)]
pub struct NameMap<V> {
    by_name: HashMap<Name, (String, V)>,
    /// Spellings `Name::enter` refused. Exact text, not a name.
    rendered: HashMap<String, V>,
}

impl<V> Default for NameMap<V> {
    fn default() -> Self {
        Self {
            by_name: HashMap::new(),
            rendered: HashMap::new(),
        }
    }
}

impl<V> NameMap<V> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, spelling: &str) -> Option<&V> {
        if let Some(name) = Name::enter(spelling) {
            self.by_name.get(&name).map(|(_, v)| v)
        } else {
            self.rendered.get(spelling)
        }
    }

    pub fn get_name(&self, name: &Name) -> Option<&V> {
        self.by_name.get(name).map(|(_, v)| v)
    }

    pub fn get_mut(&mut self, spelling: &str) -> Option<&mut V> {
        if let Some(name) = Name::enter(spelling) {
            self.by_name.get_mut(&name).map(|(_, v)| v)
        } else {
            self.rendered.get_mut(spelling)
        }
    }

    pub fn insert(&mut self, spelling: String, value: V) -> Option<V> {
        if let Some(name) = Name::enter(&spelling) {
            self.by_name.insert(name, (spelling, value)).map(|(_, v)| v)
        } else {
            self.rendered.insert(spelling, value)
        }
    }

    pub fn contains_key(&self, spelling: &str) -> bool {
        self.get(spelling).is_some()
    }

    pub fn remove(&mut self, spelling: &str) -> Option<V> {
        if let Some(name) = Name::enter(spelling) {
            self.by_name.remove(&name).map(|(_, v)| v)
        } else {
            self.rendered.remove(spelling)
        }
    }

    pub fn len(&self) -> usize {
        self.by_name.len() + self.rendered.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_name.is_empty() && self.rendered.is_empty()
    }

    /// The spelling that was inserted, then the value.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &V)> {
        self.by_name
            .values()
            .map(|(s, v)| (s, v))
            .chain(self.rendered.iter())
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.iter().map(|(s, _)| s)
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.by_name
            .values()
            .map(|(_, v)| v)
            .chain(self.rendered.values())
    }

    pub fn or_default(&mut self, spelling: String) -> &mut V
    where
        V: Default,
    {
        if let Some(name) = Name::enter(&spelling) {
            &mut self
                .by_name
                .entry(name)
                .or_insert_with(|| (spelling, V::default()))
                .1
        } else {
            self.rendered.entry(spelling).or_default()
        }
    }

    pub fn rendered_spellings(&self) -> impl Iterator<Item = &String> {
        self.rendered.keys()
    }
}

#[derive(Clone, Debug, Default)]
pub struct NameSet {
    spellings: HashMap<Name, String>,
    rendered: std::collections::HashSet<String>,
}

impl NameSet {
    pub fn insert(&mut self, spelling: String) -> bool {
        if let Some(name) = Name::enter(&spelling) {
            if self.spellings.contains_key(&name) {
                return false;
            }
            self.spellings.insert(name, spelling);
            true
        } else {
            self.rendered.insert(spelling)
        }
    }

    pub fn contains(&self, spelling: &str) -> bool {
        if let Some(name) = Name::enter(spelling) {
            self.spellings.contains_key(&name)
        } else {
            self.rendered.contains(spelling)
        }
    }

    pub fn remove(&mut self, spelling: &str) -> bool {
        if let Some(name) = Name::enter(spelling) {
            self.spellings.remove(&name).is_some()
        } else {
            self.rendered.remove(spelling)
        }
    }

    pub fn len(&self) -> usize {
        self.spellings.len() + self.rendered.len()
    }

    pub fn is_empty(&self) -> bool {
        self.spellings.is_empty() && self.rendered.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &String> {
        self.spellings.values().chain(self.rendered.iter())
    }

    pub fn rendered_spellings(&self) -> impl Iterator<Item = &String> {
        self.rendered.iter()
    }
}

/// Ordered by `Name`, not by the inserted spelling.
#[derive(Clone, Debug)]
pub struct NameBTree<V> {
    by_name: BTreeMap<Name, (String, V)>,
    rendered: BTreeMap<String, V>,
}

impl<V> Default for NameBTree<V> {
    fn default() -> Self {
        Self {
            by_name: BTreeMap::new(),
            rendered: BTreeMap::new(),
        }
    }
}

impl<V> NameBTree<V> {
    pub fn get(&self, spelling: &str) -> Option<&V> {
        if let Some(name) = Name::enter(spelling) {
            self.by_name.get(&name).map(|(_, v)| v)
        } else {
            self.rendered.get(spelling)
        }
    }

    pub fn insert(&mut self, spelling: String, value: V) -> Option<V> {
        if let Some(name) = Name::enter(&spelling) {
            self.by_name.insert(name, (spelling, value)).map(|(_, v)| v)
        } else {
            self.rendered.insert(spelling, value)
        }
    }

    pub fn contains_key(&self, spelling: &str) -> bool {
        self.get(spelling).is_some()
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.by_name
            .values()
            .map(|(s, _)| s)
            .chain(self.rendered.keys())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &V)> {
        self.by_name
            .values()
            .map(|(s, v)| (s, v))
            .chain(self.rendered.iter())
    }

    pub fn rendered_spellings(&self) -> impl Iterator<Item = &String> {
        self.rendered.keys()
    }
}
