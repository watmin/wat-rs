//! The EDN value model — closed under the full spec.

use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use compact_str::CompactString;
use num_bigint::BigInt;
use std::borrow::Cow;
use std::fmt;
use uuid::Uuid;

/// A parsed EDN value. Closed under the spec.
///
/// Maps are stored as `Vec<(K, V)>` rather than `HashMap` because EDN
/// permits any value as a key — including `f64`, which doesn't impl
/// `Eq + Hash`. Consumers that want hash-based lookup convert to
/// their preferred map type after reading.
///
/// Equality is per the EDN spec:
///   - Lists / Vectors: ordered, positional.
///   - Maps: unordered — same set of `(key, value)` entries by value.
///   - Sets: unordered — same multiset of elements by value.
///   - All other variants: structural equality on inner data.
///
/// `PartialEq` is hand-written (not derived) to honor the spec's
/// unordered-collection semantics for maps and sets.
#[derive(Debug, Clone)]
pub enum Value<'a> {
    Nil,
    Bool(bool),
    Integer(i64),
    /// Big integers ship in a `Box` so the `Value` enum stays small
    /// (cache-friendly for `Vec<Value>`). `BigInt` itself is ~32 B;
    /// boxing it shrinks the variant to one pointer.
    BigInt(Box<BigInt>),
    Float(f64),
    /// `BigDecimal` is `BigInt + scale` (~40 B); boxed for the same
    /// reason as `BigInt`.
    BigDec(Box<BigDecimal>),
    /// String body. Borrowed (`Cow::Borrowed`) when the lexer's
    /// fast path produced no escapes — zero-copy slice into the
    /// input buffer. Owned (`Cow::Owned`) when escapes forced
    /// allocation, or when a caller constructs the variant from
    /// an owned `String` directly. Use [`Value::into_owned`] to
    /// lift to `Value<'static>` for storage beyond `'a`.
    String(Cow<'a, str>),
    Char(char),
    Symbol(Symbol),
    Keyword(Keyword),
    List(Vec<Value<'a>>),
    Vector(Vec<Value<'a>>),
    Map(Vec<(Value<'a>, Value<'a>)>),
    Set(Vec<Value<'a>>),
    Tagged(Tag, Box<Value<'a>>),
    Inst(DateTime<Utc>),
    Uuid(Uuid),
}

impl<'a> PartialEq for Value<'a> {
    fn eq(&self, other: &Self) -> bool {
        use Value::*;
        match (self, other) {
            (Nil, Nil) => true,
            (Bool(a), Bool(b)) => a == b,
            (Integer(a), Integer(b)) => a == b,
            (BigInt(a), BigInt(b)) => a == b,
            (Float(a), Float(b)) => {
                // NaN == NaN for Value equality so round-trip-via-sentinel
                // tests can use assert_eq! on NaN. This is a conscious
                // departure from IEEE 754, scoped to Value::PartialEq.
                if a.is_nan() && b.is_nan() {
                    true
                } else {
                    a == b
                }
            }
            (BigDec(a), BigDec(b)) => a == b,
            (String(a), String(b)) => a == b,
            (Char(a), Char(b)) => a == b,
            (Symbol(a), Symbol(b)) => a == b,
            (Keyword(a), Keyword(b)) => a == b,
            (List(a), List(b)) => a == b,
            (Vector(a), Vector(b)) => a == b,
            (Tagged(t1, b1), Tagged(t2, b2)) => t1 == t2 && b1 == b2,
            (Inst(a), Inst(b)) => a == b,
            (Uuid(a), Uuid(b)) => a == b,
            (Map(a), Map(b)) => map_eq(a, b),
            (Set(a), Set(b)) => set_eq(a, b),
            _ => false,
        }
    }
}

/// Map equality per spec: same number of entries, every (k,v) in
/// `a` matches an entry in `b` and vice versa. O(n²) in the
/// pathological case but n is small (parsed maps are usually < 100
/// entries) and `Value` doesn't implement `Hash` so there is no
/// hash-based shortcut available.
fn map_eq<'a>(a: &[(Value<'a>, Value<'a>)], b: &[(Value<'a>, Value<'a>)]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut matched = vec![false; b.len()];
    for (ka, va) in a {
        let mut found = false;
        for (i, (kb, vb)) in b.iter().enumerate() {
            if !matched[i] && ka == kb && va == vb {
                matched[i] = true;
                found = true;
                break;
            }
        }
        if !found {
            return false;
        }
    }
    true
}

/// Set equality per spec: multiset equality. Same length, every
/// element in `a` has a matching element in `b`.
fn set_eq<'a>(a: &[Value<'a>], b: &[Value<'a>]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut matched = vec![false; b.len()];
    for x in a {
        let mut found = false;
        for (i, y) in b.iter().enumerate() {
            if !matched[i] && x == y {
                matched[i] = true;
                found = true;
                break;
            }
        }
        if !found {
            return false;
        }
    }
    true
}

impl<'a> Value<'a> {
    /// Consume `self` and produce an [`crate::OwnedValue`]
    /// (= `Value<'static>`). Borrowed string slices are copied
    /// into owned `String`s; already-owned data passes through.
    /// Recurses through containers.
    pub fn into_owned(self) -> crate::OwnedValue {
        match self {
            Value::Nil => Value::Nil,
            Value::Bool(b) => Value::Bool(b),
            Value::Integer(i) => Value::Integer(i),
            Value::BigInt(n) => Value::BigInt(n),
            Value::Float(f) => Value::Float(f),
            Value::BigDec(n) => Value::BigDec(n),
            Value::String(s) => Value::String(Cow::Owned(s.into_owned())),
            Value::Char(c) => Value::Char(c),
            Value::Symbol(s) => Value::Symbol(s),
            Value::Keyword(k) => Value::Keyword(k),
            Value::List(v) => Value::List(v.into_iter().map(Value::into_owned).collect()),
            Value::Vector(v) => Value::Vector(v.into_iter().map(Value::into_owned).collect()),
            Value::Map(m) => Value::Map(
                m.into_iter()
                    .map(|(k, v)| (k.into_owned(), v.into_owned()))
                    .collect(),
            ),
            Value::Set(s) => Value::Set(s.into_iter().map(Value::into_owned).collect()),
            Value::Tagged(t, b) => Value::Tagged(t, Box::new(b.into_owned())),
            Value::Inst(d) => Value::Inst(d),
            Value::Uuid(u) => Value::Uuid(u),
        }
    }
}

/// EDN symbol. Namespaced if `namespace().is_some()`.
///
/// Fields are private; constructors validate per spec ("Symbols begin
/// with a non-numeric character; if `-`/`+`/`.` are first, second
/// must be non-numeric"). Use `Symbol::new` / `Symbol::ns` for the
/// panic-on-invalid form, or `Symbol::try_new` / `Symbol::try_ns`
/// for fallible construction.
///
/// Body fields use `CompactString`, which inlines up to 24 bytes
/// (on 64-bit). Typical EDN identifiers (`:asset`, `wat.core/Vec`,
/// `enterprise.observer/TradeSignal`) fit inline → zero heap alloc.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Symbol {
    namespace: Option<CompactString>,
    name: CompactString,
}

/// EDN keyword. Namespaced if `namespace().is_some()`. Same
/// validation rules as `Symbol`. CompactString-inlined like Symbol.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Keyword {
    namespace: Option<CompactString>,
    name: CompactString,
}

