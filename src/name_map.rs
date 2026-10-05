//! Registries keyed by `Name`. A spelling becomes a `Name` once, at insert.
//!
//! A spelling `Name::enter` refuses is not stored. The insert panics with that
//! spelling so the site is named.

use std::collections::{BTreeMap, HashMap};
use std::io::Write;

use crate::scope::Name;

/// One insert whose `Name` was already present under a different spelling.
#[derive(Clone, Debug)]
pub struct NameCollision {
    pub namespace: String,
    pub name: String,
    pub kept: String,
    pub incoming: String,
    /// `Some` when the two values could be compared. `None` when the value
    /// type has no equality (the insert recorded the spellings only).
    pub values_equal: Option<bool>,
}

std::thread_local! {
    static COLLISIONS: std::cell::RefCell<Vec<NameCollision>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

fn record_collision(name: &Name, kept: &str, incoming: &str, values_equal: Option<bool>) {
    if kept == incoming {
        return;
    }
    let row = NameCollision {
        namespace: name.namespace().to_string(),
        name: name.name().to_string(),
        kept: kept.to_string(),
        incoming: incoming.to_string(),
        values_equal,
    };
    COLLISIONS.with(|c| c.borrow_mut().push(row));
    let Ok(path) = std::env::var("WAT_NAME_COLLISIONS") else {
        return;
    };
    if path.is_empty() {
        return;
    }
    let eq = match values_equal {
        Some(true) => "equal",
        Some(false) => "unequal",
        None => "unknown",
    };
    // One line, well under PIPE_BUF, so an O_APPEND write is atomic.
    let line = format!(
        "{}\t{}\t{kept}\t{incoming}\t{eq}\n",
        name.namespace(),
        name.name()
    );
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = f.write_all(line.as_bytes());
    }
}

/// The collisions recorded on this thread. Not a gate.
pub fn collision_log() -> Vec<NameCollision> {
    COLLISIONS.with(|c| c.borrow().clone())
}

fn must_name(spelling: &str) -> Name {
    Name::enter(spelling).unwrap_or_else(|| {
        panic!("a registry insert was not a name: {spelling}");
    })
}

/// The lookup hoist. `Name::enter` parses and allocates. A text boundary is
/// the insert; a repeated lookup of a spelling already entered on this thread
/// reuses that pair. Inserts still call [`Name::enter`] directly. The cache
/// is not an identity: two threads, and a spelling never seen, still enter.
fn cached_enter(spelling: &str) -> Option<Name> {
    std::thread_local! {
        static ENTERED: std::cell::RefCell<HashMap<String, Option<Name>>> =
            std::cell::RefCell::new(HashMap::new());
    }
    ENTERED.with(|slot| {
        if let Some(hit) = slot.borrow().get(spelling) {
            return hit.clone();
        }
        let entered = Name::enter(spelling);
        slot.borrow_mut().insert(spelling.to_string(), entered.clone());
        entered
    })
}

#[derive(Clone, Debug)]
pub struct NameMap<V> {
    values: Vec<Option<V>>,
    /// The spelling the first insert kept, parallel to `values`.
    kept: Vec<String>,
    /// Inserted spelling, its keyword image, and its `Name` display. A
    /// lookup of any of those is one string hash. Measured on the rete fuzz:
    /// most lookups are not the spelling the insert kept.
    index: HashMap<String, u32>,
    by_name: HashMap<Name, u32>,
}

impl<V> Default for NameMap<V> {
    fn default() -> Self {
        Self {
            values: Vec::new(),
            kept: Vec::new(),
            index: HashMap::new(),
            by_name: HashMap::new(),
        }
    }
}

impl<V> NameMap<V> {
    pub fn new() -> Self {
        Self::default()
    }

    fn live(&self, idx: u32) -> Option<&V> {
        self.values.get(idx as usize)?.as_ref()
    }

