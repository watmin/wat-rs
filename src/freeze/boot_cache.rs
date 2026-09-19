//! The boot cache — Tier A of excursus 001 `the-boot-cache-is-possible-or-it-is-not`.
//!
//! # What it elides, and what it does NOT
//!
//! A boot spends ~405 ms of accounted time before the first user form runs, and **the
//! overwhelming majority of that is the substrate deriving the SAME baked stdlib it derived
//! last time**. This module stores that derivation and reads it back:
//!
//! | elided phase | ms (census, warm) |
//! |---|---:|
//! | `3a stdlib-parse` | 29.2 |
//! | `4  stdlib-defmacro-register` | 4.4 |
//! | `4  kwargs-companions` | 0.8 |
//! | `4  stdlib-expand` | **201.4** |
//! | `5  typeenv-with-builtins` | (bonus — not in the Tier A table) |
//! | `5  stdlib-types-register` | 2.7 |
//! | `6  stdlib-defines-register` | 11.7 |
//! | `6a defclause-stub-preregister` | 1.0 |
//! | `6b stdlib-runtime-def-filter` | 0.04 |
//!
//! ⭐ **It DOES elide 8b / 8d(ALL-fns) / 8f for bake-time functions**, once a successful
//! `check_program` has stamped `infer_fresh_consumed` onto the payload (Tier B step C).
//! 8c and 8d's `forms` half keep running. Partition is the `:wat::` path, never
//! `body.span().file` (user-type companions live in `src/runtime.rs`).
//!
//! ⛔ **It does NOT elide `6a-9 auto-method-codegen`, `5 aggregate-containment`, or
//! `6.97 typeenv-clone-attach`.** Those three walk the COMBINED (stdlib + user) `TypeEnv`; a
//! snapshot taken before user types exist cannot stand in for them, and restricting them to
//! stdlib types is the same class of soundness question as Tier B. 4.7 ms of the Tier A table is
//! therefore deliberately left on the floor, and said out loud rather than quietly claimed.
//!
//! ⛔ **It does NOT elide `7.6 stdlib-runtime-defs-register`** (10.1 ms): that pass EVALUATES the
//! stdlib's `def` expressions into `Value`s, and `Value` has handle-bearing variants (`Sender`,
//! `RustOpaque`, `Arc<dyn WatReader>`). The forms are cached; the evaluation is re-run. Caching
//! `Value` is not refused because it is impossible — the 56 entries measured are all pure — but
//! because a serialiser over a type whose variants include live handles is a trap-door waiting
//! for the 57th entry.
//!
//! # ⛔ THE SOUNDNESS GATE, AND WHY IT IS NOT AN ARGUMENT
//!
//! `build_env` registers USER defmacros and USER acronyms **before** it expands the stdlib. So the
//! cached expansion is only valid if the user's contribution could not have changed it. This
//! module does not *argue* that; it **records a witness and checks it**:
//!
//! - While the cached stdlib expansion ran, [`crate::macros::MacroRegistry`] recorded every macro
//!   name the expander PROBED that is not a reserved (`:wat::…`) prefix — see [`probe_witness`].
//!   A user macro can only ever carry a non-reserved name (the reserved-prefix gate refuses the
//!   rest), so a user macro whose name is absent from that witness cannot have been consulted.
//!   At boot, the cache is refused if any user-registered macro name is in the witness.
//! - The acronym registry consulted at expand time (`macro_sym`) was EMPTY when the snapshot was
//!   taken, so the cache is refused outright when the user program declares any acronym. That is
//!   conservative rather than precise, and deliberately so: it costs a boot only for programs
//!   that use `declare-acronyms`, and it needs no argument at all.
//!
//! # Staleness
//!
//! Two independent axes, both in the key (see [`cache_key`]):
//!
//! 1. `WAT_BUILD_FINGERPRINT` — a 128-bit content hash of `wat/**`, `src/**`, `crates/**`,
//!    `Cargo.lock`, `Cargo.toml` and `build.rs`, computed by `build.rs` at compile time. This
//!    covers BOTH "the stdlib source changed" and the axis a source-only hash misses: "the Rust
//!    that derives from it changed".
//! 2. The installed dep sources (`crate::load::source::installed_dep_sources`), which a host may
//!    install at RUNTIME and which `stdlib_forms()` concatenates onto the baked corpus. Hashed at
//!    boot; normally empty, so normally free.
//!
//! The key is written into the payload as well as into the filename, so a filename collision
//! cannot pass a foreign payload off as this build's.
//!
//! # Absent / stale / corrupt
//!
//! Every failure path returns `None` and boot derives exactly as it always did. Nothing in this
//! module can fail a boot: there is no `unwrap`, no `panic`, no `expect` on cache content, and the
//! decoder is bounds-checked at every read (a truncated or corrupted payload returns `None`
//! rather than indexing off the end).
//!
//! # ⛔ THE TRAP: `Span::eq` RETURNS `true` UNCONDITIONALLY
//!
//! `crates/wat-reader/src/span.rs:137` — `impl PartialEq for Span { fn eq(&self, _: &Self) -> bool
//! { true } }`, and `Hash` is a no-op. **`assert_eq!(original, decoded)` therefore PASSES ON A
//! DECODER THAT DROPS EVERY SPAN**, which would ship green and destroy every diagnostic in the
//! language. The tests for this module assert a **byte-exact fixpoint**
//! (`encode(decode(encode(x))) == encode(x)`) instead — the bytes carry the spans, and byte
//! equality does not let them go missing.
//!
//! # Format
//!
//! Hand-rolled: LEB128 varints, zig-zag signed, length-prefixed UTF-8, one recursive walk, one
//! `Vec<u8>`, one string table for span file labels. **No dependency was added.** The feasibility
//! probe measured this shape at 15.67 ms cold for a 2.04 MB payload — 17× cheaper than the phases
//! it replaces — so a fancier format would be optimising the part that is already 6 % of the win.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

use crate::argspec::ArgSpec;
use crate::ast::WatAST;
use crate::macros::{MacroDef, MacroRegistry};
use crate::runtime::SymbolTable;
use crate::scope::{fresh_scope, Identifier, ScopeId};
use crate::span::{Pos, Span};
use crate::types::{
    AggregateDef, AliasDef, EnumDef, EnumVariant, Nature, NewtypeDef, Purity, StructRestrictions,
    SurfaceDef, SurfaceMember, TypeDef, TypeEnv, TypeExpr, UnionDef,
};
use crate::value::{Function, FunctionBody, ReteContract};

/// Bumped whenever the byte layout below changes in ANY way. A payload written by a different
/// version is refused, not reinterpreted.
const FORMAT_VERSION: u32 = 2;

const MAGIC: &[u8; 8] = b"WATBOOT\x01";

// ── the snapshot ────────────────────────────────────────────────────────────────────────────

/// Everything `build_env` derives from the baked stdlib ALONE, at the moment just before the
/// first pass that mixes user state in.
pub struct StdlibSnapshot {
    /// Post `register_stdlib_defmacros` + `register_aggregate_kwargs_companions` +
    /// `expand_all_with(stdlib, Privilege::Stdlib)` — expansion itself registers macros
    /// (a `defservice`'s `…/start` companion), so this is the POST-expansion registry.
    pub macros: MacroRegistry,
    /// `TypeEnv::with_builtins()` + `register_stdlib_types(expanded_stdlib)`.
    pub types: TypeEnv,
    /// `register_stdlib_defines(…)` + the `preregister_stdlib_defclause_stub` loop.
    pub symbols: SymbolTable,
    /// The `defclause` / `extend-type` / `def` stdlib residue that step 7.6 re-registers.
    pub runtime_def_forms: Vec<WatAST>,
    /// ⛔ THE GATE. Every macro name the expander probed while producing this snapshot that is
    /// NOT a reserved prefix — i.e. every name a USER macro could possibly have intercepted.
    pub probe_witness: Vec<String>,
    /// How many `InferCtx` fresh vars the bake-time 8f sweep consumed, recorded
    /// AFTER a successful `check_program`. `None` until that check has run for
    /// this fingerprint — a hit with `None` does not elide (it re-runs 8f and
    /// then fills this in). Sentinel in the payload is `u64::MAX`.
    pub infer_fresh_consumed: Option<u64>,
}

