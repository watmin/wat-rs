//! FEASIBILITY PROBE — excursus 001 `the-boot-cache-is-possible-or-it-is-not`, question 3.
//!
//! ⛔ THIS IS NOT THE CACHE, AND IT IS NOT WIRED TO BOOT. It is a *crude* hand-rolled
//! serializer for the single largest serialisable component of the frozen world — every
//! `FunctionBody::Wat` body in the symbol table after a real stdlib freeze — existing only to
//! put a NUMBER on "how fast does a round-trip of this size go". `startup_from_source` does not
//! know this file exists; nothing in `src/` calls it.
//!
//! WHY CRUDE IS THE RIGHT INSTRUMENT (DESIGN § 3): a crude encoder is a PESSIMISTIC estimate.
//! If crude is already fast enough, GO is proven. If crude is slow, that proves NOTHING — a
//! zero-copy / mmap format could be an order faster. The format below is deliberately
//! unsophisticated: LEB128 varints, length-prefixed UTF-8, one recursive walk, one `Vec<u8>`.
//! The only concession above "stupid" is a string table for span FILE labels (56 distinct
//! values across ~1.7 M nodes); the size of the un-interned form is measured too, so the
//! concession is visible rather than assumed.
//!
//! WHAT THE ROUND-TRIP PROVES. Hygiene `ScopeId`s are process-global opaque tokens with no
//! `from_u64` constructor — and `Function::params`' own doc says why that is correct: "an
//! exec'd child restarts `fresh_scope()` at 1, so imported scopes must be REMAPPED". So this
//! encoder writes scopes as DENSE FIRST-ENCOUNTER INDICES and the decoder re-mints them through
//! `fresh_scope()`. That makes `encode(decode(encode(x))) == encode(x)` a byte-exact fixpoint
//! test, which is a STRONGER check than `assert_eq!` on the trees would be: `WatAST`'s derived
//! `PartialEq` is span-transparent by design (see `crates/wat-reader/src/ast.rs`), so a
//! tree-equality assertion would pass on a decoder that dropped every span. The bytes carry the
//! spans; byte equality does not let them go missing.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use wat::ast::WatAST;
use wat::scope::{fresh_scope, Identifier, ScopeId};
use wat::span::{Pos, Span};
use wat::value::FunctionBody;

// ── the crude format ────────────────────────────────────────────────────────────────────────

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
    /// span file label → index. 56-ish entries for a whole stdlib boot.
    files: HashMap<String, u32>,
    file_order: Vec<String>,
    /// hygiene ScopeId → dense first-encounter index.
    scopes: HashMap<u64, u32>,
    /// bytes the file labels WOULD have cost written verbatim at every node.
    verbatim_file_bytes: usize,
}

impl Enc {
    fn new() -> Self {
        Enc {
            out: Vec::with_capacity(1 << 22),
            files: HashMap::new(),
            file_order: Vec::new(),
            scopes: HashMap::new(),
            verbatim_file_bytes: 0,
        }
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
        self.uvar(s.len() as u64);
        self.out.extend_from_slice(s.as_bytes());
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
        self.verbatim_file_bytes += s.file.len() + 1;
        self.uvar(idx as u64);
        self.ivar(s.line);
        self.ivar(s.col);
        match &s.end {
            None => self.out.push(0),
            Some(p) => {
                self.out.push(1);
                self.ivar(p.line);
                self.ivar(p.col);
            }
        }
    }

    fn ident(&mut self, id: &Identifier) {
        self.str(id.as_str());
        let sc = id.scopes();
        self.uvar(sc.len() as u64);
        for s in sc {
            let raw = s.as_u64();
            let next = self.scopes.len() as u32;
            let dense = *self.scopes.entry(raw).or_insert(next);
            self.uvar(dense as u64);
        }
    }