    fn bind_images(&mut self, idx: u32, name: &Name, spelling: String) {
        self.index.insert(spelling, idx);
        let keyword = crate::type_key::name_as_keyword(name);
        self.index.entry(keyword).or_insert(idx);
        let shown = name.to_string();
        self.index.entry(shown).or_insert(idx);
    }

    /// Index slot of `spelling`, if this map holds that name.
    ///
    /// A keyword image (`:ns::name`, no `/`) and a display image (`ns/name`,
    /// no `::`) are written into `index` at insert. Missing there means the
    /// name is absent: do not `enter`. The slash-keyword (`:ns::a/b`) is
    /// neither image, so it still enters.
    fn find_idx(&self, spelling: &str) -> Option<u32> {
        if let Some(&idx) = self.index.get(spelling) {
            return Some(idx);
        }
        let keyword_image = spelling.starts_with(':') && spelling.contains("::") && !spelling.contains('/');
        let display_image = !spelling.starts_with(':') && spelling.contains('/') && !spelling.contains("::");
        if keyword_image || display_image {
            return None;
        }
        // The keyword printer (`:ns/name`, `:k` has no slash and hit `index`
        // or falls through). Parse it through the one constructor. `enter`
        // would keep the leading colon on the namespace.
        if spelling.starts_with(':') && spelling.contains('/') && !spelling.contains("::") {
            let name = Name::from_keyword_value(spelling)?;
            return self.by_name.get(&name).copied();
        }
        let name = cached_enter(spelling)?;
        self.by_name.get(&name).copied()
    }

    pub fn get(&self, spelling: &str) -> Option<&V> {
        let idx = self.find_idx(spelling)?;
        self.live(idx)
    }

    pub fn get_name(&self, name: &Name) -> Option<&V> {
        let &idx = self.by_name.get(name)?;
        self.live(idx)
    }

    pub fn get_mut(&mut self, spelling: &str) -> Option<&mut V> {
        let idx = self.find_idx(spelling)?;
        self.values.get_mut(idx as usize)?.as_mut()
    }

    pub fn insert(&mut self, spelling: String, value: V) -> Option<V> {
        self.insert_with(spelling, value, |_, _| None)
    }

    pub fn insert_eq(&mut self, spelling: String, value: V) -> Option<V>
    where
        V: PartialEq,
    {
        self.insert_with(spelling, value, |a, b| Some(a == b))
    }

    pub fn insert_with(
        &mut self,
        spelling: String,
        value: V,
        values_equal: impl FnOnce(&V, &V) -> Option<bool>,
    ) -> Option<V> {
        let name = must_name(&spelling);
        if let Some(&idx) = self.by_name.get(&name) {
            let kept_owned = self.kept[idx as usize].clone();
            if kept_owned != spelling {
                let eq = self.live(idx).and_then(|old| values_equal(old, &value));
                record_collision(&name, &kept_owned, &spelling, eq);
            }
            self.index.insert(spelling, idx);
            return self.values[idx as usize].replace(value);
        }
        let idx = self.values.len() as u32;
        self.values.push(Some(value));
        self.kept.push(spelling.clone());
        self.by_name.insert(name.clone(), idx);
        self.bind_images(idx, &name, spelling);
        None
    }

    pub fn contains_key(&self, spelling: &str) -> bool {
        self.get(spelling).is_some()
    }

    pub fn remove(&mut self, spelling: &str) -> Option<V> {
        let name = cached_enter(spelling)?;
        let idx = self.by_name.remove(&name)?;
        self.index.retain(|_, i| *i != idx);
        self.kept[idx as usize].clear();
        self.values[idx as usize].take()
    }

    pub fn len(&self) -> usize {
        self.by_name.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_name.is_empty()
    }