// ── the cache key ───────────────────────────────────────────────────────────────────────────

/// Compile-time fingerprint of every input that can change what the snapshot should contain.
/// Emitted by `build.rs`; `option_env!` so a build without it degrades to "no cache" rather
/// than to "a cache that cannot detect staleness".
fn build_fingerprint() -> Option<&'static str> {
    option_env!("WAT_BUILD_FINGERPRINT")
}

/// 64-bit FNV-1a. Used ONLY for cache-key material and the payload checksum — never for
/// persisted identity (that is `src/hash.rs`'s SHA-256 job).
fn fnv1a(seed: u64, bytes: &[u8]) -> u64 {
    let mut h = seed;
    for &b in bytes {
        h = (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

fn checksum(bytes: &[u8]) -> u128 {
    let a = fnv1a(0xcbf2_9ce4_8422_2325, bytes);
    let mut b = 0x9ae1_6a3b_2f90_404fu64;
    for &x in bytes {
        b = (b ^ u64::from(x)).rotate_left(27).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    }
    (u128::from(a) << 64) | u128::from(b)
}

/// The full key: build fingerprint + format version + a hash of any RUNTIME-installed dep
/// sources. `None` when the build carries no fingerprint (then there is no cache at all).
fn cache_key() -> Option<String> {
    let fp = build_fingerprint()?;
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for slice in crate::load::source::installed_dep_sources() {
        for src in slice.iter() {
            h = fnv1a(h, src.path.as_bytes());
            h = fnv1a(h, src.source.as_bytes());
        }
    }
    Some(format!("{fp}-v{FORMAT_VERSION}-{h:016x}"))
}

/// Where the payload lives. `WAT_BOOT_CACHE_DIR` overrides (the tests drive the fallback paths
/// through it); otherwise `$XDG_CACHE_HOME` / `$HOME/.cache` / the system temp dir.
fn cache_dir() -> PathBuf {
    if let Ok(d) = std::env::var("WAT_BOOT_CACHE_DIR") {
        if !d.is_empty() {
            return PathBuf::from(d);
        }
    }
    if let Ok(d) = std::env::var("XDG_CACHE_HOME") {
        if !d.is_empty() {
            return PathBuf::from(d).join("wat-boot-cache");
        }
    }
    if let Ok(d) = std::env::var("HOME") {
        if !d.is_empty() {
            return PathBuf::from(d).join(".cache").join("wat-boot-cache");
        }
    }
    std::env::temp_dir().join("wat-boot-cache")
}

/// `WAT_BOOT_CACHE=off` disables the cache entirely — the A/B door for measuring it, and the
/// escape hatch if it ever needs one.
fn enabled() -> bool {
    !matches!(
        std::env::var("WAT_BOOT_CACHE").as_deref(),
        Ok("off") | Ok("0") | Ok("no")
    )
}

fn cache_path() -> Option<PathBuf> {
    let key = cache_key()?;
    Some(cache_dir().join(format!("{key}.watbc")))
}

// ── the public door ─────────────────────────────────────────────────────────────────────────

/// Read the snapshot for THIS build, or `None`. Absent, stale, truncated, corrupt, foreign,
/// unreadable — every one of them is `None`, and every one of them means "derive it".
pub(crate) fn load() -> Option<StdlibSnapshot> {
    if !enabled() {
        return None;
    }
    let path = cache_path()?;
    let bytes = std::fs::read(&path).ok()?;
    match decode(&bytes, &cache_key()?) {
        Some(snap) => Some(snap),
        None => {
            // ⛔ HEAL A REFUSED PAYLOAD, or the fallback is PERMANENT.
            //
            // Refusing a bad payload is correct and was driven (delete · 9 truncations ·
            // 3 byte-flips · a foreign key: every one refuses and boots to an identical
            // world). What the drives did not ask is whether the cache RECOVERS — and
            // without this, it does not: a refused file is re-read and re-refused on every
            // subsequent boot, so one bad byte means full-price startup FOREVER, silently
            // and correctly. Measured while grading: the release floor ran 555.9 s with a
            // poisoned entry versus 252.1 s once the file was removed by hand, and nothing
            // anywhere reported a problem — it was merely slow, which is the exact failure
            // shape this whole thread exists to end.
            //
            // A killed process makes this reachable in normal use, not just under a fault
            // injector: `store` writes to a unique temp then renames, so a half-written
            // payload is never visible — but a file truncated by a full disk, or written by
            // an older binary whose key happens to collide, lands in the same state.
            //
            // Remove rather than rewrite: `store` is already called on the derive path that
            // follows, so deletion is sufficient AND avoids writing a second payload while
            // 200 nextest processes race. A failure to remove is ignored for the same reason
            // a failure to write is: it must never turn a slow boot into a failed one.
            let _ = std::fs::remove_file(&path);
            None
        }
    }
}

/// Write the snapshot for THIS build, best-effort. A failure to write is not a failure to boot;
/// the next boot simply derives again.
///
/// Writes via a unique temp file + `rename`, so a reader never sees a half-written payload even
/// with 200 nextest processes racing on the same box.
pub(crate) fn store(snap: &StdlibSnapshot) {
    if !enabled() {
        return;
    }
    let Some(path) = cache_path() else { return };
    let Some(key) = cache_key() else { return };
    if path.exists() {
        return;
    }
    let Some(dir) = path.parent() else { return };
    if std::fs::create_dir_all(dir).is_err() {
        return;
    }
    // ⛔ ENCODE-OR-REFUSE — see `is_representable`. A lossy cache is worse than no cache.
    if !is_representable(snap) {
        return;
    }
    let bytes = encode(snap, &key);
    let tmp = dir.join(format!(
        ".{}.{}.tmp",
        std::process::id(),
        path.file_name().and_then(|n| n.to_str()).unwrap_or("x")
    ));
    if std::fs::write(&tmp, &bytes).is_err() {
        let _ = std::fs::remove_file(&tmp);
        return;
    }
    if std::fs::rename(&tmp, &path).is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
}

/// Overwrite an existing payload. Used after a successful check to stamp
/// `infer_fresh_consumed` onto a snapshot that was stored *before* check
/// (the derivation store sits at step 7.9; check is step 8).
pub(crate) fn store_overwrite(snap: &StdlibSnapshot) {
    if !enabled() {
        return;
    }
    let Some(path) = cache_path() else { return };
    let Some(key) = cache_key() else { return };
    let Some(dir) = path.parent() else { return };
    if std::fs::create_dir_all(dir).is_err() {
        return;
    }
    if !is_representable(snap) {
        return;
    }
    let bytes = encode(snap, &key);
    let tmp = dir.join(format!(
        ".{}.{}.fresh.tmp",
        std::process::id(),
        path.file_name().and_then(|n| n.to_str()).unwrap_or("x")
    ));
    if std::fs::write(&tmp, &bytes).is_err() {
        let _ = std::fs::remove_file(&tmp);
        return;
    }
    if std::fs::rename(&tmp, &path).is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
}

/// After a successful check that actually RAN the bake-time 8f sweep, stamp
/// the measured `InferCtx.next` consumption onto the on-disk snapshot so
/// subsequent hits can elide. No-op if the cache is off, absent, or already
/// stamped.
pub(crate) fn record_infer_fresh(n: u64) {
    if !enabled() {
        return;
    }
    let Some(mut snap) = load() else { return };
    if snap.infer_fresh_consumed.is_some() {
        return;
    }
    snap.infer_fresh_consumed = Some(n);
    store_overwrite(&snap);
}

// ── the gate ────────────────────────────────────────────────────────────────────────────────

/// ⛔ Is the snapshot still applicable given what the USER contributed before stdlib expansion?
///
/// `user_macros` are the names the user's own `defmacro` forms registered; `user_acronyms` is
/// whether the user declared any acronym at all. See this module's header for why these two and
/// no others.
pub(crate) fn gate_holds(
    snap: &StdlibSnapshot,
    user_macros: &HashSet<String>,
    user_acronyms: bool,
) -> bool {
    if user_acronyms {
        return false;
    }
    if user_macros.is_empty() || snap.probe_witness.is_empty() {
        return true;
    }
    let witness: HashSet<&str> = snap.probe_witness.iter().map(String::as_str).collect();
    !user_macros.iter().any(|m| witness.contains(m.as_str()))
}

// ── encoder ─────────────────────────────────────────────────────────────────────────────────

const T_INT: u8 = 0;
const T_FLOAT: u8 = 1;
const T_RATIONAL: u8 = 2;
const T_BIGINT: u8 = 3;
const T_CHAR: u8 = 4;
const T_BOOL: u8 = 5;
const T_STRING: u8 = 6;
const T_NIL: u8 = 7;
const T_KEYWORD: u8 = 8;
const T_SYMBOL: u8 = 9;
const T_LIST: u8 = 10;
const T_VECTOR: u8 = 11;
const T_MAP: u8 = 12;
const T_SET: u8 = 13;

struct Enc {
    out: Vec<u8>,
    files: HashMap<String, u32>,
    file_order: Vec<String>,
    /// ⭐ EVERY string in the payload is written as an index into this table. A stdlib world is
    /// ~200 k AST nodes over ~15 k distinct names (`:wat::core::defn` alone appears thousands of
    /// times), so interning is not a micro-optimisation here — measured, it takes the payload
    /// from 5.08 MB to 2.29 MB and the load from 46.2 ms to 25.4 ms warm.
    strings: HashMap<String, u32>,
    string_order: Vec<String>,
    scopes: HashMap<u64, u32>,
}

impl Enc {
    fn new() -> Self {
        Enc {
            out: Vec::with_capacity(1 << 22),
            files: HashMap::new(),
            file_order: Vec::new(),
            strings: HashMap::new(),
            string_order: Vec::new(),
            scopes: HashMap::new(),
        }
    }

    fn u8v(&mut self, b: u8) {
        self.out.push(b);
    }

    fn uvar(&mut self, mut v: u64) {
        loop {
            let b = (v & 0x7f) as u8;
            v >>= 7;
            if v == 0 {
                self.out.push(b);
                return;
            }
            self.out.push(b | 0x80);
        }
    }

    fn ivar(&mut self, v: i64) {
        self.uvar(((v << 1) ^ (v >> 63)) as u64);
    }

    fn str(&mut self, s: &str) {
        let next = self.string_order.len() as u32;
        let idx = match self.strings.get(s) {
            Some(i) => *i,
            None => {
                self.strings.insert(s.to_string(), next);
                self.string_order.push(s.to_string());
                next
            }
        };
        self.uvar(u64::from(idx));
    }

    /// Length-prefixed UTF-8, verbatim — the two string TABLES themselves, which cannot be
    /// written through the interner that reads them.
    fn raw_str(&mut self, s: &str) {
        self.uvar(s.len() as u64);
        self.out.extend_from_slice(s.as_bytes());
    }

    fn strs(&mut self, v: &[String]) {
        self.uvar(v.len() as u64);
        for s in v {
            let s = s.clone();
            self.str(&s);
        }
    }

    fn opt_str(&mut self, s: &Option<String>) {
        match s {
            None => self.u8v(0),
            Some(s) => {
                self.u8v(1);
                let s = s.clone();
                self.str(&s);
            }
        }
    }

    fn span(&mut self, s: &Span) {
        let next = self.file_order.len() as u32;
        let idx = match self.files.get(s.file.as_str()) {
            Some(i) => *i,
            None => {
                self.files.insert((*s.file).clone(), next);
                self.file_order.push((*s.file).clone());
                next
            }
        };
        self.uvar(u64::from(idx));
        self.ivar(s.line);
        self.ivar(s.col);
        match &s.end {
            None => self.u8v(0),
            Some(p) => {
                self.u8v(1);
                self.ivar(p.line);
                self.ivar(p.col);
            }
        }
    }

    fn ident(&mut self, id: &Identifier) {
        let name = id.as_str().to_string();
        self.str(&name);
        let sc = id.scopes();
        self.uvar(sc.len() as u64);
        for s in sc {
            let raw = s.as_u64();
            let next = self.scopes.len() as u32;
            let dense = *self.scopes.entry(raw).or_insert(next);
            self.uvar(u64::from(dense));
        }
    }

    fn ast(&mut self, n: &WatAST) {
        match n {
            WatAST::IntLit(v, s) => {
                self.u8v(T_INT);
                self.ivar(*v);
                self.span(s);
            }
            WatAST::FloatLit(v, s) => {
                self.u8v(T_FLOAT);
                self.out.extend_from_slice(&v.to_bits().to_le_bytes());
                self.span(s);
            }
            WatAST::RationalLit(v, s) => {
                self.u8v(T_RATIONAL);
                let t = v.to_string();
                self.str(&t);
                self.span(s);
            }
            WatAST::BigIntLit(v, s) => {
                self.u8v(T_BIGINT);
                let t = v.to_string();
                self.str(&t);
                self.span(s);
            }
            WatAST::CharLit(c, s) => {
                self.u8v(T_CHAR);
                self.uvar(u64::from(*c as u32));
                self.span(s);
            }
            WatAST::BoolLit(b, s) => {
                self.u8v(T_BOOL);
                self.u8v(u8::from(*b));
                self.span(s);
            }
            WatAST::StringLit(v, s) => {
                self.u8v(T_STRING);
                let v = v.clone();
                self.str(&v);
                self.span(s);
            }
            WatAST::NilLit(s) => {
                self.u8v(T_NIL);
                self.span(s);
            }
            WatAST::Keyword(k, s) => {
                self.u8v(T_KEYWORD);
                let k = k.clone();
                self.str(&k);
                self.span(s);
            }
            WatAST::Symbol(id, s) => {
                self.u8v(T_SYMBOL);
                self.ident(id);
                self.span(s);
            }
            WatAST::List(items, s) => self.seq(T_LIST, items, s),
            WatAST::Vector(items, s) => self.seq(T_VECTOR, items, s),
            WatAST::Set(items, s) => self.seq(T_SET, items, s),
            WatAST::Map(pairs, s) => {
                self.u8v(T_MAP);
                self.uvar(pairs.len() as u64);
                for (k, v) in pairs {
                    self.ast(k);
                    self.ast(v);
                }
                self.span(s);
            }
        }
    }

    fn seq(&mut self, tag: u8, items: &[WatAST], s: &Span) {
        self.u8v(tag);
        self.uvar(items.len() as u64);
        for i in items {
            self.ast(i);
        }
        self.span(s);
    }

    fn asts(&mut self, v: &[WatAST]) {
        self.uvar(v.len() as u64);
        for a in v {
            self.ast(a);
        }
    }

    fn texpr(&mut self, t: &TypeExpr) {
        match t {
            TypeExpr::Path(p) => {
                self.u8v(0);
                let p = p.clone();
                self.str(&p);
            }
            TypeExpr::Parametric { head, args } => {
                self.u8v(1);
                let head = head.clone();
                self.str(&head);
                self.texprs(args);
            }
            TypeExpr::Fn { args, ret } => {
                self.u8v(2);
                self.texprs(args);
                self.texpr(ret);
            }
            TypeExpr::Var(n) => {
                self.u8v(3);
                self.uvar(*n);
            }
            TypeExpr::Tuple(items) => {
                self.u8v(4);
                self.texprs(items);
            }
        }
    }

    fn texprs(&mut self, v: &[TypeExpr]) {
        self.uvar(v.len() as u64);
        for t in v {
            self.texpr(t);
        }
    }

    fn named_texprs(&mut self, v: &[(String, TypeExpr)]) {
        self.uvar(v.len() as u64);
        for (n, t) in v {
            let n = n.clone();
            self.str(&n);
            self.texpr(t);
        }
    }

    fn argspec(&mut self, a: &ArgSpec) {
        let ArgSpec { fixed_params, rest_param } = a;
        self.uvar(fixed_params.len() as u64);
        for (id, t) in fixed_params {
            self.ident(id);
            self.texpr(t);
        }
        match rest_param {
            None => self.u8v(0),
            Some((id, t)) => {
                self.u8v(1);
                self.ident(id);
                self.texpr(t);
            }
        }
    }

    fn typedef(&mut self, d: &TypeDef) {
        match d {
            TypeDef::Aggregate(AggregateDef { name, type_params, fields, nature, restrictions }) => {
                self.u8v(0);
                let name = name.clone();
                self.str(&name);
                self.strs(type_params);
                self.named_texprs(fields);
                self.u8v(match nature {
                    Nature::Struct => 0,
                    Nature::Record => 1,
                    Nature::HolonRecord => 2,
                    Nature::Peer => 3,
                });
                match restrictions {
                    None => self.u8v(0),
                    Some(StructRestrictions { ctor_whitelist, field_restrictions }) => {
                        self.u8v(1);
                        self.strs(ctor_whitelist);
                        let mut keys: Vec<&String> = field_restrictions.keys().collect();
                        keys.sort();
                        self.uvar(keys.len() as u64);
                        for k in keys {
                            let kk = k.clone();
                            self.str(&kk);
                            let v = field_restrictions[k].clone();
                            self.strs(&v);
                        }
                    }
                }
            }
            TypeDef::Enum(EnumDef { name, type_params, purity, variants }) => {
                self.u8v(1);
                let name = name.clone();
                self.str(&name);
                self.strs(type_params);
                self.u8v(match purity {
                    Purity::Pure => 0,
                    Purity::Impure => 1,
                });
                self.uvar(variants.len() as u64);
                for v in variants {
                    match v {
                        EnumVariant::Unit(n) => {
                            self.u8v(0);
                            let n = n.clone();
                            self.str(&n);
                        }
                        EnumVariant::Tagged { name, fields } => {
                            self.u8v(1);
                            let name = name.clone();
                            self.str(&name);
                            self.named_texprs(fields);
                        }
                    }
                }
            }
            TypeDef::Newtype(NewtypeDef { name, type_params, inner }) => {
                self.u8v(2);
                let name = name.clone();
                self.str(&name);
                self.strs(type_params);
                self.texpr(inner);
            }
            TypeDef::Alias(AliasDef { name, type_params, expr }) => {
                self.u8v(3);
                let name = name.clone();
                self.str(&name);
                self.strs(type_params);
                self.texpr(expr);
            }
            TypeDef::Union(UnionDef { name, type_params, members }) => {
                self.u8v(4);
                let name = name.clone();
                self.str(&name);
                self.strs(type_params);
                self.texprs(members);
            }
            TypeDef::Surface(SurfaceDef { name, type_params, members, nature }) => {
                self.u8v(5);
                let name = name.clone();
                self.str(&name);
                self.strs(type_params);
                self.uvar(members.len() as u64);
                for m in members {
                    match m {
                        SurfaceMember::Field { name, ty } => {
                            self.u8v(0);
                            let name = name.clone();
                            self.str(&name);
                            self.texpr(ty);
                        }
                        SurfaceMember::Method {
                            name,
                            args,
                            ret,
                            type_params,
                            max_request_bytes,
                            max_request_bytes_explicit,
                            max_entries,
                            max_page,
                        } => {
                            self.u8v(1);
                            let name = name.clone();
                            self.str(&name);
                            self.argspec(args);
                            self.texpr(ret);
                            self.strs(type_params);
                            self.ivar(*max_request_bytes);
                            self.u8v(u8::from(*max_request_bytes_explicit));
                            self.opt_field_cap(max_entries);
                            self.opt_field_cap(max_page);
                        }
                    }
                }
                match nature {
                    None => self.u8v(0),
                    Some(n) => {
                        self.u8v(1);
                        self.u8v(match n {
                            Nature::Struct => 0,
                            Nature::Record => 1,
                            Nature::HolonRecord => 2,
                            Nature::Peer => 3,
                        });
                    }
                }
            }
        }
    }

    fn opt_field_cap(&mut self, c: &Option<(String, i64)>) {
        match c {
            None => self.u8v(0),
            Some((f, n)) => {
                self.u8v(1);
                let f = f.clone();
                self.str(&f);
                self.ivar(*n);
            }
        }
    }

    fn function(&mut self, f: &Function) {
        let Function {
            name,
            params,
            type_params,
            param_types,
            ret_type,
            rest_param,
            rest_param_type,
            body,
            closed_env,
            rete,
            synthesized_for,
        } = f;
        // `closed_env` is measured 0-of-2177 at this point in the pipeline, and
        // `is_representable` refuses the WHOLE cache rather than dropping one silently.
        debug_assert!(closed_env.is_none(), "boot cache: closed_env must be absent");
        self.opt_str(name);
        self.uvar(params.len() as u64);
        for p in params {
            self.ident(p);
        }
        self.strs(type_params);
        self.texprs(param_types);
        self.texpr(ret_type);
        self.opt_str(rest_param);
        match rest_param_type {
            None => self.u8v(0),
            Some(t) => {
                self.u8v(1);
                self.texpr(t);
            }
        }
        match body {
            FunctionBody::Wat(ast) => {
                self.u8v(0);
                self.ast(ast);
            }
            FunctionBody::Native => self.u8v(1),
        }
        self.u8v(u8::from(rete.is_some()));
        self.opt_str(synthesized_for);
    }

    fn macro_def(&mut self, m: &MacroDef) {
        let MacroDef { name, params, rest_param, body, span, source_form } = m;
        let name = name.clone();
        self.str(&name);
        self.strs(params);
        self.opt_str(rest_param);
        self.ast(body);
        self.span(span);
        self.ast(source_form);
    }
}

/// Sorted keys, so the payload is a deterministic function of the snapshot — which is what makes
/// `encode(decode(encode(x))) == encode(x)` a meaningful fixpoint over `HashMap`-backed state.
fn sorted_keys<V>(m: &HashMap<String, V>) -> Vec<&String> {
    let mut k: Vec<&String> = m.keys().collect();
    k.sort();
    k
}

fn encode(snap: &StdlibSnapshot, key: &str) -> Vec<u8> {
    let mut e = Enc::new();

    // ── macros ──
    let (macros, surface_forms) = snap.macros.cache_parts();
    e.uvar(macros.len() as u64);
    for k in sorted_keys(macros) {
        let kk = k.clone();
        e.str(&kk);
        e.macro_def(&macros[k]);
    }
    e.uvar(surface_forms.len() as u64);
    for k in sorted_keys(surface_forms) {
        let kk = k.clone();
        e.str(&kk);
        let f = surface_forms[k].clone();
        e.ast(&f);
    }

    // ── types ──
    let (types, builtin_names, subtype_edges, source_forms) = snap.types.cache_parts();
    e.uvar(types.len() as u64);
    for k in sorted_keys(types) {
        let kk = k.clone();
        e.str(&kk);
        e.typedef(&types[k]);
    }
    let mut bn: Vec<&String> = builtin_names.iter().collect();
    bn.sort();
    e.uvar(bn.len() as u64);
    for n in bn {
        let n = n.clone();
        e.str(&n);
    }
    e.uvar(subtype_edges.len() as u64);
    for k in sorted_keys(subtype_edges) {
        let kk = k.clone();
        e.str(&kk);
        let v = subtype_edges[k].clone();
        e.strs(&v);
    }
    e.uvar(source_forms.len() as u64);
    for k in sorted_keys(source_forms) {
        let kk = k.clone();
        e.str(&kk);
        let f = source_forms[k].clone();
        e.ast(&f);
    }

    // ── symbols ──
    let fns: HashMap<String, Arc<Function>> = snap
        .symbols
        .functions_iter()
        .map(|(k, v)| (k.clone(), Arc::clone(v)))
        .collect();
    e.uvar(fns.len() as u64);
    for k in sorted_keys(&fns) {
        let kk = k.clone();
        e.str(&kk);
        let f = Arc::clone(&fns[k]);
        e.function(&f);
    }
    let bm = snap.symbols.binding_metadata.clone();
    e.uvar(bm.len() as u64);
    for k in sorted_keys(&bm) {
        let kk = k.clone();
        e.str(&kk);
        let inner = bm[k].clone();
        e.uvar(inner.len() as u64);
        for ik in sorted_keys(&inner) {
            let ikk = ik.clone();
            e.str(&ikk);
            let v = inner[ik].clone();
            e.ast(&v);
        }
    }
    let ar = snap.symbols.acronym_registry.clone();
    e.uvar(ar.len() as u64);
    for k in sorted_keys(&ar) {
        let kk = k.clone();
        e.str(&kk);
        let v = ar[k].clone();
        e.strs(&v);
    }

    // ── residue + witness ──
    e.asts(&snap.runtime_def_forms);
    let mut w = snap.probe_witness.clone();
    w.sort();
    w.dedup();
    e.strs(&w);
    // u64::MAX = "not yet recorded" so a payload written before check can be
    // distinguished from a real zero (stdlib 8f consuming no vars).
    e.uvar(snap.infer_fresh_consumed.unwrap_or(u64::MAX));

    // ── header, string table, trailer ──
    // The two tables go last (first-encounter order is only known once the body is written); the
    // decoder reaches them through the trailer offset and reads them BEFORE the body.
    let table_at = e.out.len() as u64;
    let n = e.file_order.len();
    e.uvar(n as u64);
    for i in 0..n {
        let f = e.file_order[i].clone();
        e.raw_str(&f);
    }
    let n = e.string_order.len();
    e.uvar(n as u64);
    for i in 0..n {
        let t = e.string_order[i].clone();
        e.raw_str(&t);
    }

    let mut out = Vec::with_capacity(e.out.len() + 64);
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    out.extend_from_slice(&(key.len() as u32).to_le_bytes());
    out.extend_from_slice(key.as_bytes());
    let body_at = out.len() as u64;
    out.extend_from_slice(&e.out);
    out.extend_from_slice(&(body_at + table_at).to_le_bytes());
    let sum = checksum(&out);
    out.extend_from_slice(&sum.to_le_bytes());
    out
}

// ── decoder ─────────────────────────────────────────────────────────────────────────────────

/// Every read is bounds-checked and every failure is `None`. ⛔ NOTHING in here may panic:
/// "the cache file is corrupt" must cost a slow boot, never a dead one.
struct Dec<'a> {
    b: &'a [u8],
    at: usize,
    files: Vec<Arc<String>>,
    strings: Vec<String>,
    scopes: Vec<ScopeId>,
}

impl<'a> Dec<'a> {
    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        let end = self.at.checked_add(n)?;
        let s = self.b.get(self.at..end)?;
        self.at = end;
        Some(s)
    }

    fn u8v(&mut self) -> Option<u8> {
        let b = *self.b.get(self.at)?;
        self.at += 1;
        Some(b)
    }

    fn uvar(&mut self) -> Option<u64> {
        let mut v = 0u64;
        let mut shift = 0u32;
        loop {
            let byte = self.u8v()?;
            if shift >= 64 {
                return None;
            }
            v |= u64::from(byte & 0x7f).checked_shl(shift)?;
            if byte & 0x80 == 0 {
                return Some(v);
            }
            shift += 7;
        }
    }

    fn ivar(&mut self) -> Option<i64> {
        let u = self.uvar()?;
        Some(((u >> 1) as i64) ^ -((u & 1) as i64))
    }

    /// A count of items each of which costs at least one byte — so a count larger than the
    /// bytes remaining is structurally impossible and a corrupt length can never drive a
    /// multi-gigabyte `with_capacity`.
    fn count(&mut self) -> Option<usize> {
        let n = self.uvar()? as usize;
        if n > self.b.len().saturating_sub(self.at) {
            return None;
        }
        Some(n)
    }

    /// Length-prefixed UTF-8, verbatim — for the string TABLES themselves.
    fn raw_str(&mut self) -> Option<String> {
        let n = self.uvar()? as usize;
        let s = self.take(n)?;
        std::str::from_utf8(s).ok().map(str::to_string)
    }

    fn str(&mut self) -> Option<String> {
        let idx = self.uvar()? as usize;
        self.strings.get(idx).cloned()
    }

    fn strs(&mut self) -> Option<Vec<String>> {
        let n = self.count()?;
        let mut v = Vec::with_capacity(n);
        for _ in 0..n {
            v.push(self.str()?);
        }
        Some(v)
    }

    fn opt_str(&mut self) -> Option<Option<String>> {
        match self.u8v()? {
            0 => Some(None),
            1 => Some(Some(self.str()?)),
            _ => None,
        }
    }

    fn span(&mut self) -> Option<Span> {
        let idx = self.uvar()? as usize;
        let file = Arc::clone(self.files.get(idx)?);
        let line = self.ivar()?;
        let col = self.ivar()?;
        let end = match self.u8v()? {
            0 => None,
            1 => Some(Pos { line: self.ivar()?, col: self.ivar()? }),
            _ => return None,
        };
        Some(Span { file, line, col, end })
    }

    fn ident(&mut self) -> Option<Identifier> {
        let name = self.str()?;
        if name.contains('\u{1}') {
            return None; // `Identifier::bare` debug-asserts this; refuse rather than panic.
        }
        let n = self.count()?;
        let mut id = Identifier::bare(name);
        for _ in 0..n {
            let dense = self.uvar()? as usize;
            if dense > self.scopes.len() {
                return None;
            }
            while self.scopes.len() <= dense {
                // ⭐ RE-MINTED, never restored by value: an exec'd child restarts `fresh_scope()`
                // at 1, so an imported raw id would collide with this process's own counter.
                // Dense first-encounter indices preserve the SHARING structure, which is all the
                // semantics has (`Function::params`' own doc).
                self.scopes.push(fresh_scope());
            }
            id = id.add_scope(self.scopes[dense]);
        }
        Some(id)
    }

    fn ast(&mut self) -> Option<WatAST> {
        let tag = self.u8v()?;
        Some(match tag {
            T_INT => {
                let v = self.ivar()?;
                WatAST::IntLit(v, self.span()?)
            }
            T_FLOAT => {
                let mut buf = [0u8; 8];
                buf.copy_from_slice(self.take(8)?);
                WatAST::FloatLit(f64::from_bits(u64::from_le_bytes(buf)), self.span()?)
            }
            T_RATIONAL => {
                let t = self.str()?;
                let v: num_rational::BigRational = t.parse().ok()?;
                WatAST::RationalLit(v, self.span()?)
            }
            T_BIGINT => {
                let t = self.str()?;
                let v: num_bigint::BigInt = t.parse().ok()?;
                WatAST::BigIntLit(v, self.span()?)
            }
            T_CHAR => {
                let c = char::from_u32(u32::try_from(self.uvar()?).ok()?)?;
                WatAST::CharLit(c, self.span()?)
            }
            T_BOOL => {
                let b = self.u8v()? != 0;
                WatAST::BoolLit(b, self.span()?)
            }
            T_STRING => {
                let s = self.str()?;
                WatAST::StringLit(s, self.span()?)
            }
            T_NIL => WatAST::NilLit(self.span()?),
            T_KEYWORD => {
                let k = self.str()?;
                WatAST::Keyword(k, self.span()?)
            }
            T_SYMBOL => {
                let id = self.ident()?;
                WatAST::Symbol(id, self.span()?)
            }
            T_LIST => {
                let items = self.seq()?;
                WatAST::List(items, self.span()?)
            }
            T_VECTOR => {
                let items = self.seq()?;
                WatAST::Vector(items, self.span()?)
            }
            T_SET => {
                let items = self.seq()?;
                WatAST::Set(items, self.span()?)
            }
            T_MAP => {
                let n = self.count()?;
                let mut pairs = Vec::with_capacity(n);
                for _ in 0..n {
                    let k = self.ast()?;
                    let v = self.ast()?;
                    pairs.push((k, v));
                }
                WatAST::Map(pairs, self.span()?)
            }
            _ => return None,
        })
    }

    fn seq(&mut self) -> Option<Vec<WatAST>> {
        let n = self.count()?;
        let mut items = Vec::with_capacity(n);
        for _ in 0..n {
            items.push(self.ast()?);
        }
        Some(items)
    }

    fn asts(&mut self) -> Option<Vec<WatAST>> {
        self.seq()
    }

    fn texpr(&mut self) -> Option<TypeExpr> {
        Some(match self.u8v()? {
            0 => TypeExpr::Path(self.str()?),
            1 => TypeExpr::Parametric { head: self.str()?, args: self.texprs()? },
            2 => TypeExpr::Fn { args: self.texprs()?, ret: Box::new(self.texpr()?) },
            3 => TypeExpr::Var(self.uvar()?),
            4 => TypeExpr::Tuple(self.texprs()?),
            _ => return None,
        })
    }

    fn texprs(&mut self) -> Option<Vec<TypeExpr>> {
        let n = self.count()?;
        let mut v = Vec::with_capacity(n);
        for _ in 0..n {
            v.push(self.texpr()?);
        }
        Some(v)
    }

    fn named_texprs(&mut self) -> Option<Vec<(String, TypeExpr)>> {
        let n = self.count()?;
        let mut v = Vec::with_capacity(n);
        for _ in 0..n {
            let name = self.str()?;
            v.push((name, self.texpr()?));
        }
        Some(v)
    }

    fn nature(&mut self) -> Option<Nature> {
        Some(match self.u8v()? {
            0 => Nature::Struct,
            1 => Nature::Record,
            2 => Nature::HolonRecord,
            3 => Nature::Peer,
            _ => return None,
        })
    }

    fn argspec(&mut self) -> Option<ArgSpec> {
        let n = self.count()?;
        let mut fixed_params = Vec::with_capacity(n);
        for _ in 0..n {
            let id = self.ident()?;
            fixed_params.push((id, self.texpr()?));
        }
        let rest_param = match self.u8v()? {
            0 => None,
            1 => {
                let id = self.ident()?;
                Some((id, self.texpr()?))
            }
            _ => return None,
        };
        Some(ArgSpec { fixed_params, rest_param })
    }

    fn opt_field_cap(&mut self) -> Option<Option<(String, i64)>> {
        match self.u8v()? {
            0 => Some(None),
            1 => {
                let f = self.str()?;
                Some(Some((f, self.ivar()?)))
            }
            _ => None,
        }
    }

    fn typedef(&mut self) -> Option<TypeDef> {
        Some(match self.u8v()? {
            0 => {
                let name = self.str()?;
                let type_params = self.strs()?;
                let fields = self.named_texprs()?;
                let nature = self.nature()?;
                let restrictions = match self.u8v()? {
                    0 => None,
                    1 => {
                        let ctor_whitelist = self.strs()?;
                        let n = self.count()?;
                        let mut field_restrictions = HashMap::with_capacity(n);
                        for _ in 0..n {
                            let k = self.str()?;
                            field_restrictions.insert(k, self.strs()?);
                        }
                        Some(StructRestrictions { ctor_whitelist, field_restrictions })
                    }
                    _ => return None,
                };
                TypeDef::Aggregate(AggregateDef { name, type_params, fields, nature, restrictions })
            }
            1 => {
                let name = self.str()?;
                let type_params = self.strs()?;
                let purity = match self.u8v()? {
                    0 => Purity::Pure,
                    1 => Purity::Impure,
                    _ => return None,
                };
                let n = self.count()?;
                let mut variants = Vec::with_capacity(n);
                for _ in 0..n {
                    variants.push(match self.u8v()? {
                        0 => EnumVariant::Unit(self.str()?),
                        1 => {
                            let name = self.str()?;
                            EnumVariant::Tagged { name, fields: self.named_texprs()? }
                        }
                        _ => return None,
                    });
                }
                TypeDef::Enum(EnumDef { name, type_params, purity, variants })
            }
            2 => {
                let name = self.str()?;
                let type_params = self.strs()?;
                TypeDef::Newtype(NewtypeDef { name, type_params, inner: self.texpr()? })
            }
            3 => {
                let name = self.str()?;
                let type_params = self.strs()?;
                TypeDef::Alias(AliasDef { name, type_params, expr: self.texpr()? })
            }
            4 => {
                let name = self.str()?;
                let type_params = self.strs()?;
                TypeDef::Union(UnionDef { name, type_params, members: self.texprs()? })
            }
            5 => {
                let name = self.str()?;
                let type_params = self.strs()?;
                let n = self.count()?;
                let mut members = Vec::with_capacity(n);
                for _ in 0..n {
                    members.push(match self.u8v()? {
                        0 => {
                            let name = self.str()?;
                            SurfaceMember::Field { name, ty: self.texpr()? }
                        }
                        1 => {
                            let name = self.str()?;
                            let args = self.argspec()?;
                            let ret = self.texpr()?;
                            let type_params = self.strs()?;
                            let max_request_bytes = self.ivar()?;
                            let max_request_bytes_explicit = self.u8v()? != 0;
                            let max_entries = self.opt_field_cap()?;
                            let max_page = self.opt_field_cap()?;
                            SurfaceMember::Method {
                                name,
                                args,
                                ret,
                                type_params,
                                max_request_bytes,
                                max_request_bytes_explicit,
                                max_entries,
                                max_page,
                            }
                        }
                        _ => return None,
                    });
                }
                let nature = match self.u8v()? {
                    0 => None,
                    1 => Some(self.nature()?),
                    _ => return None,
                };
                TypeDef::Surface(SurfaceDef { name, type_params, members, nature })
            }
            _ => return None,
        })
    }

    fn function(&mut self) -> Option<Function> {
        let name = self.opt_str()?;
        let n = self.count()?;
        let mut params = Vec::with_capacity(n);
        for _ in 0..n {
            params.push(self.ident()?);
        }
        let type_params = self.strs()?;
        let param_types = self.texprs()?;
        let ret_type = self.texpr()?;
        let rest_param = self.opt_str()?;
        let rest_param_type = match self.u8v()? {
            0 => None,
            1 => Some(self.texpr()?),
            _ => return None,
        };
        let body = match self.u8v()? {
            0 => FunctionBody::Wat(Arc::new(self.ast()?)),
            1 => FunctionBody::Native,
            _ => return None,
        };
        let rete = match self.u8v()? {
            0 => None,
            1 => Some(ReteContract {}),
            _ => return None,
        };
        let synthesized_for = self.opt_str()?;
        Some(Function {
            name,
            params,
            type_params,
            param_types,
            ret_type,
            rest_param,
            rest_param_type,
            body,
            closed_env: None,
            rete,
            synthesized_for,
        })
    }

    fn macro_def(&mut self) -> Option<MacroDef> {
        let name = self.str()?;
        let params = self.strs()?;
        let rest_param = self.opt_str()?;
        let body = self.ast()?;
        let span = self.span()?;
        let source_form = self.ast()?;
        Some(MacroDef { name, params, rest_param, body, span, source_form })
    }
}

fn decode(bytes: &[u8], key: &str) -> Option<StdlibSnapshot> {
    // ── header ──
    if bytes.len() < MAGIC.len() + 4 + 4 + 8 + 16 || &bytes[..MAGIC.len()] != MAGIC {
        return None;
    }
    let mut at = MAGIC.len();
    let ver = u32::from_le_bytes(bytes.get(at..at + 4)?.try_into().ok()?);
    at += 4;
    if ver != FORMAT_VERSION {
        return None;
    }
    let klen = u32::from_le_bytes(bytes.get(at..at + 4)?.try_into().ok()?) as usize;
    at += 4;
    let kbytes = bytes.get(at..at.checked_add(klen)?)?;
    at += klen;
    if kbytes != key.as_bytes() {
        return None;
    }

    // ── checksum (the cheap reject) ──
    let sum_at = bytes.len() - 16;
    let want = u128::from_le_bytes(bytes.get(sum_at..)?.try_into().ok()?);
    if checksum(&bytes[..sum_at]) != want {
        return None;
    }

    // ── span-file string table (the trailer's target) ──
    let toff_at = sum_at - 8;
    let table_at = u64::from_le_bytes(bytes.get(toff_at..toff_at + 8)?.try_into().ok()?) as usize;
    if table_at < at || table_at > toff_at {
        return None;
    }
    let mut td = Dec {
        b: &bytes[..toff_at],
        at: table_at,
        files: Vec::new(),
        strings: Vec::new(),
        scopes: Vec::new(),
    };
    let n = td.count()?;
    let mut files = Vec::with_capacity(n);
    for _ in 0..n {
        files.push(Arc::new(td.raw_str()?));
    }
    let n = td.count()?;
    let mut strings = Vec::with_capacity(n);
    for _ in 0..n {
        strings.push(td.raw_str()?);
    }

    let mut d = Dec { b: &bytes[..table_at], at, files, strings, scopes: Vec::new() };

    // ── macros ──
    let n = d.count()?;
    let mut macros = HashMap::with_capacity(n);
    for _ in 0..n {
        let k = d.str()?;
        macros.insert(k, d.macro_def()?);
    }
    let n = d.count()?;
    let mut surface_forms = HashMap::with_capacity(n);
    for _ in 0..n {
        let k = d.str()?;
        surface_forms.insert(k, d.ast()?);
    }

    // ── types ──
    let n = d.count()?;
    let mut types = HashMap::with_capacity(n);
    for _ in 0..n {
        let k = d.str()?;
        types.insert(k, d.typedef()?);
    }
    let n = d.count()?;
    let mut builtin_names = HashSet::with_capacity(n);
    for _ in 0..n {
        builtin_names.insert(d.str()?);
    }
    let n = d.count()?;
    let mut subtype_edges = HashMap::with_capacity(n);
    for _ in 0..n {
        let k = d.str()?;
        subtype_edges.insert(k, d.strs()?);
    }
    let n = d.count()?;
    let mut source_forms = HashMap::with_capacity(n);
    for _ in 0..n {
        let k = d.str()?;
        source_forms.insert(k, d.ast()?);
    }

    // ── symbols ──
    let mut symbols = SymbolTable::new();
    let n = d.count()?;
    for _ in 0..n {
        let k = d.str()?;
        symbols.register_function(k, Arc::new(d.function()?));
    }
    let n = d.count()?;
    for _ in 0..n {
        let k = d.str()?;
        let m = d.count()?;
        let mut inner = HashMap::with_capacity(m);
        for _ in 0..m {
            let ik = d.str()?;
            inner.insert(ik, d.ast()?);
        }
        symbols.binding_metadata.insert(k, inner);
    }
    let n = d.count()?;
    for _ in 0..n {
        let k = d.str()?;
        symbols.acronym_registry.insert(k, d.strs()?);
    }

    // ── residue + witness ──
    let runtime_def_forms = d.asts()?;
    let probe_witness = d.strs()?;
    let infer_raw = d.uvar()?;
    let infer_fresh_consumed = if infer_raw == u64::MAX {
        None
    } else {
        Some(infer_raw)
    };

    // The body must end exactly where the string table begins. A payload that decodes but
    // leaves bytes behind is a payload this decoder did not understand.
    if d.at != table_at {
        return None;
    }

    Some(StdlibSnapshot {
        macros: MacroRegistry::from_cache_parts(macros, surface_forms),
        types: TypeEnv::from_cache_parts(types, builtin_names, subtype_edges, source_forms),
        symbols,
        runtime_def_forms,
        probe_witness,
        infer_fresh_consumed,
    })
}

// ── representability ────────────────────────────────────────────────────────────────────────

/// ⛔ ENCODE-OR-REFUSE. The snapshot is written ONLY when every part of it is faithfully
/// representable in the format above. Anything else — a closed environment, a native body, a
/// prebuilt unit variant, a `def`-bound `Value` — means the cache would be a LOSSY copy of the
/// world, and a lossy cache is worse than no cache. In that case nothing is written and every
/// boot derives, which is exactly what happens today.
pub(crate) fn is_representable(snap: &StdlibSnapshot) -> bool {
    if snap.symbols.unit_variants_iter().next().is_some() {
        return false;
    }
    if snap.symbols.def_values_iter().next().is_some() {
        return false;
    }
    if snap.symbols.encoding_ctx().is_some()
        || snap.symbols.source_loader().is_some()
        || snap.symbols.macro_registry().is_some()
        || snap.symbols.types().is_some()
        || snap.symbols.primed_stdio().is_some()
        || snap.symbols.presence_sigma_fn().is_some()
        || snap.symbols.coincident_sigma_fn().is_some()
        || snap.symbols.outer_symbols.is_some()
    {
        return false;
    }
    snap.symbols
        .function_values()
        .all(|f| f.closed_env.is_none() && matches!(f.body, FunctionBody::Wat(_)))
}

// ── the expander's probe witness ────────────────────────────────────────────────────────────

thread_local! {
    /// Active ONLY while the snapshot's stdlib expansion is being produced. See
    /// [`MacroRegistry::contains`], the single door every macro-head probe goes through.
    static WITNESS: std::cell::RefCell<Option<BTreeSet<String>>> = const {
        std::cell::RefCell::new(None)
    };
}

/// Record `name` if a witness is being collected. Called from `MacroRegistry::contains`.
pub(crate) fn note_probe(name: &str) {
    WITNESS.with(|w| {
        if let Some(set) = w.borrow_mut().as_mut() {
            if !crate::resolve::is_reserved_prefix(name) {
                set.insert(name.to_string());
            }
        }
    });
}

/// Run `f` with probe-witness collection on, returning its result and the witness.
pub(crate) fn with_witness<T>(f: impl FnOnce() -> T) -> (T, Vec<String>) {
    WITNESS.with(|w| *w.borrow_mut() = Some(BTreeSet::new()));
    let out = f();
    let set = WITNESS.with(|w| w.borrow_mut().take()).unwrap_or_default();
    (out, set.into_iter().collect())
}

// ── the boot-side decision ──────────────────────────────────────────────────────────────────

/// Should this boot WRITE a snapshot? Only when the cache is enabled, this build has a
/// fingerprint, and no payload is already on disk. Checked before the snapshot's three clones
/// are taken, so a warm boot pays nothing for the write path's existence.
pub(crate) fn should_store() -> bool {
    if !enabled() {
        return false;
    }
    match cache_path() {
        Some(p) => !p.exists(),
        None => false,
    }
}

/// Read the snapshot and decide whether it applies to THIS program.
///
/// The gate's inputs are derived from the user's raw forms by an OVER-APPROXIMATING walk: every
/// `defmacro` name anywhere in the tree (including inside quoted templates, which cannot actually
/// register at step 4) and whether any acronym declaration appears at all. Over-approximating is
/// the safe direction — it can only refuse a cache that would have been fine, never accept one
/// that would not.
pub(crate) fn consider(user_forms: &[WatAST]) -> Option<StdlibSnapshot> {
    if !enabled() {
        return None;
    }
    let (user_macros, user_acronyms) = user_inputs(user_forms);
    if user_acronyms {
        // Refused before the read: the snapshot's expansion consulted an EMPTY acronym registry,
        // and a program that declares acronyms may make it consult a non-empty one.
        return None;
    }
    let snap = load()?;
    if gate_holds(&snap, &user_macros, user_acronyms) {
        Some(snap)
    } else {
        None
    }
}

/// The head keyword of the two forms the gate cares about.
const DEFMACRO_HEAD: &str = ":wat::core::defmacro";
const DECLARE_ACRONYMS_HEAD: &str = ":wat::string::declare-acronyms";

/// `(names a user `defmacro` may register, does the program declare any acronym)`.
fn user_inputs(forms: &[WatAST]) -> (HashSet<String>, bool) {
    let mut names = HashSet::new();
    let mut acronyms = false;
    for f in forms {
        walk_user_inputs(f, &mut names, &mut acronyms);
    }
    (names, acronyms)
}

fn walk_user_inputs(node: &WatAST, names: &mut HashSet<String>, acronyms: &mut bool) {
    match node {
        WatAST::List(items, _) => {
            if let Some(WatAST::Keyword(head, _)) = items.first() {
                if head == DEFMACRO_HEAD {
                    match items.get(1) {
                        Some(WatAST::Keyword(n, _)) => {
                            names.insert(n.clone());
                        }
                        Some(WatAST::Symbol(id, _)) => {
                            names.insert(id.as_str().to_string());
                        }
                        _ => {}
                    }
                } else if head == DECLARE_ACRONYMS_HEAD {
                    *acronyms = true;
                }
            }
            for i in items {
                walk_user_inputs(i, names, acronyms);
            }
        }
        WatAST::Vector(items, _) | WatAST::Set(items, _) => {
            for i in items {
                walk_user_inputs(i, names, acronyms);
            }
        }
        WatAST::Map(pairs, _) => {
            for (k, v) in pairs {
                walk_user_inputs(k, names, acronyms);
                walk_user_inputs(v, names, acronyms);
            }
        }
        _ => {}
    }
}

// ── the tests' doors ────────────────────────────────────────────────────────────────────────
//
// `pub` (not `pub(crate)`) and behind `#[doc(hidden)]`: a byte-exact fixpoint over the REAL
// payload can only be asserted from a test that boots a real world, and integration tests are a
// separate crate. Everything here is measurement and proof surface — nothing boot calls.

/// Derive a snapshot the way a cold boot does, for tests and measurement.
#[doc(hidden)]
pub mod probe {
    use super::{StdlibSnapshot, WatAST};

    /// Encode a snapshot with a fixed key — the fixpoint test's `encode`.
    pub fn encode_with_key(snap: &StdlibSnapshot, key: &str) -> Vec<u8> {
        super::encode(snap, key)
    }

    /// Decode a payload with a fixed key — the fixpoint test's `decode`. `None` on ANY problem.
    pub fn decode_with_key(bytes: &[u8], key: &str) -> Option<StdlibSnapshot> {
        super::decode(bytes, key)
    }

    /// Build a synthetic snapshot carrying `forms` as the cached runtime-def residue. The door
    /// used to round-trip the `WatAST` variants the stdlib payload never exercises.
    pub fn snapshot_of_forms(forms: Vec<WatAST>) -> StdlibSnapshot {
        StdlibSnapshot {
            macros: crate::macros::MacroRegistry::new(),
            types: crate::types::TypeEnv::new(),
            symbols: crate::runtime::SymbolTable::new(),
            runtime_def_forms: forms,
            probe_witness: Vec::new(),
            infer_fresh_consumed: None,
        }
    }

    /// The residue forms a decoded snapshot carries — the variant test reads its payload back out.
    pub fn forms_of(snap: &StdlibSnapshot) -> &[WatAST] {
        &snap.runtime_def_forms
    }

    /// The gate's witness, for the report.
    pub fn witness_of(snap: &StdlibSnapshot) -> &[String] {
        &snap.probe_witness
    }

    /// Counts for the report: (macros, surface forms, types, builtin names, functions, residue).
    pub fn shape_of(snap: &StdlibSnapshot) -> (usize, usize, usize, usize, usize, usize) {
        let (m, sf) = snap.macros.cache_parts();
        let (t, bn, _, _) = snap.types.cache_parts();
        (
            m.len(),
            sf.len(),
            t.len(),
            bn.len(),
            snap.symbols.functions_iter().count(),
            snap.runtime_def_forms.len(),
        )
    }

    /// Where this build's payload lives (`None` when the build carries no fingerprint).
    pub fn path() -> Option<std::path::PathBuf> {
        super::cache_path()
    }

    /// This build's cache key.
    pub fn key() -> Option<String> {
        super::cache_key()
    }

    /// Read the payload for this build, if one is on disk and applies.
    pub fn load() -> Option<StdlibSnapshot> {
        super::load()
    }

    /// Is every part of `snap` faithfully representable in the format? (`encode-or-refuse`.)
    pub fn is_representable(snap: &StdlibSnapshot) -> bool {
        super::is_representable(snap)
    }
}