    fn node(&mut self, n: &WatAST) {
        match n {
            WatAST::IntLit(v, s) => {
                self.out.push(T_INT);
                self.ivar(*v);
                self.span(s);
            }
            WatAST::FloatLit(v, s) => {
                self.out.push(T_FLOAT);
                self.out.extend_from_slice(&v.to_bits().to_le_bytes());
                self.span(s);
            }
            WatAST::RationalLit(v, s) => {
                self.out.push(T_RATIONAL);
                let t = v.to_string();
                self.str(&t);
                self.span(s);
            }
            WatAST::BigIntLit(v, s) => {
                self.out.push(T_BIGINT);
                let t = v.to_string();
                self.str(&t);
                self.span(s);
            }
            WatAST::CharLit(c, s) => {
                self.out.push(T_CHAR);
                self.uvar(*c as u64);
                self.span(s);
            }
            WatAST::BoolLit(b, s) => {
                self.out.push(T_BOOL);
                self.out.push(u8::from(*b));
                self.span(s);
            }
            WatAST::StringLit(v, s) => {
                self.out.push(T_STRING);
                let v = v.clone();
                self.str(&v);
                self.span(s);
            }
            WatAST::NilLit(s) => {
                self.out.push(T_NIL);
                self.span(s);
            }
            WatAST::Keyword(k, s) => {
                self.out.push(T_KEYWORD);
                let k = k.clone();
                self.str(&k);
                self.span(s);
            }
            WatAST::Symbol(id, s) => {
                self.out.push(T_SYMBOL);
                self.ident(id);
                self.span(s);
            }
            WatAST::List(items, s) => self.seq(T_LIST, items, s),
            WatAST::Vector(items, s) => self.seq(T_VECTOR, items, s),
            WatAST::Set(items, s) => self.seq(T_SET, items, s),
            WatAST::Map(pairs, s) => {
                self.out.push(T_MAP);
                self.uvar(pairs.len() as u64);
                for (k, v) in pairs {
                    self.node(k);
                    self.node(v);
                }
                self.span(s);
            }
        }
    }

    fn seq(&mut self, tag: u8, items: &[WatAST], s: &Span) {
        self.out.push(tag);
        self.uvar(items.len() as u64);
        for i in items {
            self.node(i);
        }
        self.span(s);
    }
}

/// Encode `(path, body)` pairs. Returns the payload plus the encoder's own bookkeeping.
fn encode(entries: &[(String, Arc<WatAST>)]) -> Enc {
    let mut e = Enc::new();
    e.uvar(entries.len() as u64);
    for (path, body) in entries {
        let p = path.clone();
        e.str(&p);
        e.node(body);
    }
    // the string table goes last: the decoder reads it from a trailer offset.
    let table_at = e.out.len() as u64;
    let n = e.file_order.len();
    e.uvar(n as u64);
    for i in 0..n {
        let f = e.file_order[i].clone();
        e.str(&f);
    }
    e.out.extend_from_slice(&table_at.to_le_bytes());
    e
}

struct Dec<'a> {
    b: &'a [u8],
    at: usize,
    files: Vec<Arc<String>>,
    scopes: Vec<ScopeId>,
}

impl<'a> Dec<'a> {
    fn uvar(&mut self) -> u64 {
        let mut v = 0u64;
        let mut shift = 0u32;
        loop {
            let byte = self.b[self.at];
            self.at += 1;
            v |= u64::from(byte & 0x7f) << shift;
            if byte & 0x80 == 0 {
                return v;
            }
            shift += 7;
        }
    }

    fn ivar(&mut self) -> i64 {
        let u = self.uvar();
        ((u >> 1) as i64) ^ -((u & 1) as i64)
    }

    fn str(&mut self) -> String {
        let n = self.uvar() as usize;
        let s = std::str::from_utf8(&self.b[self.at..self.at + n])
            .expect("crude payload holds UTF-8")
            .to_string();
        self.at += n;
        s
    }

    fn span(&mut self) -> Span {
        let idx = self.uvar() as usize;
        let file = Arc::clone(&self.files[idx]);
        let line = self.ivar();
        let col = self.ivar();
        let tag = self.b[self.at];
        self.at += 1;
        let end = if tag == 0 {
            None
        } else {
            let l = self.ivar();
            let c = self.ivar();
            Some(Pos { line: l, col: c })
        };
        Span { file, line, col, end }
    }

    fn ident(&mut self) -> Identifier {
        let name = self.str();
        let n = self.uvar() as usize;
        let mut id = Identifier::bare(name);
        for _ in 0..n {
            let dense = self.uvar() as usize;
            while self.scopes.len() <= dense {
                self.scopes.push(fresh_scope());
            }
            id = id.add_scope(self.scopes[dense]);
        }
        id
    }