/// EDN tag (the symbol part of `#tag value`). Per spec, user tags
/// MUST be namespaced — the type ENFORCES this. There is no
/// `Option` on `namespace`; there is no `Tag::new(name)` because a
/// no-namespace tag is invalid input. Build via `Tag::ns(namespace,
/// name)` (panics on invalid) or `Tag::try_ns` (returns Result).
///
/// CompactString-inlined like Symbol/Keyword.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Tag {
    namespace: CompactString,
    name: CompactString,
}

// ─── Symbol ─────────────────────────────────────────────────────

impl Symbol {
    /// Build a non-namespaced symbol. Panics if the name fails the
    /// spec first-character rule.
    ///
    /// Prefer [`Symbol::try_new`] for caller-supplied input; reach
    /// for `new` when the name is compile-time known and panicking
    /// on a typo is the right failure mode.
    #[track_caller]
    pub fn new(name: impl AsRef<str>) -> Self {
        let name = name.as_ref();
        crate::escapes::validate_first_char(name)
            .unwrap_or_else(|m| panic!("invalid symbol name {:?}: {}", name, m));
        Self { namespace: None, name: CompactString::from(name) }
    }

    /// Build a namespaced symbol. Panics if either segment fails
    /// the spec first-character rule. See [`Symbol::new`] for
    /// guidance on panic-vs-Result.
    #[track_caller]
    pub fn ns(namespace: impl AsRef<str>, name: impl AsRef<str>) -> Self {
        let namespace = namespace.as_ref();
        let name = name.as_ref();
        crate::escapes::validate_first_char(namespace)
            .unwrap_or_else(|m| panic!("invalid symbol namespace {:?}: {}", namespace, m));
        crate::escapes::validate_first_char(name)
            .unwrap_or_else(|m| panic!("invalid symbol name {:?}: {}", name, m));
        Self {
            namespace: Some(CompactString::from(namespace)),
            name: CompactString::from(name),
        }
    }