    /// The spelling that was inserted first, then the value.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &V)> {
        self.kept.iter().zip(self.values.iter()).filter_map(|(s, v)| {
            let val = v.as_ref()?;
            if s.is_empty() {
                None
            } else {
                Some((s, val))
            }
        })
    }

    pub fn names(&self) -> impl Iterator<Item = &Name> {
        self.by_name.keys()
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.iter().map(|(s, _)| s)
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.values.iter().filter_map(|v| v.as_ref())
    }

    pub fn or_default(&mut self, spelling: String) -> &mut V
    where
        V: Default,
    {
        let name = must_name(&spelling);
        if let Some(&idx) = self.by_name.get(&name) {
            if self.kept[idx as usize] != spelling {
                let kept_owned = self.kept[idx as usize].clone();
                record_collision(&name, &kept_owned, &spelling, Some(true));
            }
            self.index.insert(spelling, idx);
            return self.values[idx as usize]
                .as_mut()
                .expect("a live name has a value");
        }
        let idx = self.values.len() as u32;
        self.values.push(Some(V::default()));
        self.kept.push(spelling.clone());
        self.by_name.insert(name.clone(), idx);
        self.bind_images(idx, &name, spelling);
        self.values[idx as usize]
            .as_mut()
            .expect("a just-inserted name has a value")
    }
}

#[derive(Clone, Debug, Default)]
pub struct NameSet {
    by_name: HashMap<Name, String>,
    /// Kept spellings. `contains` of one of these is one string hash.
    kept: std::collections::HashSet<String>,
}

impl NameSet {
    pub fn insert(&mut self, spelling: String) -> bool {
        let name = must_name(&spelling);
        if let Some(kept) = self.by_name.get(&name) {
            if kept != &spelling {
                record_collision(&name, kept, &spelling, Some(true));
            }
            return false;
        }
        self.kept.insert(spelling.clone());
        self.by_name.insert(name, spelling);
        true
    }

    pub fn contains(&self, spelling: &str) -> bool {
        if self.kept.contains(spelling) {
            return true;
        }
        let Some(name) = cached_enter(spelling) else {
            return false;
        };
        self.by_name.contains_key(&name)
    }

    pub fn contains_name(&self, name: &Name) -> bool {
        self.by_name.contains_key(name)
    }

    pub fn remove(&mut self, spelling: &str) -> bool {
        let Some(name) = cached_enter(spelling) else {
            return false;
        };
        let Some(kept) = self.by_name.remove(&name) else {
            return false;
        };
        self.kept.remove(&kept);
        true
    }

    pub fn len(&self) -> usize {
        self.by_name.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_name.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &String> {
        self.by_name.values()
    }

    pub fn names(&self) -> impl Iterator<Item = (&Name, &String)> {
        self.by_name.iter()
    }
}

/// Ordered by `Name`, not by the inserted spelling.
#[derive(Clone, Debug)]
pub struct NameBTree<V> {
    by_name: BTreeMap<Name, (String, V)>,
}

impl<V> Default for NameBTree<V> {
    fn default() -> Self {
        Self {
            by_name: BTreeMap::new(),
        }
    }
}

impl<V> NameBTree<V> {
    pub fn get(&self, spelling: &str) -> Option<&V> {
        let name = cached_enter(spelling)?;
        self.by_name.get(&name).map(|(_, v)| v)
    }

    pub fn insert(&mut self, spelling: String, value: V) -> Option<V> {
        self.insert_with(spelling, value, |_, _| None)
    }

    pub fn insert_with(
        &mut self,
        spelling: String,
        value: V,
        values_equal: impl FnOnce(&V, &V) -> Option<bool>,
    ) -> Option<V> {
        let name = must_name(&spelling);
        if let Some((kept, old)) = self.by_name.get(&name) {
            if kept != &spelling {
                let eq = values_equal(old, &value);
                record_collision(&name, kept, &spelling, eq);
            }
        }
        match self.by_name.get_mut(&name) {
            Some(slot) => Some(std::mem::replace(&mut slot.1, value)),
            None => {
                self.by_name.insert(name, (spelling, value));
                None
            }
        }
    }

    pub fn contains_key(&self, spelling: &str) -> bool {
        self.get(spelling).is_some()
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.by_name.values().map(|(s, _)| s)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &V)> {
        self.by_name.values().map(|(s, v)| (s, v))
    }
}