    fn node(&mut self) -> WatAST {
        let tag = self.b[self.at];
        self.at += 1;
        match tag {
            T_INT => {
                let v = self.ivar();
                WatAST::IntLit(v, self.span())
            }
            T_FLOAT => {
                let mut buf = [0u8; 8];
                buf.copy_from_slice(&self.b[self.at..self.at + 8]);
                self.at += 8;
                WatAST::FloatLit(f64::from_bits(u64::from_le_bytes(buf)), self.span())
            }
            T_RATIONAL => {
                let t = self.str();
                let v: num_rational::BigRational = t.parse().expect("rational text round-trips");
                WatAST::RationalLit(v, self.span())
            }
            T_BIGINT => {
                let t = self.str();
                let v: num_bigint::BigInt = t.parse().expect("bigint text round-trips");
                WatAST::BigIntLit(v, self.span())
            }
            T_CHAR => {
                let c = char::from_u32(self.uvar() as u32).expect("char round-trips");
                WatAST::CharLit(c, self.span())
            }
            T_BOOL => {
                let b = self.b[self.at] != 0;
                self.at += 1;
                WatAST::BoolLit(b, self.span())
            }
            T_STRING => {
                let s = self.str();
                WatAST::StringLit(s, self.span())
            }
            T_NIL => WatAST::NilLit(self.span()),
            T_KEYWORD => {
                let k = self.str();
                WatAST::Keyword(k, self.span())
            }
            T_SYMBOL => {
                let id = self.ident();
                WatAST::Symbol(id, self.span())
            }
            T_LIST => {
                let items = self.seq();
                WatAST::List(items, self.span())
            }
            T_VECTOR => {
                let items = self.seq();
                WatAST::Vector(items, self.span())
            }
            T_SET => {
                let items = self.seq();
                WatAST::Set(items, self.span())
            }
            T_MAP => {
                let n = self.uvar() as usize;
                let mut pairs = Vec::with_capacity(n);
                for _ in 0..n {
                    let k = self.node();
                    let v = self.node();
                    pairs.push((k, v));
                }
                WatAST::Map(pairs, self.span())
            }
            other => panic!("crude payload carries an unknown tag {other}"),
        }
    }

    fn seq(&mut self) -> Vec<WatAST> {
        let n = self.uvar() as usize;
        let mut items = Vec::with_capacity(n);
        for _ in 0..n {
            items.push(self.node());
        }
        items
    }
}

fn decode(bytes: &[u8]) -> Vec<(String, Arc<WatAST>)> {
    let trailer = bytes.len() - 8;
    let mut off = [0u8; 8];
    off.copy_from_slice(&bytes[trailer..]);
    let table_at = u64::from_le_bytes(off) as usize;

    // Read the file string table first (it is the trailer's target).
    let mut td = Dec { b: bytes, at: table_at, files: Vec::new(), scopes: Vec::new() };
    let n = td.uvar() as usize;
    let mut files = Vec::with_capacity(n);
    for _ in 0..n {
        files.push(Arc::new(td.str()));
    }

    let mut d = Dec { b: bytes, at: 0, files, scopes: Vec::new() };
    let count = d.uvar() as usize;
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        let path = d.str();
        let body = Arc::new(d.node());
        out.push((path, body));
    }
    out
}

// ── the probe ───────────────────────────────────────────────────────────────────────────────

fn count_nodes(n: &WatAST) -> usize {
    match n {
        WatAST::List(items, _) | WatAST::Vector(items, _) | WatAST::Set(items, _) => {
            1 + items.iter().map(count_nodes).sum::<usize>()
        }
        WatAST::Map(pairs, _) => {
            1 + pairs.iter().map(|(k, v)| count_nodes(k) + count_nodes(v)).sum::<usize>()
        }
        _ => 1,
    }
}

fn median(mut v: Vec<Duration>) -> Duration {
    v.sort();
    v[v.len() / 2]
}