    /// Fallible non-namespaced symbol constructor. Returns a stable
    /// diagnostic string on rejection (not for programmatic
    /// dispatch — use [`crate::ErrorKind`] when you need typed
    /// errors from the parser path).
    pub fn try_new(name: impl AsRef<str>) -> std::result::Result<Self, &'static str> {
        let name = name.as_ref();
        crate::escapes::validate_first_char(name)?;
        Ok(Self { namespace: None, name: CompactString::from(name) })
    }

    /// Fallible namespaced symbol constructor.
    pub fn try_ns(
        namespace: impl AsRef<str>,
        name: impl AsRef<str>,
    ) -> std::result::Result<Self, &'static str> {
        let namespace = namespace.as_ref();
        let name = name.as_ref();
        crate::escapes::validate_first_char(namespace)?;
        crate::escapes::validate_first_char(name)?;
        Ok(Self {
            namespace: Some(CompactString::from(namespace)),
            name: CompactString::from(name),
        })
    }

    /// Crate-private constructor; trusts the caller (the parser,
    /// after `parse_namespaced`'s validation). Use
    /// [`Symbol::new`] / [`Symbol::ns`] / [`Symbol::try_new`] /
    /// [`Symbol::try_ns`] for any other code path.
    pub(crate) fn from_parts_unchecked(namespace: Option<&str>, name: &str) -> Self {
        Self {
            namespace: namespace.map(CompactString::from),
            name: CompactString::from(name),
        }
    }

    pub fn namespace(&self) -> Option<&str> {
        self.namespace.as_deref()
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

// ─── Keyword ────────────────────────────────────────────────────

impl Keyword {
    /// Build a non-namespaced keyword. Panics on invalid name.
    /// See [`Symbol::new`] for guidance on panic-vs-Result.
    #[track_caller]
    pub fn new(name: impl AsRef<str>) -> Self {
        let name = name.as_ref();
        crate::escapes::validate_first_char(name)
            .unwrap_or_else(|m| panic!("invalid keyword name {:?}: {}", name, m));
        Self { namespace: None, name: CompactString::from(name) }
    }

    /// Build a namespaced keyword. Panics on invalid name or namespace.
    #[track_caller]
    pub fn ns(namespace: impl AsRef<str>, name: impl AsRef<str>) -> Self {
        let namespace = namespace.as_ref();
        let name = name.as_ref();
        crate::escapes::validate_first_char(namespace)
            .unwrap_or_else(|m| panic!("invalid keyword namespace {:?}: {}", namespace, m));
        crate::escapes::validate_first_char(name)
            .unwrap_or_else(|m| panic!("invalid keyword name {:?}: {}", name, m));
        Self {
            namespace: Some(CompactString::from(namespace)),
            name: CompactString::from(name),
        }
    }

    /// Fallible non-namespaced keyword constructor. See
    /// [`Symbol::try_new`] for the error-type rationale.
    pub fn try_new(name: impl AsRef<str>) -> std::result::Result<Self, &'static str> {
        let name = name.as_ref();
        crate::escapes::validate_first_char(name)?;
        Ok(Self { namespace: None, name: CompactString::from(name) })
    }

    /// Fallible namespaced keyword constructor.
    pub fn try_ns(
        namespace: impl AsRef<str>,
        name: impl AsRef<str>,
    ) -> std::result::Result<Self, &'static str> {
        let namespace = namespace.as_ref();
        let name = name.as_ref();
        crate::escapes::validate_first_char(namespace)?;
        crate::escapes::validate_first_char(name)?;
        Ok(Self {
            namespace: Some(CompactString::from(namespace)),
            name: CompactString::from(name),
        })
    }

    /// Crate-private constructor; trusts the caller. Use
    /// [`Keyword::new`] / [`Keyword::ns`] / [`Keyword::try_new`] /
    /// [`Keyword::try_ns`] for any other code path.
    pub(crate) fn from_parts_unchecked(namespace: Option<&str>, name: &str) -> Self {
        Self {
            namespace: namespace.map(CompactString::from),
            name: CompactString::from(name),
        }
    }

    pub fn namespace(&self) -> Option<&str> {
        self.namespace.as_deref()
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

// ─── Tag ────────────────────────────────────────────────────────

impl Tag {
    /// Build a namespaced tag. Per the EDN spec, user tags MUST be
    /// namespaced — there is no `Tag::new(name)` because a no-namespace
    /// tag is invalid input. Panics on invalid name or namespace.
    #[track_caller]
    pub fn ns(namespace: impl AsRef<str>, name: impl AsRef<str>) -> Self {
        let namespace = namespace.as_ref();
        let name = name.as_ref();
        crate::escapes::validate_first_char(namespace)
            .unwrap_or_else(|m| panic!("invalid tag namespace {:?}: {}", namespace, m));
        crate::escapes::validate_first_char(name)
            .unwrap_or_else(|m| panic!("invalid tag name {:?}: {}", name, m));
        Self {
            namespace: CompactString::from(namespace),
            name: CompactString::from(name),
        }
    }

    /// Fallible namespaced tag constructor. Returns a stable
    /// diagnostic string on rejection (not for programmatic
    /// dispatch — use [`crate::ErrorKind`] from the parser path
    /// when you need typed errors).
    pub fn try_ns(
        namespace: impl AsRef<str>,
        name: impl AsRef<str>,
    ) -> std::result::Result<Self, &'static str> {
        let namespace = namespace.as_ref();
        let name = name.as_ref();
        crate::escapes::validate_first_char(namespace)?;
        crate::escapes::validate_first_char(name)?;
        Ok(Self {
            namespace: CompactString::from(namespace),
            name: CompactString::from(name),
        })
    }

    /// Crate-private constructor; trusts the caller (the parser,
    /// after `parse_namespaced`'s validation). Use
    /// [`Tag::ns`] or [`Tag::try_ns`] for any other code path.
    pub(crate) fn from_parts_unchecked(namespace: &str, name: &str) -> Self {
        Self {
            namespace: CompactString::from(namespace),
            name: CompactString::from(name),
        }
    }

    /// The tag's namespace prefix. Always present per spec; the
    /// type enforces this (no `Option`).
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

// ─── Display ────────────────────────────────────────────────────

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.namespace() {
            Some(ns) => write!(f, "{}/{}", ns, self.name()),
            None => f.write_str(self.name()),
        }
    }
}

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(":")?;
        match self.namespace() {
            Some(ns) => write!(f, "{}/{}", ns, self.name()),
            None => f.write_str(self.name()),
        }
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}/{}", self.namespace(), self.name())
    }
}

// ─── Convenience accessors ──────────────────────────────────────

impl<'a> Value<'a> {
    /// The variant name for diagnostics and error messages.
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Nil => "nil",
            Value::Bool(_) => "bool",
            Value::Integer(_) => "integer",
            Value::BigInt(_) => "bigint",
            Value::Float(_) => "float",
            Value::BigDec(_) => "bigdec",
            Value::String(_) => "string",
            Value::Char(_) => "char",
            Value::Symbol(_) => "symbol",
            Value::Keyword(_) => "keyword",
            Value::List(_) => "list",
            Value::Vector(_) => "vector",
            Value::Map(_) => "map",
            Value::Set(_) => "set",
            Value::Tagged(_, _) => "tagged",
            Value::Inst(_) => "inst",
            Value::Uuid(_) => "uuid",
        }
    }

    // ─── Convenience accessors ──────────────────────────────────
    //
    // Each `as_*` returns `Some(&inner)` when the variant matches,
    // `None` otherwise. Convenient for downstream consumers that
    // know the expected shape and want to short-circuit out of
    // wrong types without writing a full `match`.

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Integer(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_char(&self) -> Option<char> {
        match self {
            Value::Char(c) => Some(*c),
            _ => None,
        }
    }

    pub fn as_symbol(&self) -> Option<&Symbol> {
        match self {
            Value::Symbol(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_keyword(&self) -> Option<&Keyword> {
        match self {
            Value::Keyword(k) => Some(k),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&[Value<'a>]> {
        match self {
            Value::List(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_vector(&self) -> Option<&[Value<'a>]> {
        match self {
            Value::Vector(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_map(&self) -> Option<&[(Value<'a>, Value<'a>)]> {
        match self {
            Value::Map(m) => Some(m),
            _ => None,
        }
    }

    pub fn as_set(&self) -> Option<&[Value<'a>]> {
        match self {
            Value::Set(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_tagged(&self) -> Option<(&Tag, &Value<'a>)> {
        match self {
            Value::Tagged(t, b) => Some((t, b)),
            _ => None,
        }
    }

    pub fn as_inst(&self) -> Option<&DateTime<Utc>> {
        match self {
            Value::Inst(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_uuid(&self) -> Option<&Uuid> {
        match self {
            Value::Uuid(u) => Some(u),
            _ => None,
        }
    }

    /// `true` iff this value is `Value::Nil`.
    pub fn is_nil(&self) -> bool {
        matches!(self, Value::Nil)
    }
}