/// Evict `path` from the page cache without root. `POSIX_FADV_DONTNEED` on a clean file drops
/// its pages; `fsync` first so nothing is dirty and therefore unevictable.
fn evict(path: &std::path::Path) {
    use std::os::unix::io::AsRawFd;
    let f = std::fs::File::open(path).expect("cache file opens");
    f.sync_all().ok();
    // SAFETY: fd is live for the call; DONTNEED is advisory and cannot corrupt the file.
    unsafe {
        libc::posix_fadvise(f.as_raw_fd(), 0, 0, libc::POSIX_FADV_DONTNEED);
    }
}

/// ⭑ The round-trip, at representative size, timed both directions, warm and cold.
///
/// Correctness is asserted; SPEED IS NOT. A timing threshold in the floor would be a flake
/// factory on a shared box — the numbers are printed and read by the excursus, and the
/// invariant this test gates is the one that can actually be violated by a code change: that
/// the frozen world's function bodies are pure data that survives a byte round-trip.
#[test]
fn the_frozen_worlds_function_bodies_round_trip_through_a_crude_byte_format() {
    let t_boot = Instant::now();
    let world = wat::freeze::startup_bare().expect("the bare world must freeze");
    let boot = t_boot.elapsed();

    let mut entries: Vec<(String, Arc<WatAST>)> = Vec::new();
    let mut native = 0usize;
    for (path, func) in world.symbols().functions_iter() {
        match &func.body {
            FunctionBody::Wat(body) => entries.push((path.clone(), Arc::clone(body))),
            FunctionBody::Native => native += 1,
        }
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let nodes: usize = entries.iter().map(|(_, b)| count_nodes(b)).sum();
    assert!(
        entries.len() > 1000,
        "a payload of {} bodies is not representative of a stdlib boot",
        entries.len()
    );

    // ── encode ──────────────────────────────────────────────────────────────────────────
    let mut enc_times = Vec::new();
    let mut payload = Vec::new();
    let mut verbatim = 0usize;
    let mut files = 0usize;
    let mut scopes = 0usize;
    for i in 0..7 {
        let t = Instant::now();
        let e = encode(&entries);
        let dt = t.elapsed();
        if i == 0 {
            payload = e.out.clone();
            verbatim = e.verbatim_file_bytes;
            files = e.file_order.len();
            scopes = e.scopes.len();
        }
        enc_times.push(dt);
    }
    let enc_cold = enc_times[0];
    let enc_warm = median(enc_times[1..].to_vec());

    // ── decode, in memory ───────────────────────────────────────────────────────────────
    let mut dec_times = Vec::new();
    let mut decoded = Vec::new();
    for i in 0..7 {
        let t = Instant::now();
        let d = decode(&payload);
        let dt = t.elapsed();
        if i == 0 {
            decoded = d;
        }
        dec_times.push(dt);
    }
    let dec_cold = dec_times[0];
    let dec_warm = median(dec_times[1..].to_vec());

    // ── the fixpoint: re-encoding the decode must reproduce the bytes exactly ───────────
    let again = encode(&decoded);
    assert_eq!(
        again.out.len(),
        payload.len(),
        "re-encoding the decoded payload changed its length — the decode lost or invented data"
    );
    assert!(
        again.out == payload,
        "re-encoding the decoded payload did not reproduce the bytes; spans, scopes or literals \
         did not survive the round trip"
    );
    assert_eq!(
        decoded.len(),
        entries.len(),
        "the decode returned a different number of function bodies"
    );
    for ((p1, b1), (p2, b2)) in entries.iter().zip(decoded.iter()) {
        assert_eq!(p1, p2, "function paths must round-trip in order");
        assert_eq!(b1.as_ref(), b2.as_ref(), "function body {p1} did not round-trip");
    }

    // ── through a real file, cold (page cache evicted) and warm ─────────────────────────
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("boot-cache-probe.bin");
    std::fs::write(&path, &payload).expect("write payload");

    evict(&path);
    let t = Instant::now();
    let cold_bytes = std::fs::read(&path).expect("cold read");
    let cold_read = t.elapsed();
    let t = Instant::now();
    let cold_decoded = decode(&cold_bytes);
    let cold_full = cold_read + t.elapsed();
    assert_eq!(cold_decoded.len(), entries.len(), "the cold-path decode must agree");

    let mut warm_full = Vec::new();
    for _ in 0..5 {
        let t = Instant::now();
        let b = std::fs::read(&path).expect("warm read");
        let d = decode(&b);
        warm_full.push(t.elapsed());
        assert_eq!(d.len(), entries.len());
    }
    let warm_full = median(warm_full);

    // ── what else is in the frozen world (question 2: data vs handles) ──────────────────
    let sym = world.symbols();
    let mut def_kinds: std::collections::BTreeMap<&'static str, usize> =
        std::collections::BTreeMap::new();
    for (_, v) in sym.def_values_iter() {
        *def_kinds.entry(v.type_name()).or_insert(0) += 1;
    }
    let unit_variants = sym.unit_variants_iter().count();
    let types = world.types.iter().count();
    let residue = world.program.len();
    let mut closed_env = 0usize;
    let mut scoped_params = 0usize;
    let mut params_total = 0usize;
    for f in sym.function_values() {
        if f.closed_env.is_some() {
            closed_env += 1;
        }
        for p in &f.params {
            params_total += 1;
            if !p.scopes().is_empty() {
                scoped_params += 1;
            }
        }
    }

    // How much of the world's reachable AST does the payload actually cover? "Largest
    // serialisable component" is a claim; these are the numbers behind it.
    let mut type_form_nodes = 0usize;
    let mut type_forms = 0usize;
    for (name, _) in world.types.iter() {
        if let Some(f) = world.types.source_form(name) {
            type_forms += 1;
            type_form_nodes += count_nodes(f);
        }
    }
    let mut meta_nodes = 0usize;
    for m in sym.binding_metadata.values() {
        for v in m.values() {
            meta_nodes += count_nodes(v);
        }
    }
    let residue_nodes: usize = world.program.iter().map(count_nodes).sum();
    // The baked manifest (`src/load/stdlib.rs`'s `stdlib_forms`) is `pub(crate)`, so the
    // pre-expansion size is taken by re-parsing `wat/**/*.wat` from disk instead. That is a
    // NEAR-match for the 55-entry manifest, not the manifest itself — stated rather than
    // implied, since it is only used as an order-of-magnitude denominator.
    let mut stdlib_parsed_nodes = 0usize;
    let mut stdlib_parsed_files = 0usize;
    let mut stack = vec![std::path::PathBuf::from("wat")];
    while let Some(dir) = stack.pop() {
        for e in std::fs::read_dir(&dir).expect("wat/ is readable from the crate root") {
            let p = e.expect("dir entry").path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| x == "wat") {
                let src = std::fs::read_to_string(&p).expect("stdlib file reads");
                if let Ok(forms) = wat::parse_all_with_file(&src, &p.display().to_string()) {
                    stdlib_parsed_files += 1;
                    stdlib_parsed_nodes += forms.iter().map(count_nodes).sum::<usize>();
                }
            }
        }
    }

    // Node-kind census of the payload — says WHY the hygiene-scope count is what it is.
    let mut kinds: std::collections::BTreeMap<&'static str, usize> =
        std::collections::BTreeMap::new();
    fn tally(n: &WatAST, k: &mut std::collections::BTreeMap<&'static str, usize>) {
        let name = match n {
            WatAST::IntLit(..) => "IntLit",
            WatAST::FloatLit(..) => "FloatLit",
            WatAST::RationalLit(..) => "RationalLit",
            WatAST::BigIntLit(..) => "BigIntLit",
            WatAST::CharLit(..) => "CharLit",
            WatAST::BoolLit(..) => "BoolLit",
            WatAST::StringLit(..) => "StringLit",
            WatAST::NilLit(..) => "NilLit",
            WatAST::Keyword(..) => "Keyword",
            WatAST::Symbol(..) => "Symbol",
            WatAST::List(..) => "List",
            WatAST::Vector(..) => "Vector",
            WatAST::Map(..) => "Map",
            WatAST::Set(..) => "Set",
        };
        *k.entry(name).or_insert(0) += 1;
        match n {
            WatAST::List(items, _) | WatAST::Vector(items, _) | WatAST::Set(items, _) => {
                for i in items {
                    tally(i, k);
                }
            }
            WatAST::Map(pairs, _) => {
                for (a, b) in pairs {
                    tally(a, k);
                    tally(b, k);
                }
            }
            _ => {}
        }
    }
    for (_, b) in &entries {
        tally(b, &mut kinds);
    }
    let mut scoped_symbols = 0usize;
    fn scoped(n: &WatAST, c: &mut usize) {
        match n {
            WatAST::Symbol(id, _) => {
                if !id.scopes().is_empty() {
                    *c += 1;
                }
            }
            WatAST::List(items, _) | WatAST::Vector(items, _) | WatAST::Set(items, _) => {
                for i in items {
                    scoped(i, c);
                }
            }
            WatAST::Map(pairs, _) => {
                for (a, b) in pairs {
                    scoped(a, c);
                    scoped(b, c);
                }
            }
            _ => {}
        }
    }
    for (_, b) in &entries {
        scoped(b, &mut scoped_symbols);
    }

    println!("=== FROZEN-WORLD STATE CENSUS (question 2) ===");
    println!("functions (total)            {:>10}", entries.len() + native);
    println!("  FunctionBody::Wat          {:>10}", entries.len());
    println!("  FunctionBody::Native       {:>10}", native);
    println!("  with closed_env (a handle) {:>10}", closed_env);
    println!("MacroRegistry attached       {:>10}", sym.macro_registry().is_some());
    println!("TypeEnv entries              {:>10}", types);
    println!("unit_variants                {:>10}", unit_variants);
    println!("residue program forms        {:>10}", residue);
    println!("primed_stdio present         {:>10}", sym.primed_stdio().is_some());
    println!("runtime def_values by Value kind:");
    for (k, n) in &def_kinds {
        println!("   {k:<40} {n:>6}");
    }
    println!("fn params (total)            {:>10}", params_total);
    println!("  carrying hygiene scopes    {:>10}", scoped_params);
    println!("--- reachable AST nodes, by component ---");
    println!("function bodies              {:>10}", nodes);
    println!("TypeEnv source forms ({type_forms:>4})  {:>10}", type_form_nodes);
    println!("binding_metadata values      {:>10}", meta_nodes);
    println!("residue program              {:>10}", residue_nodes);
    println!("(wat/**/*.wat as PARSED, {stdlib_parsed_files:>3} files){:>7}", stdlib_parsed_nodes);
    println!("payload node-kind census:");
    for (k, n) in &kinds {
        println!("   {k:<14} {n:>8}");
    }
    println!("  Symbols carrying scopes    {:>10}", scoped_symbols);

    println!("=== BOOT CACHE FEASIBILITY PROBE — crude round-trip ===");
    println!("boot (startup_bare)          {:>10.2?}", boot);
    println!("bodies (FunctionBody::Wat)   {:>10}", entries.len());
    println!("natives (FunctionBody::Native){:>9}", native);
    println!("AST nodes                    {:>10}", nodes);
    println!("distinct span files          {:>10}", files);
    println!("distinct hygiene scopes      {:>10}", scopes);
    println!("payload bytes                {:>10}", payload.len());
    println!("  span-file bytes saved by the 1 string table {:>10}", verbatim);
    println!("encode  cold {:>10.3?}   warm {:>10.3?}", enc_cold, enc_warm);
    println!("decode  cold {:>10.3?}   warm {:>10.3?}", dec_cold, dec_warm);
    println!("file read (page-cache COLD)  {:>10.3?}", cold_read);
    println!("read+decode COLD             {:>10.3?}", cold_full);
    println!("read+decode WARM             {:>10.3?}", warm_full);
}

/// ⭑ The stdlib payload exercises only 9 of `WatAST`'s 14 variants and carries ZERO hygiene
/// scopes (both facts are printed by the probe above). A round-trip claim made on that payload
/// alone would be a claim about nine variants dressed as a claim about the format — so this
/// covers the other five (`RationalLit`, `BigIntLit`, `CharLit`, `Map`, `Set`) plus a scoped
/// `Identifier`, which is the one piece of `WatAST` that is NOT plain data: `ScopeId` is a
/// process-global opaque token and must be RE-MINTED at load, never restored by value.
#[test]
fn the_crude_format_round_trips_the_variants_the_stdlib_payload_never_exercises() {
    use num_bigint::BigInt;
    use num_rational::BigRational;

    let sp = |n: i64| Span {
        file: Arc::new("probe.wat".to_string()),
        line: n,
        col: n + 1,
        end: Some(Pos { line: n, col: n + 9 }),
    };

    let s1 = fresh_scope();
    let s2 = fresh_scope();
    let scoped = Identifier::bare("x").add_scope(s1).add_scope(s2);
    let also_s1 = Identifier::bare("y").add_scope(s1);

    let tree = WatAST::List(
        vec![
            WatAST::Keyword(":wat::core::do".into(), sp(1)),
            WatAST::RationalLit(BigRational::new(BigInt::from(-22), BigInt::from(7)), sp(2)),
            WatAST::BigIntLit(
                "170141183460469231731687303715884105727".parse::<BigInt>().unwrap(),
                sp(3),
            ),
            WatAST::CharLit('\u{1F600}', sp(4)),
            WatAST::CharLit('\n', sp(5)),
            WatAST::FloatLit(-0.0, sp(6)),
            WatAST::FloatLit(f64::MIN_POSITIVE, sp(7)),
            WatAST::IntLit(i64::MIN, sp(8)),
            WatAST::IntLit(i64::MAX, sp(9)),
            WatAST::NilLit(sp(10)),
            WatAST::BoolLit(false, sp(11)),
            WatAST::StringLit("a \"quoted\" ünïcode ☃ string".into(), sp(12)),
            WatAST::Symbol(scoped, sp(13)),
            WatAST::Symbol(also_s1, sp(14)),
            WatAST::Symbol(Identifier::bare("unscoped"), sp(15)),
            WatAST::Map(
                vec![
                    (WatAST::Keyword(":a".into(), sp(16)), WatAST::IntLit(1, sp(17))),
                    (WatAST::Keyword(":b".into(), sp(18)), WatAST::NilLit(sp(19))),
                ],
                sp(20),
            ),
            WatAST::Set(vec![WatAST::IntLit(3, sp(21)), WatAST::IntLit(4, sp(22))], sp(23)),
            WatAST::Vector(vec![], sp(24)),
            WatAST::List(vec![], Span { file: Arc::new("other.wat".into()), line: 1, col: 1, end: None }),
        ],
        sp(25),
    );

    let entries = vec![(":probe::all-variants".to_string(), Arc::new(tree))];
    let first = encode(&entries);
    assert_eq!(first.scopes.len(), 2, "the probe tree carries exactly two distinct scopes");
    let decoded = decode(&first.out);
    let second = encode(&decoded);
    assert!(
        second.out == first.out,
        "the crude format is not a fixpoint over the full variant set — a literal, a span or a \
         hygiene scope did not survive the round trip"
    );

    // The scopes came back as DIFFERENT ScopeIds (re-minted, as they must be) that preserve the
    // SHARING structure: `x` holds both, `y` holds the one they share.
    let WatAST::List(items, _) = decoded[0].1.as_ref() else {
        panic!("the decoded tree must still be a list")
    };
    let (WatAST::Symbol(dx, _), WatAST::Symbol(dy, _)) = (&items[12], &items[13]) else {
        panic!("items 12 and 13 must be the scoped symbols")
    };
    assert_eq!(dx.scopes().len(), 2);
    assert_eq!(dy.scopes().len(), 1);
    let shared = dy.scopes().iter().next().copied().expect("y has one scope");
    assert!(dx.scopes().contains(&shared), "the scope x and y SHARE must still be shared");
    assert!(
        !dx.scopes().contains(&s1) && !dx.scopes().contains(&s2),
        "restored scopes must be freshly minted, not the encoding process's own ids — reusing a \
         raw ScopeId across processes is exactly what `Function::params`' doc forbids"
    );

    // And spans genuinely survived: the derived PartialEq would not have caught it.
    let WatAST::Symbol(_, span13) = &items[12] else { unreachable!() };
    assert_eq!(span13.line, 13);
    assert_eq!(span13.col, 14);
    assert_eq!(span13.end.as_ref().map(|p| p.col), Some(22));
    let WatAST::List(_, other) = &items[18] else { panic!("item 18 is the other-file list") };
    assert_eq!(&*other.file, "other.wat", "the span string table must not collapse two files");
    assert!(other.end.is_none(), "a point-span must stay a point-span");
}
