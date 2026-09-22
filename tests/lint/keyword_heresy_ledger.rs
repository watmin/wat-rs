//! ⚑ THE HERESY LEDGER — arc 255 stone 255.13.
//!
//! The builder's ruling, 2026-09-22: *"the end state is that all call heads are symbols, not
//! keywords, anything doing exact keyword matches should be considered heresy at this point…
//! we need a dual support for the migration, but it feels like that dual support is nearing its
//! terminal state."* Dual acceptance is a MIGRATION SCAFFOLD with an expiry. **This file is the
//! instrument that says when the expiry has arrived: keyword call heads become illegal when this
//! ledger reads 0 and the `.wat` corpus is converted. Not before, and the stone that does it
//! cites this number.**
//!
//! ⛔ THIS COUNTS. IT DOES NOT CURE. Nothing in `src/` was changed by the stone that wrote it.
//!
//! ═══ THE DISCRIMINATOR ═══════════════════════════════════════════════════════════════════════
//!
//! Heresy is **NOT** "a `":wat::…"` string literal" — most of those are message text, `OP`
//! labels and `#[wat_intrinsic]` attributes. And comparing an **already-canonicalized** name to a
//! keyword constant is **CORRECT**: the internal identity *is* keyword-spelled
//! (`canonical_identity` → `:wat::core::X`). The heresy is
//!
//! > a decision made by comparing a NAME THAT CAN ARRIVE IN EITHER SPELLING against a
//! > keyword-spelled literal, WITHOUT passing it through the identity door first.
//!
//! That is a **DATA-FLOW** question, so this lint answers it with a real Rust parser (`syn`) and
//! a def-use analysis — never a line window. ⛔ The instrument this deliberately replaces is the
//! ±3-line proximity probe, which inside this arc alone produced **3 false positives in 14**
//! (255.9 §1.2) and **missed `walk_for_restricted_call`** (255.11 §1), which turned out to be a
//! live capability escape. A window cannot see a `let` chain, a `match` scrutinee, or a callee.
//!
//! **What it does, in four passes:**
//!
//! 1. **Parse** every `.rs` under `src/` with `syn` (333 files, 0 parse failures; a parse failure
//!    is a hard error, never a silent skip). `#[cfg(test)]` modules, `mod tests` and `#[test]`
//!    fns are excluded — this is a `src/` instrument.
//! 2. **Derive the DOOR SET by fixpoint**, seeded at exactly two roots — `canonical_identity` and
//!    `ns_to_wat_path` (`src/edn/render.rs`), the two functions that turn a written spelling into
//!    the internal identity. A fn joins the set when it returns a NAME (`String`/`&str`/`Cow`)
//!    and every one of its returns is door-derived. ⭐ The set is DERIVED, not hand-listed, and
//!    it is printed by the census test so it is falsifiable.
//! 3. **Provenance taint**, block-scoped and order-sensitive (shadowing honoured), over `let`,
//!    `if let`, `while let`, `match` arms, closures, and collection `push`/`insert`/`extend`.
//!    Each value carries one of:
//!    - `Door` / `KwConst` — proven canonical (through a door, or an internal `:wat::` constant)
//!    - `KwAst` — a raw `WatAST::Keyword` payload (**shape A**: correct VALUE, blind dispatch)
//!    - `SymAst` — a raw `WatAST::Symbol` payload (**shape C**)
//!    - `BothAst` — a value that joins both, un-normalized (**shape B**: it SEES the symbol and
//!      then keys on the keyword — strictly worse than A)
//!    - `TyPath` — a `TypeExpr::Path`/`Parametric` payload, compared without the denotation door
//!      (**shape E**: 255.12's class)
//!    - `Unknown` — no evidence either way (**shape D**: NOT counted as heresy, counted apart)
//! 4. **Inter-procedural parameter provenance by fixpoint** over every call site in `src/`, so a
//!    `fn f(head: &str)` that compares `head` to a keyword literal is judged by what its CALLERS
//!    hand it. Converges in 6 rounds (cap 8; the round count is asserted).
//!
//! **Two named rules carry the discrimination, and each is stated with its failure mode:**
//!
//! - **THE DUAL-ARM RULE.** A `match` over a `WatAST` that has BOTH a `Keyword` arm and a
//!   `Symbol` arm covers both spellings: the `Keyword` payload IS the internal identity, and the
//!   `Symbol` payload needs a door. If every `Symbol` arm routes through one, the whole
//!   expression yields a canonical identity. This is what makes `declare::parse::head_fqdn` a
//!   DOOR and `form_match::identity_text` — whose `Symbol` arm returns `id.as_str()` raw — NOT
//!   one, which is exactly what their own doc comments say.
//! - **THE COVERAGE RULE** (`KwAst ∨ Door = Door`): the same fact generalised to a value that
//!   reaches the decision through a candidate LIST rather than a match value — 255.10's cure in
//!   `macros/eval.rs::validate_pure_total` builds `head_candidates` by pushing the raw keyword in
//!   one arm and `canonical_identity(...)` in the other. ⚠ Failure mode, stated: a door output
//!   and an UNRELATED keyword-only read joined into one local would read as covered. `SymAst`
//!   and `BothAst` are untouched by it.
//!
//! ═══ WHAT IT CANNOT SEE ══════════════════════════════════════════════════════════════════════
//!
//! 1. **REACHABILITY.** It cannot say whether a symbol spelling can actually ARRIVE at a site —
//!    that is the pipeline-order argument 255.9/255.10/255.11 made by hand, per site, and no
//!    static instrument here reproduces it. The ledger is a COUNTDOWN, not a bug list: shape A
//!    sites that 255.9 classified UNREACHABLE or LOUD are still counted, because the terminal cut
//!    retires them regardless.
//! 2. **A keyword-only READ that feeds a COVERED decision.** The unit is the DECISION, so
//!    `closure_extract.rs:962`'s `match quote_boundary(k)` — a raw `Keyword` payload handed to a
//!    callee whose OTHER callers normalize — is invisible here. 255.9's hand census of the 82
//!    `Some(WatAST::Keyword` read sites is the instrument for that half.
//! 3. **`D-unresolved`** — a decision on a `&str` whose provenance no call site resolved. It is
//!    counted and printed, never folded into the ledger, and never claimed to be clean.
//! 4. **Dynamic dispatch, trait impls and macro-generated code.** Call sites are matched by
//!    function NAME, so a name collision makes a param MORE convicted (the safe direction), and a
//!    body produced by `#[wat_intrinsic]`/`defservice` expansion is not in the token stream `syn`
//!    reads (`NOTE-the-span-lints-cannot-see-generated-code.md`).
//! 5. **Anything outside `src/`** — not `tests/`, not `crates/`, not the `.wat` corpus.
//!
//! ═══ THE CALIBRATION ═════════════════════════════════════════════════════════════════════════
//!
//! `the_discriminator_separates_the_cured_from_the_open` is the gate with teeth: every site this
//! arc has already CURED must be absent from the ledger, and the one still OPEN must be present.
//! See that test for the table and the evidence.
//!
//! ⛔ **And it corrects the brief.** The brief names `src/collection/transform.rs:304-340` as the
//! site that must be flagged. That range contains **no keyword comparison at all** — it is
//! `sort$native`'s CALL of the purity gate. The comparison 251.8d-ii's fourth draw convicted
//! lives in `src/rete/purity.rs`, and this lint finds it there BY DATA FLOW: `classify_expr`
//! reads a head from `Keyword(k) => k.as_str()` / `Symbol(id) => id.as_str()` with no door
//! (**shape B**), and pass C carries that provenance into `intrinsic_meta` and
//! `effectful_by_prefix`, whose tables are keyword-keyed. That is the `wat.core/<` failure.
//!
//! ⭐ **CURED, 2026-09-22, arc 251 stone 251.8d-ii FIFTH draw — and the ledger is the receipt.**
//! `classify_expr`'s Symbol arm now takes `edn::render::canonical_identity`; the Keyword arm is
//! byte-identical (THE DUAL-ARM RULE: the keyword payload IS the internal identity). The ledger
//! read **232 → 229**, and the shape mix is the louder half of that number: `intrinsic_meta`
//! **3 [Bx3] → 0**, `effectful_by_prefix` **8 [Bx8] → 8 [Ax8]** — same count, better class, which
//! is why this test now freezes the MIX as well as the count. Crate-wide shape **B: 14 → 3**
//! (`rete/kernel/arm.rs` ×2, `rete/purity.rs::walk_rete_defn_callees` ×1).
//!
//! Exemption form for a site that is genuinely not heresy: a per-offense
//! `// rune:lint(keyword-heresy) — <reason>` on the offending line or the one above, mirroring
//! `no_inlined_edn`'s per-offense marker. `ALLOWLIST` below is the frozen, reasoned list.

#![allow(clippy::needless_range_loop)]

// rune:lint(no-inlined-wat) — THE SUBJECT OF THIS LINT IS WAT NAMES WRITTEN AS RUST STRING
// LITERALS. It cannot be moved to a `.wat` fixture: the literals here are not a wat PROGRAM to
// run, they are (a) the keyword-spelled FQDNs the discriminator hunts for in `src/`
// (`":wat::core::…"`, and the bare `"wat::core::…"` that `TypeExpr::Parametric` stores), and
// (b) the six synthetic RUST sources in
// `the_discriminator_convicts_a_synthetic_heretic_and_clears_its_cure`, which must contain a
// real comparison against such a literal or the negative control asserts nothing. Every one of
// them is DATA for a text analysis, never a form this test evaluates — there is no wat runtime
// anywhere in this file. Moving them out would delete the only thing being tested.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{Expr, Item, Lit, Pat, Stmt};

const SEED_DOORS: &[&str] = &["canonical_identity", "ns_to_wat_path"];
const KW: &str = ":wat::";

#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
enum Prov { Bottom, Door, KwConst, Unknown, KwAst, TyPath, SymAst, BothAst }
impl Prov {
    /// Rank on the one axis that matters: HOW MUCH EVIDENCE is there that a name which can
    /// arrive in the OTHER spelling reaches this decision. `Door`/`KwConst` = proof it cannot.
    /// `Unknown` = no evidence either way (an unresolved `&str`). `KwAst`/`SymAst`/`BothAst` =
    /// positive evidence of a raw `WatAST` name. ⭐ Positive evidence outranks absence of
    /// evidence — one caller passing a raw AST name convicts the site even if a sibling caller's
    /// argument could not be resolved (`[[feedback_a_failure_to_find_is_not_a_proof_of_absence]]`).
    fn rank(self) -> u8 { match self { Prov::Bottom => 0, Prov::Door => 1, Prov::KwConst => 2, Prov::Unknown => 3, Prov::KwAst => 4, Prov::TyPath => 5, Prov::SymAst => 6, Prov::BothAst => 7 } }
    fn join(self, o: Prov) -> Prov {
        use Prov::*;
        if self == o { return self; }
        if matches!((self, o), (KwAst, SymAst) | (SymAst, KwAst)) { return BothAst; }
        // ⭐ THE COVERAGE RULE. A raw `WatAST::Keyword` payload IS the internal identity — the
        // keyword spelling is what `canonical_identity` would return for it, unchanged. So
        // `KwAst` never means "a wrong value"; it means "this dispatch admitted ONLY the keyword
        // node". If the SAME value can also arrive from a door, the dispatch demonstrably read
        // the symbol spelling too and normalized it, so the pair is covered. This is the
        // `match`-value dual-arm rule generalised to a value that reaches the decision through a
        // candidate LIST (`macros/eval.rs::validate_pure_total`'s `head_candidates`).
        // ⚠ Its failure mode, stated: a door output and an UNRELATED keyword-only read joined
        // into one local would read as covered. `SymAst`/`BothAst` are untouched by this rule.
        if matches!((self, o), (KwAst, Door) | (Door, KwAst)) { return Door; }
        if self.rank() >= o.rank() { self } else { o }
    }
    fn heretical(self) -> bool { matches!(self, Prov::KwAst | Prov::TyPath | Prov::SymAst | Prov::BothAst) }
    fn shape(self) -> &'static str {
        match self { Prov::KwAst => "A-keyword-only", Prov::SymAst => "C-symbol-raw", Prov::BothAst => "B-dual-raw",
                     Prov::Door => "cured", Prov::KwConst => "internal-const", Prov::Bottom => "empty", Prov::TyPath => "E-typepath-raw", Prov::Unknown => "D-unresolved" }
    }
}

const TRANSPARENT: &[&str] = &["as_str","as_ref","as_deref","to_string","to_owned","into","clone","unwrap","expect","borrow","deref","unwrap_or_default","trim","name","text"];
const CARRIER: &[&str] = &["map","and_then","unwrap_or_else","map_or","or_else","unwrap_or","filter","cloned","copied","flatten","ok","last","first","get"];
const WRAPPERS: &[&str] = &["Some","Ok","Owned","Borrowed"];

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(es) = std::fs::read_dir(dir) else { return };
    // rune:lint(one-variant-separator, not-a-name) — `DirEntry::path()` is a FILESYSTEM path; it names no wat enum and no variant.
    for e in es.flatten() { let p = e.path();
        if p.is_dir() { collect_rs(&p, out); } else if p.extension().and_then(|x| x.to_str()) == Some("rs") { out.push(p); } }
}
fn is_kw_spelled(v: &str) -> bool {
    // `:wat::core::i64` — a keyword-spelled FQDN — and `wat::core::Vector`, the SAME name under
    // `TypeExpr::Parametric`'s documented storage convention (heads are stored WITHOUT the
    // leading colon; `parametric_head_fqdn` is the door that puts it back).
    v.starts_with(KW) || v.starts_with("wat::")
}
fn kw_lit(e: &Expr) -> Option<String> {
    if let Expr::Lit(l) = e { if let Lit::Str(s) = &l.lit { let v = s.value(); if is_kw_spelled(&v) { return Some(v); } } } None
}
fn pat_kw_lits(p: &Pat, out: &mut Vec<String>) {
    match p {
        Pat::Lit(l) => { if let Lit::Str(s) = &l.lit { let v = s.value(); if is_kw_spelled(&v) { out.push(v); } } }
        Pat::Or(o) => for c in &o.cases { pat_kw_lits(c, out) },
        Pat::Paren(x) => pat_kw_lits(&x.pat, out),
        Pat::Reference(x) => pat_kw_lits(&x.pat, out),
        _ => {}
    }
}
/// idents bound by a pattern, tagged with the WatAST variant they were destructured from
fn pat_binds(p: &Pat, from: Option<Prov>, out: &mut Vec<(String, Prov)>) {
    match p {
        Pat::Ident(i) => { out.push((i.ident.to_string(), from.unwrap_or(Prov::Unknown)));
                           if let Some((_, s)) = &i.subpat { pat_binds(s, from, out); } }
        Pat::TupleStruct(t) => {
            let tail: Vec<String> = t.path.segments.iter().map(|s| s.ident.to_string()).collect();
            let n = tail.len();
            let variant = if n >= 2 && (tail[n-2] == "WatAST" || tail[n-2] == "TypeExpr") { Some(tail[n-1].as_str()) } else { None };
            let inner = match variant {
                Some("Keyword") => Some(Prov::KwAst),
                Some("Symbol") => Some(Prov::SymAst),
                // A `TypeExpr::Path` payload is a DECLARED type name, which arrives in whatever
                // dialect (and whichever of `wat.type`/`wat.core`) the source wrote — 255.12's
                // whole class. `denoted_type_path` is its door.
                Some("Path") => Some(Prov::TyPath),
                _ => from };
            for (i, e) in t.elems.iter().enumerate() {
                // only the FIRST field of Keyword/Symbol is the name; the 2nd is the span
                let f = if variant.is_some() && i > 0 { Some(Prov::Unknown) } else { inner };
                pat_binds(e, f, out);
            }
        }
        Pat::Tuple(t) => for e in &t.elems { pat_binds(e, from, out) },
        Pat::Struct(st) => {
            let seg: Vec<String> = st.path.segments.iter().map(|x| x.ident.to_string()).collect();
            let n = seg.len();
            let f2 = if n >= 2 && seg[n-2] == "TypeExpr" && seg[n-1] == "Parametric" { Some(Prov::TyPath) } else { from };
            for f in &st.fields { pat_binds(&f.pat, f2, out) }
        }
        Pat::Reference(r) => pat_binds(&r.pat, from, out),
        Pat::Paren(x) => pat_binds(&x.pat, from, out),
        Pat::Slice(s) => for e in &s.elems { pat_binds(e, from, out) },
        Pat::Or(o) => for c in &o.cases { pat_binds(c, from, out) },
        Pat::Type(t) => pat_binds(&t.pat, from, out),
        _ => {}
    }
}
fn path_tail(e: &Expr) -> Option<String> {
    match e { Expr::Path(p) => p.path.segments.last().map(|s| s.ident.to_string()),
        Expr::Reference(r) => path_tail(&r.expr), Expr::Paren(p) => path_tail(&p.expr), Expr::Group(g) => path_tail(&g.expr), _ => None }
}
fn simple_ident(e: &Expr) -> Option<String> {
    match e { Expr::Path(p) if p.qself.is_none() && p.path.segments.len() == 1 => Some(p.path.segments[0].ident.to_string()),
        Expr::Reference(r) => simple_ident(&r.expr), Expr::Paren(p) => simple_ident(&p.expr), Expr::Group(g) => simple_ident(&g.expr),
        Expr::Unary(u) if matches!(u.op, syn::UnOp::Deref(_)) => simple_ident(&u.expr), _ => None }
}

#[derive(Default, Clone)]
struct World { doors: BTreeSet<String>, stringy: BTreeSet<String>, kw_consts: BTreeSet<String>, kw_lists: BTreeSet<String>,
               param_prov: BTreeMap<(String, usize), Prov>, fn_params: BTreeMap<String, Vec<String>> }

struct Ctx<'a> { w: &'a World, scopes: Vec<BTreeMap<String, Prov>>, cur_fn: String }
impl<'a> Ctx<'a> {
    fn bind(&mut self, n: String, p: Prov) { if let Some(t) = self.scopes.last_mut() { t.insert(n, p); } }
    /// A collection LOCAL accumulates the provenance of everything pushed into it.
    fn update(&mut self, n: &str, p: Prov) {
        for s in self.scopes.iter_mut().rev() { if let Some(q) = s.get_mut(n) { *q = q.join(p); return; } }
    }
    fn lookup(&self, n: &str) -> Option<Prov> { for s in self.scopes.iter().rev() { if let Some(p) = s.get(n) { return Some(*p); } } None }
    fn prov(&self, e: &Expr) -> Prov {
        use Prov::*;
        match e {
            Expr::Call(c) => { let t = path_tail(&c.func).unwrap_or_default();
                if self.w.doors.contains(&t) { return Door; }
                if (t == "new" || t == "default" || t == "with_capacity") && c.args.iter().all(|a| matches!(a, Expr::Lit(_))) { return Bottom; }
                if WRAPPERS.contains(&t.as_str()) && c.args.len() == 1 { return self.prov(&c.args[0]); }
                // A non-door fn that RETURNS A NAME (`String`/`&str`/`Cow<str>`) and is handed a
                // raw AST name hands one back: `resolve_core_name(head)` is `head` itself for
                // everything outside `RETE_OPS`. Join the arguments' provenance rather than
                // giving up — over-convicting is the safe direction for a ledger, and the
                // alternative is the `D-unresolved` bucket swallowing the call graph.
                if self.w.stringy.contains(&t) {
                    let mut acc = Bottom;
                    for a in &c.args { acc = acc.join(self.prov(a)); }
                    return acc;
                }
                Unknown }
            Expr::MethodCall(m) => { let n = m.method.to_string();
                if self.w.doors.contains(&n) { return Door; }
                if TRANSPARENT.contains(&n.as_str()) || CARRIER.contains(&n.as_str()) {
                    let mut p = self.prov(&m.receiver);
                    for a in &m.args { if let Some(t) = path_tail(a) { if self.w.doors.contains(&t) { return Door; } }
                                       if let Expr::Closure(c) = a {
                                           let q = self.prov(&c.body);
                                           if q == Door { return Door; }
                                           if CARRIER.contains(&n.as_str()) && q != Unknown { p = q; }
                                       } }
                    return p; }
                if n == "strip_prefix" || n == "strip_suffix" { return self.prov(&m.receiver); }
                Unknown }
            Expr::Path(_) => { if let Some(id) = simple_ident(e) { if let Some(p) = self.lookup(&id) { return p; } }
                if let Some(t) = path_tail(e) { if self.w.kw_consts.contains(&t) { return KwConst; } } Unknown }
            Expr::Reference(r) => self.prov(&r.expr),
            Expr::Paren(p) => self.prov(&p.expr),
            Expr::Group(g) => self.prov(&g.expr),
            Expr::Try(t) => self.prov(&t.expr),
            Expr::Cast(c) => self.prov(&c.expr),
            Expr::Unary(u) if matches!(u.op, syn::UnOp::Deref(_)) => self.prov(&u.expr),
            Expr::Lit(_) => if kw_lit(e).is_some() { KwConst } else { Unknown },
            // An EMPTY collection carries no evidence yet; `.push`/`.insert`/`.extend` join into it.
            Expr::Macro(mc) if mc.mac.path.is_ident("vec") && mc.mac.tokens.is_empty() => Bottom,
            Expr::Macro(mc) if mc.mac.path.is_ident("format") => {
                // `format!(":wat::…{x}")` MANUFACTURES the internal spelling
                let s = mc.mac.tokens.to_string();
                if s.contains("\":wat::") || s.contains("\"wat::") { Door } else { Unknown } }
            Expr::If(i) => { let t = i.then_branch.stmts.last().and_then(|s| match s { Stmt::Expr(e, None) => Some(self.prov(e)), _ => None }).unwrap_or(Unknown);
                let el = i.else_branch.as_ref().map(|(_, e)| self.prov(e)).unwrap_or(Unknown); t.join(el) }
            Expr::Match(m) => {
                // ⭐ THE DUAL-ARM RULE. A `match` over a WatAST that has BOTH a `Keyword` arm and
                // a `Symbol` arm COVERS both spellings. The Keyword arm's payload IS the internal
                // identity (keyword-spelled by construction), so it needs no door; the Symbol arm's
                // payload is the OTHER spelling and does. If every Symbol arm routes through a door,
                // the whole expression yields a canonical identity — that is what makes `head_fqdn`
                // a door and `identity_text` (whose Symbol arm returns `id.as_str()` raw) NOT one.
                let sp = self.prov(&m.expr);
                let mut acc: Option<Prov> = None;
                let (mut has_kw_arm, mut has_sym_arm) = (false, false);
                let (mut kw_ok, mut sym_ok) = (true, true);
                for a in &m.arms {
                    let mut c2 = Ctx { w: self.w, scopes: self.scopes.clone(), cur_fn: self.cur_fn.clone() };
                    c2.scopes.push(BTreeMap::new());
                    let mut bs = Vec::new(); pat_binds(&a.pat, if sp == Unknown { None } else { Some(sp) }, &mut bs);
                    for (n, p) in bs { c2.bind(n, p); }
                    if is_never(&a.body) { continue; }
                    let p = c2.prov(&a.body);
                    match pat_watast_variant(&a.pat) {
                        Some(V::Kw) => { has_kw_arm = true; if !matches!(p, Door | KwConst | KwAst | Bottom) { kw_ok = false; } }
                        Some(V::Sym) => { has_sym_arm = true; if !matches!(p, Door | KwConst | Bottom) { sym_ok = false; } }
                        None => {}
                    }
                    acc = Some(match acc { None => p, Some(q) => q.join(p) });
                }
                if has_kw_arm && has_sym_arm && kw_ok && sym_ok { return Door; }
                acc.unwrap_or(Unknown) }
            Expr::Block(b) => b.block.stmts.last().and_then(|s| match s { Stmt::Expr(e, None) => Some(self.prov(e)), _ => None }).unwrap_or(Unknown),
            _ => Unknown,
        }
    }
}
#[derive(PartialEq)] enum V { Kw, Sym }
/// Which `WatAST` variant (if any) this pattern destructures — looking THROUGH `Some(..)`/`&`.
fn pat_watast_variant(p: &Pat) -> Option<V> {
    match p {
        Pat::TupleStruct(t) => {
            let seg: Vec<String> = t.path.segments.iter().map(|s| s.ident.to_string()).collect();
            let n = seg.len();
            if n >= 2 && seg[n-2] == "WatAST" {
                return match seg[n-1].as_str() { "Keyword" => Some(V::Kw), "Symbol" => Some(V::Sym), _ => None };
            }
            t.elems.iter().find_map(pat_watast_variant)
        }
        Pat::Reference(r) => pat_watast_variant(&r.pat),
        Pat::Paren(x) => pat_watast_variant(&x.pat),
        Pat::Or(o) => o.cases.iter().find_map(pat_watast_variant),
        Pat::Ident(i) => i.subpat.as_ref().and_then(|(_, s)| pat_watast_variant(s)),
        _ => None,
    }
}

fn is_never(e: &Expr) -> bool {
    matches!(e, Expr::Return(_) | Expr::Continue(_) | Expr::Break(_))
        || matches!(e, Expr::Macro(m) if m.mac.path.is_ident("panic") || m.mac.path.is_ident("unreachable"))
        || matches!(e, Expr::Block(b) if b.block.stmts.len() == 1 && matches!(b.block.stmts.first(), Some(Stmt::Expr(x, _)) if is_never(x)))
}

#[derive(Clone, Debug)]
struct Site { file: String, line: usize, func: String, kind: &'static str, prov: Prov, literal: String, operand: String }

const CMP_METHODS: &[&str] = &["starts_with","ends_with","contains","strip_prefix","strip_suffix","eq","eq_ignore_ascii_case","ne"];

struct Scan<'a> { w: &'a World, ctx: Ctx<'a>, file: String, func: Vec<String>, sites: Vec<Site>, calls: Vec<(String, usize, Prov)> }
impl<'a> Scan<'a> {
    fn push(&mut self, sp: proc_macro2::Span, kind: &'static str, lit: String, operand: &Expr) {
        let prov = self.ctx.prov(operand);
        self.sites.push(Site { file: self.file.clone(), line: sp.start().line,
            func: self.func.last().cloned().unwrap_or_else(|| "<top>".into()), kind, prov, literal: lit, operand: render(operand) });
    }
    fn enter(&mut self, name: String, params: &[String]) {
        self.func.push(name.clone());
        let mut s = BTreeMap::new();
        for (i, p) in params.iter().enumerate() {
            s.insert(p.clone(), *self.w.param_prov.get(&(name.clone(), i)).unwrap_or(&Prov::Unknown));
        }
        self.ctx.scopes.push(s); self.ctx.cur_fn = name;
    }
    fn exit(&mut self) { self.func.pop(); self.ctx.scopes.pop(); }
}
fn render(e: &Expr) -> String { let s = format!("{}", R(e)); if s.len() > 70 { format!("{}…", &s[..70]) } else { s } }
struct R<'a>(&'a Expr);
impl<'a> std::fmt::Display for R<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.0 {
            // rune:lint(one-variant-separator, namespace) — renders a RUST module path (`crate::types::denoted_type_path`) for the diagnostic; the `::` is Rust's, not a wat enum/variant separator.
            Expr::Path(p) => write!(f, "{}", p.path.segments.iter().map(|s| s.ident.to_string()).collect::<Vec<_>>().join("::")),
            Expr::MethodCall(m) => write!(f, "{}.{}()", R(&m.receiver), m.method),
            Expr::Call(c) => write!(f, "{}(..)", R(&c.func)),
            Expr::Reference(r) => write!(f, "&{}", R(&r.expr)),
            Expr::Field(fe) => write!(f, "{}.{}", R(&fe.base), match &fe.member { syn::Member::Named(n) => n.to_string(), syn::Member::Unnamed(i) => i.index.to_string() }),
            Expr::Lit(_) => write!(f, "<lit>"),
            Expr::Unary(u) if matches!(u.op, syn::UnOp::Deref(_)) => write!(f, "*{}", R(&u.expr)),
            Expr::Paren(p) => write!(f, "({})", R(&p.expr)),
            Expr::Try(t) => write!(f, "{}?", R(&t.expr)),
            Expr::Index(i) => write!(f, "{}[..]", R(&i.expr)),
            _ => write!(f, "<expr>"),
        }
    }
}
fn is_test_mod(attrs: &[syn::Attribute], ident: &syn::Ident) -> bool {
    // rune:lint(one-variant-separator, not-a-name) — `syn::Attribute::path()` is the ATTRIBUTE's own Rust path (`cfg`, `test`); it names no wat enum and no variant.
    ident == "tests" || attrs.iter().any(|a| a.path().is_ident("cfg") && a.to_token_stream_str().contains("test"))
}
trait Tk { fn to_token_stream_str(&self) -> String; }
impl Tk for syn::Attribute { fn to_token_stream_str(&self) -> String { format!("{:?}", self.meta) } }

impl<'a, 'ast> Visit<'ast> for Scan<'a> {
    fn visit_item_mod(&mut self, m: &'ast syn::ItemMod) { if is_test_mod(&m.attrs, &m.ident) { return; } syn::visit::visit_item_mod(self, m); }
    fn visit_item_fn(&mut self, f: &'ast syn::ItemFn) {
    // rune:lint(one-variant-separator, not-a-name) — `syn::Attribute::path()` / `DirEntry::path()`: an attribute's Rust path and a FILESYSTEM path. Neither names a wat enum or a variant.
        if f.attrs.iter().any(|a| a.path().is_ident("test")) { return; }
        let p = sig_params(&f.sig); self.enter(f.sig.ident.to_string(), &p); self.visit_block(&f.block); self.exit();
    }
    fn visit_impl_item_fn(&mut self, f: &'ast syn::ImplItemFn) {
    // rune:lint(one-variant-separator, not-a-name) — `syn::Attribute::path()` / `DirEntry::path()`: an attribute's Rust path and a FILESYSTEM path. Neither names a wat enum or a variant.
        if f.attrs.iter().any(|a| a.path().is_ident("test")) { return; }
        let p = sig_params(&f.sig); self.enter(f.sig.ident.to_string(), &p); self.visit_block(&f.block); self.exit();
    }
    fn visit_block(&mut self, b: &'ast syn::Block) {
        self.ctx.scopes.push(BTreeMap::new());
        for s in &b.stmts { self.visit_stmt(s); }
        self.ctx.scopes.pop();
    }
    fn visit_stmt(&mut self, s: &'ast Stmt) {
        match s {
            Stmt::Local(l) => {
                let mut p = Prov::Unknown;
                if let Some(init) = &l.init { self.visit_expr(&init.expr); p = self.ctx.prov(&init.expr);
                    if let Some((_, d)) = &init.diverge { self.visit_expr(d); } }
                let mut bs = Vec::new(); pat_binds(&l.pat, if p == Prov::Unknown { None } else { Some(p) }, &mut bs);
                for (n, q) in bs { self.ctx.bind(n, q); }
            }
            Stmt::Item(Item::Fn(f)) => self.visit_item_fn(f),
            _ => syn::visit::visit_stmt(self, s),
        }
    }
    fn visit_expr(&mut self, e: &'ast Expr) {
        // ── record every call site's ARGUMENT PROVENANCE, for the inter-procedural fixpoint ──
        match e {
            Expr::Call(c) => { if let Some(n) = path_tail(&c.func) {
                for (i, a) in c.args.iter().enumerate() { let p = self.ctx.prov(a); self.calls.push((n.clone(), i, p)); } } }
            Expr::MethodCall(m) => { let n = m.method.to_string();
                if matches!(n.as_str(), "push" | "insert" | "extend" | "push_str") {
                    if let Some(recv) = simple_ident(&m.receiver) {
                        let v = m.args.last().map(|a| self.ctx.prov(a)).unwrap_or(Prov::Unknown);
                        self.ctx.update(&recv, v);
                    }
                }
                // an inherent method's params are offset by the receiver
                for (i, a) in m.args.iter().enumerate() { let p = self.ctx.prov(a); self.calls.push((n.clone(), i + 1, p)); self.calls.push((n.clone(), i, p)); } }
            _ => {}
        }
        match e {
            Expr::Binary(b) if matches!(b.op, syn::BinOp::Eq(_) | syn::BinOp::Ne(_)) => {
                if let Some(l) = kw_lit(&b.right) { self.push(e.span(), "eq-literal", l, &b.left); }
                else if let Some(l) = kw_lit(&b.left) { self.push(e.span(), "eq-literal", l, &b.right); }
                syn::visit::visit_expr(self, e);
            }
            Expr::MethodCall(m) if CMP_METHODS.contains(&m.method.to_string().as_str()) => {
                let hit = m.args.iter().find_map(kw_lit);
                if let Some(l) = hit { self.push(e.span(), "str-predicate", l, &m.receiver); }
                else if m.method == "contains" {
                    if let Some(t) = path_tail(&m.receiver) { if self.w.kw_lists.contains(&t) {
                        if let Some(a) = m.args.first() { self.push(e.span(), "kw-list-membership", format!("<{t}>"), a); } } }
                }
                syn::visit::visit_expr(self, e);
            }
            Expr::Match(m) => {
                self.visit_expr(&m.expr);
                let mut lits = Vec::new(); for a in &m.arms { pat_kw_lits(&a.pat, &mut lits); }
                if !lits.is_empty() { self.push(m.match_token.span, "match-arm", format!("{} arm(s), e.g. {}", lits.len(), lits[0]), &m.expr); }
                let sp = self.ctx.prov(&m.expr);
                for a in &m.arms {
                    self.ctx.scopes.push(BTreeMap::new());
                    let mut bs = Vec::new(); pat_binds(&a.pat, if sp == Prov::Unknown { None } else { Some(sp) }, &mut bs);
                    for (n, q) in bs { self.ctx.bind(n, q); }
                    if let Some((_, g)) = &a.guard { self.visit_expr(g); }
                    self.visit_expr(&a.body);
                    self.ctx.scopes.pop();
                }
            }
            Expr::Macro(mc) if mc.mac.path.is_ident("matches") => {
                if let Ok(p) = syn::parse2::<MatchesArgs>(mc.mac.tokens.clone()) {
                    self.visit_expr(&p.scrutinee);
                    let sp = self.ctx.prov(&p.scrutinee);
                    let mut lits = Vec::new(); pat_kw_lits(&p.pat, &mut lits);
                    if !lits.is_empty() { self.push(mc.span(), "matches-macro", format!("{} pat(s), e.g. {}", lits.len(), lits[0]), &p.scrutinee); }
                    self.ctx.scopes.push(BTreeMap::new());
                    let mut bs = Vec::new(); pat_binds(&p.pat, if sp == Prov::Unknown { None } else { Some(sp) }, &mut bs);
                    for (n, q) in bs { self.ctx.bind(n, q); }
                    if let Some(g) = &p.guard { self.visit_expr(g); }
                    self.ctx.scopes.pop();
                }
            }
            Expr::If(i) => {
                if let Expr::Let(le) = &*i.cond {
                    self.visit_expr(&le.expr);
                    let sp = self.ctx.prov(&le.expr);
                    self.ctx.scopes.push(BTreeMap::new());
                    let mut bs = Vec::new(); pat_binds(&le.pat, if sp == Prov::Unknown { None } else { Some(sp) }, &mut bs);
                    for (n, q) in bs { self.ctx.bind(n, q); }
                    self.visit_block(&i.then_branch);
                    self.ctx.scopes.pop();
                    if let Some((_, e2)) = &i.else_branch { self.visit_expr(e2); }
                } else { syn::visit::visit_expr(self, e); }
            }
            Expr::Closure(c) => { self.ctx.scopes.push(BTreeMap::new()); self.visit_expr(&c.body); self.ctx.scopes.pop(); }
            _ => syn::visit::visit_expr(self, e),
        }
    }
}
struct MatchesArgs { scrutinee: Expr, pat: Pat, guard: Option<Expr> }
impl syn::parse::Parse for MatchesArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let scrutinee: Expr = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let pat = Pat::parse_multi_with_leading_vert(input)?;
        let mut guard = None;
        if input.peek(syn::Token![if]) { input.parse::<syn::Token![if]>()?; guard = Some(input.parse()?); }
        let _ = input.parse::<syn::Token![,]>();
        Ok(MatchesArgs { scrutinee, pat, guard })
    }
}
fn sig_params(s: &syn::Signature) -> Vec<String> {
    s.inputs.iter().map(|a| match a { syn::FnArg::Receiver(_) => "self".into(),
        syn::FnArg::Typed(t) => { let mut v = Vec::new(); pat_binds(&t.pat, None, &mut v); v.first().map(|x| x.0.clone()).unwrap_or_default() } }).collect()
}

// ─── pass A ───
#[derive(Default)]
struct PassA { kw_consts: BTreeSet<String>, kw_lists: BTreeSet<String>, stringy: BTreeSet<String>, fns: Vec<(String, Vec<String>, syn::Block)> }
impl<'ast> Visit<'ast> for PassA {
    fn visit_item_mod(&mut self, m: &'ast syn::ItemMod) { if is_test_mod(&m.attrs, &m.ident) { return; } syn::visit::visit_item_mod(self, m); }
    fn visit_item_const(&mut self, c: &'ast syn::ItemConst) { rc(self, &c.ident.to_string(), &c.expr); }
    fn visit_item_static(&mut self, s: &'ast syn::ItemStatic) { rc(self, &s.ident.to_string(), &s.expr); }
    fn visit_item_fn(&mut self, f: &'ast syn::ItemFn) {
    // rune:lint(one-variant-separator, not-a-name) — `syn::Attribute::path()` / `DirEntry::path()`: an attribute's Rust path and a FILESYSTEM path. Neither names a wat enum or a variant.
        if f.attrs.iter().any(|a| a.path().is_ident("test")) { return; }
        if ret_is_name(&f.sig.output) { self.stringy.insert(f.sig.ident.to_string()); }
        self.fns.push((f.sig.ident.to_string(), sig_params(&f.sig), (*f.block).clone())); syn::visit::visit_item_fn(self, f); }
    fn visit_impl_item_fn(&mut self, f: &'ast syn::ImplItemFn) {
    // rune:lint(one-variant-separator, not-a-name) — `syn::Attribute::path()` / `DirEntry::path()`: an attribute's Rust path and a FILESYSTEM path. Neither names a wat enum or a variant.
        if f.attrs.iter().any(|a| a.path().is_ident("test")) { return; }
        if ret_is_name(&f.sig.output) { self.stringy.insert(f.sig.ident.to_string()); }
        self.fns.push((f.sig.ident.to_string(), sig_params(&f.sig), f.block.clone())); syn::visit::visit_impl_item_fn(self, f); }
}
/// Does this signature return A NAME — `String` / `&str` / `Cow<'_, str>`, bare or in an
/// `Option`/`Result`? Rendered from the token stream, so `crate::…::Cow<str>` counts too.
fn ret_is_name(r: &syn::ReturnType) -> bool {
    let syn::ReturnType::Type(_, t) = r else { return false };
    let s = format!("{:?}", t);
    let has = s.contains("\"String\"") || s.contains("\"str\"") || s.contains("\"Cow\"");
    has && !s.contains("\"Vec\"") && !s.contains("\"HashMap\"") && !s.contains("\"BTreeMap\"")
}

fn rc(p: &mut PassA, n: &str, e: &Expr) {
    if kw_lit(e).is_some() { p.kw_consts.insert(n.into()); return; }
    match e { Expr::Reference(r) => rc(p, n, &r.expr),
        Expr::Array(a) if !a.elems.is_empty() && a.elems.iter().all(|x| kw_lit(x).is_some()) => { p.kw_lists.insert(n.into()); }, _ => {} }
}
struct Returns { exprs: Vec<Expr> }
impl<'ast> Visit<'ast> for Returns {
    fn visit_expr(&mut self, e: &'ast Expr) {
        if let Expr::Return(r) = e { if let Some(x) = &r.expr { self.exprs.push((**x).clone()); } }
        if matches!(e, Expr::Closure(_)) { return; }
        syn::visit::visit_expr(self, e); }
    fn visit_item_fn(&mut self, _: &'ast syn::ItemFn) {}
}
fn tails(b: &syn::Block) -> Vec<Expr> {
    let mut r = Returns { exprs: vec![] }; r.visit_block(b);
    if let Some(Stmt::Expr(e, None)) = b.stmts.last() { r.exprs.push(e.clone()); } r.exprs
}
fn bind_block(ctx: &mut Ctx, b: &syn::Block) {
    for s in &b.stmts { if let Stmt::Local(l) = s {
        let p = l.init.as_ref().map(|i| ctx.prov(&i.expr)).unwrap_or(Prov::Unknown);
        let mut bs = Vec::new(); pat_binds(&l.pat, if p == Prov::Unknown { None } else { Some(p) }, &mut bs);
        for (n, q) in bs { ctx.bind(n, q); } } }
}


// ═══ THE CENSUS ═════════════════════════════════════════════════════════════════════════════

struct Census {
    files: usize,
    parse_failures: Vec<String>,
    fns: usize,
    doors: BTreeSet<String>,
    rounds: usize,
    sites: Vec<Site>,
}

/// PASS B — THE DOOR SET, DERIVED. Seeded at `SEED_DOORS` and grown by fixpoint: a fn joins when
/// it RETURNS a name and every one of its returns is door-derived. Never a hand-list.
fn derive_doors(a: &PassA, w: &mut World) {
    loop {
        let mut grew = false;
        for (name, params, block) in &a.fns {
            if w.doors.contains(name) {
                continue;
            }
            let ok = {
                let mut ctx = Ctx { w, scopes: vec![BTreeMap::new()], cur_fn: name.clone() };
                for p in params {
                    ctx.bind(p.clone(), Prov::Unknown);
                }
                bind_block(&mut ctx, block);
                let t = tails(block);
                !t.is_empty()
                    && t.iter().all(|x| matches!(ctx.prov(x), Prov::Door | Prov::KwConst))
                    && t.iter().any(|x| ctx.prov(x) == Prov::Door)
            };
            if ok {
                w.doors.insert(name.clone());
                grew = true;
            }
        }
        if !grew {
            break;
        }
    }
}

fn census(root: &Path) -> Census {
    let mut files = Vec::new();
    collect_rs(&root.join("src"), &mut files);
    files.sort();

    let mut parsed = Vec::new();
    let mut a = PassA::default();
    let mut parse_failures = Vec::new();
    for f in &files {
        let Ok(src) = std::fs::read_to_string(f) else {
            parse_failures.push(format!("{}: unreadable", f.display()));
            continue;
        };
        match syn::parse_file(&src) {
            Ok(ast) => {
                a.visit_file(&ast);
                parsed.push((f.clone(), ast));
            }
            Err(e) => parse_failures.push(format!("{}: {e}", f.display())),
        }
    }

    let mut w = World {
        kw_consts: a.kw_consts.clone(),
        kw_lists: a.kw_lists.clone(),
        stringy: a.stringy.clone(),
        ..Default::default()
    };
    w.doors = SEED_DOORS.iter().map(|s| (*s).to_string()).collect();
    for (n, p, _) in &a.fns {
        w.fn_params.insert(n.clone(), p.clone());
    }

    derive_doors(&a, &mut w);

    // PASS C — inter-procedural parameter provenance, by fixpoint over every call site in src/.
    // Optimistic start, join upward; a param with NO call site in src/ stays UNRESOLVED.
    // Monotone, so it converges; the round count is returned so a non-convergence cannot hide.
    let mut sites: Vec<Site> = Vec::new();
    let mut rounds = 0usize;
    for round in 0..PASS_C_ROUND_CAP {
        rounds = round + 1;
        sites.clear();
        let mut calls: Vec<(String, usize, Prov)> = Vec::new();
        for (f, ast) in &parsed {
            let rel = f.strip_prefix(root).unwrap_or(f).display().to_string();
            let mut sc = Scan {
                w: &w,
                ctx: Ctx { w: &w, scopes: vec![], cur_fn: String::new() },
                file: rel,
                func: vec![],
                sites: vec![],
                calls: vec![],
            };
            sc.visit_file(ast);
            sites.extend(sc.sites);
            calls.extend(sc.calls);
        }
        let mut next: BTreeMap<(String, usize), Prov> = BTreeMap::new();
        for (n, i, p) in calls {
            if !w.fn_params.contains_key(&n) {
                continue;
            }
            next.entry((n, i)).and_modify(|q| *q = q.join(p)).or_insert(p);
        }
        if next == w.param_prov {
            break;
        }
        w.param_prov = next;
    }

    Census {
        files: parsed.len(),
        parse_failures,
        fns: a.fns.len(),
        doors: w.doors.clone(),
        rounds,
        sites,
    }
}

const PASS_C_ROUND_CAP: usize = 8;

/// A per-offense escape: `// rune:lint(keyword-heresy) — <reason>` on the offending line or the
/// one above. Read from disk so the marker lives beside the code it excuses, not in this file.
fn has_rune(root: &Path, s: &Site) -> bool {
    let Ok(src) = std::fs::read_to_string(root.join(&s.file)) else { return false };
    let lines: Vec<&str> = src.lines().collect();
    (s.line.saturating_sub(2)..s.line).filter_map(|i| lines.get(i)).any(|l| l.contains("rune:lint(keyword-heresy)"))
}

// ═══ THE FROZEN ALLOWLIST — sites the discriminator flags that are NOT heresy ════════════════
//
// ⛔ Keyed by file + enclosing fn, NEVER by line number (which moves under any unrelated edit).
// Each entry carries its own reason; an entry whose reason has stopped holding must be removed,
// not left stale. This list may only SHRINK.
const ALLOWLIST: &[(&str, &str, &str)] = &[
    (
        "src/types.rs",
        "parse_type_inner",
        "NOT a name dispatch. `s.starts_with(\"wat::core::Fn(\")` is a test on the RENDERED SURFACE \
         of a parametric Fn type — the open paren is part of the literal, so it matches a printed \
         form, not an identity. No spelling of a name can reach it; `parse_type_inner`'s actual \
         identity decisions (`denoted`) are separately measured and read `cured`.",
    ),
    (
        "src/edn/render.rs",
        "type_expr_to_clojure_form",
        "A RENDERER, and the one direction where the keyword spelling is the INPUT by contract: \
         this fn exists to turn the internal `:wat::…`/`wat::…` identity into the faithful-Clojure \
         surface, so stripping the internal prefix is its whole job. It decides nothing about a \
         name that arrived from source. Its forward partner `wat_keyword_to_clojure_symbol` is in \
         the same file and the same class.",
    ),
];

// ═══ THE LEDGER ═════════════════════════════════════════════════════════════════════════════
//
// ⛔ A COUNT CANNOT DISTINGUISH "+1 new, −1 fixed" FROM "NOTHING HAPPENED", and its failure text
// cannot name the offender — so this freezes NAMES, per (file, enclosing fn), with the shape mix
// ([[feedback_a_gate_freezes_names_never_a_count]]). A new offending fn is a NEW KEY and goes red
// on its own; a new offense inside an existing fn moves that key's count and goes red there.
//
// Shape tags — the reason each row is in the ledger:
//   A = the dispatch reads ONLY a `WatAST::Keyword` payload; a symbol-spelled name is invisible
//       to it (correct value, blind dispatch — 255.9's class)
//   B = the dispatch reads BOTH payloads and then keys on the keyword spelling anyway; it SEES
//       the symbol and mis-keys it (strictly worse than A — 251.8d-ii's class)
//   C = the dispatch reads only a `WatAST::Symbol` payload and compares it to a keyword literal
//   E = a `TypeExpr::Path`/`Parametric` payload compared without the denotation door (255.12's
//       class: `wat.type/i64` and `:wat::core::i64` are one type and two strings)
//
// ⭐ THE NUMBER IS THE COUNTDOWN TO THE TERMINAL CUT. Keyword call heads become illegal when it
// reads 0 and the `.wat` corpus is converted — not before.
const LEDGER_TOTAL: usize = 229;
const FROZEN_LEDGER: &[(&str, &str, usize, &str)] = &[
    ("src/check.rs", "assignable", 5, "Ex5"),
    ("src/check.rs", "check_compound_against_expected", 1, "Ax1"),
    ("src/check.rs", "check_legacy_user_main_signature", 1, "Ax1"),
    ("src/check.rs", "check_nested_variant_map", 4, "Ex4"),
    ("src/check.rs", "check_subpattern", 19, "Ax7+Ex12"),
    ("src/check.rs", "collect_process_calls", 1, "Ax1"),
    ("src/check.rs", "collect_process_stdin_and_joins", 1, "Ax1"),
    ("src/check.rs", "collect_splice_defs_ctx", 1, "Ax1"),
    ("src/check.rs", "derived_nature", 1, "Ex1"),
    ("src/check.rs", "extract_def_binding", 1, "Ax1"),
    ("src/check.rs", "extract_redef_setter", 1, "Ax1"),
    ("src/check.rs", "infer", 2, "Ax2"),
    ("src/check.rs", "infer_accept_prime", 1, "Ex1"),
    ("src/check.rs", "infer_allow_prime", 1, "Ex1"),
    ("src/check.rs", "infer_close_prime", 2, "Ex2"),
    ("src/check.rs", "infer_config_set_bool", 1, "Ex1"),
    ("src/check.rs", "infer_defclause", 1, "Ax1"),
    ("src/check.rs", "infer_deny_prime", 1, "Ex1"),
    ("src/check.rs", "infer_holon_bundle", 1, "Ex1"),
    ("src/check.rs", "infer_list", 11, "Ax8+Ex3"),
    ("src/check.rs", "infer_match", 3, "Ax2+Ex1"),
    ("src/check.rs", "infer_nth", 1, "Ex1"),
    ("src/check.rs", "infer_option_try", 1, "Ex1"),
    ("src/check.rs", "infer_poll_prime", 4, "Ex4"),
    ("src/check.rs", "infer_polymorphic_time_arith", 5, "Ax1+Ex4"),
    ("src/check.rs", "infer_positional_accessor", 1, "Ex1"),
    ("src/check.rs", "infer_select_prime", 4, "Ex4"),
    ("src/check.rs", "infer_signal", 1, "Ex1"),
    ("src/check.rs", "infer_thread_prog_type", 2, "Ex2"),
    ("src/check.rs", "infer_try", 1, "Ex1"),
    ("src/check.rs", "is_atomizable", 2, "Ex2"),
    ("src/check.rs", "is_fn_def_form", 1, "Ax1"),
    ("src/check.rs", "is_fn_form_expr", 1, "Ax1"),
    ("src/check.rs", "is_holon_or_record", 1, "Ex1"),
    ("src/check.rs", "is_holon_or_vector", 2, "Ex2"),
    ("src/check.rs", "is_must_use_type", 2, "Ex2"),
    ("src/check.rs", "is_primitive_type_keyword_in_value_position", 1, "Ax1"),
    ("src/check.rs", "is_pure_type", 3, "Ex3"),
    ("src/check.rs", "is_shared_marker", 1, "Ex1"),
    ("src/check.rs", "is_transport_slot", 2, "Ex2"),
    ("src/check.rs", "is_type_equatable", 2, "Ex2"),
    ("src/check.rs", "is_type_orderable", 1, "Ex1"),
    ("src/check.rs", "is_wire_marker", 1, "Ex1"),
    ("src/check.rs", "map_kv_of", 1, "Ex1"),
    ("src/check.rs", "preregister_defclause_in_env", 1, "Ax1"),
    ("src/check.rs", "project_peer_io", 4, "Ex4"),
    ("src/check.rs", "set_elem_of", 1, "Ex1"),
    ("src/check.rs", "transport_marker", 1, "Ex1"),
    ("src/check.rs", "unify", 1, "Ex1"),
    ("src/check.rs", "validate_def_position_with_wrapper", 1, "Ax1"),
    ("src/check.rs", "vector_elem_of", 1, "Ex1"),
    ("src/check.rs", "walk_for_bare_primitives", 4, "Ax4"),
    ("src/closure_extract.rs", "rewrite_with_scope", 2, "Ax2"),
    ("src/closure_extract.rs", "split_body_prelude", 1, "Ax1"),
    ("src/closure_extract.rs", "walk_free_symbols", 1, "Ax1"),
    ("src/collection/infer.rs", "extract_lazyable_elem", 5, "Ex5"),
    ("src/collection/infer.rs", "infer_contains", 7, "Ex7"),
    ("src/collection/infer.rs", "infer_get", 7, "Ex7"),
    ("src/collection/map_container.rs", "of_type", 2, "Ex2"),
    ("src/collection/seq_container.rs", "of_type", 10, "Ex10"),
    ("src/declare/parse.rs", "parse_type_slot", 1, "Ax1"),
    ("src/declare/parse.rs", "try_parse_user_variadic_def_fn_form", 2, "Ex2"),
    ("src/edn/render.rs", "rewrap_option_field", 1, "Ex1"),
    ("src/freeze.rs", "is_deftest_fn", 2, "Ex2"),
    ("src/function/eval.rs", "select_defclause_clause", 1, "Ex1"),
    ("src/function/subsume.rs", "value_matches_type_by_name", 1, "Ex1"),
    ("src/holon/ast.rs", "is_holon_arg_canonical", 1, "Ax1"),
    ("src/holon/ast.rs", "try_recognize_holon_value", 1, "Ax1"),
    ("src/host/test_runner.rs", "source_has_config_setter", 2, "Ax2"),
    ("src/intrinsic/holon/atom.rs", "eval_holon_from_holon", 1, "Ax1"),
    ("src/load/loader.rs", "parse_payload_interface", 1, "Ax1"),
    ("src/load/loader.rs", "parse_verify_algo", 1, "Ax1"),
    ("src/lower.rs", "lower_bundle", 1, "Ax1"),
    ("src/lower.rs", "lower_call", 1, "Ax1"),
    ("src/macros/eval.rs", "validate_pure_total", 1, "Ax1"),
    ("src/macros/eval.rs", "validate_quasiquote_template", 3, "Ax3"),
    ("src/macros/expand.rs", "expand_make_rule_then", 1, "Ax1"),
    ("src/macros/expand.rs", "expand_make_rule_when", 1, "Ax1"),
    ("src/macros/expand.rs", "is_quasiquote_form", 1, "Ax1"),
    ("src/macros/parse.rs", "is_watast", 1, "Ex1"),
    ("src/macros/parse.rs", "is_watast_vec", 1, "Ex1"),
    ("src/macros/parse.rs", "parse_defmacro_form", 1, "Ax1"),
    ("src/match_arm.rs", "builtin_variant", 1, "Ax1"),
    ("src/resolve/boundary.rs", "is_unquote_escape", 2, "Ax2"),
    ("src/resolve/boundary.rs", "is_where_form", 1, "Ax1"),
    ("src/resolve/normalize.rs", "normalize_make_rule_when", 1, "Ax1"),
    ("src/resolve/walk.rs", "check_make_rule_when", 1, "Ax1"),
    ("src/resolve/walk.rs", "is_resolvable_call_head", 1, "Ax1"),
    ("src/rete/collect.rs", "eval_collect_rules", 1, "Ex1"),
    ("src/rete/expr_ir/mod.rs", "lower_list", 1, "Ax1"),
    ("src/rete/kernel/arm.rs", "compile_acc_fold", 1, "Bx1"),
    ("src/rete/kernel/arm.rs", "compile_user_fold_programs", 1, "Bx1"),
    ("src/rete/kernel/stratify.rs", "body_constructs_computed", 3, "Ax3"),
    ("src/rete/kernel/stratify.rs", "domain_cardinality", 1, "Ex1"),
    ("src/rete/purity.rs", "classify_expr", 3, "Ax3"),
    // ⭐ 251.8d-ii FIFTH: Bx8 → Ax8. The COUNT did not move; the SHAPE did, and it is the
    // whole point of this row. `classify_expr` now hands `head_ok` a canonical identity, so
    // the dual-raw provenance is gone from this path; what is left is
    // `runtime.rs::step_list`, which reads a `WatAST::Keyword` payload ONLY (its Symbol arm
    // returns `NoStepRule` before the purity test) and reaches here through
    // `is_effectful_op`. Keyword-only = shape A. 8d-iii or later owns the A→0 cut.
    ("src/rete/purity.rs", "effectful_by_prefix", 8, "Ax8"),
    // `intrinsic_meta` is GONE (was 3 [Bx3]) — arc 251 stone 251.8d-ii FIFTH draw. Its only
    // non-canonical caller was `head_ok`, fed by `classify_expr`'s raw head read; that read
    // now takes `canonical_identity` on its Symbol arm. The calibration row that used to
    // pin it as shape B moved to `rete/kernel/arm.rs::compile_acc_fold` (still Bx1).
    ("src/rete/purity.rs", "is_declaration_derived_construction", 2, "Ax2"),
    ("src/rete/purity.rs", "walk_rete_defn_callees", 1, "Bx1"),
    ("src/runtime.rs", "conforms_check", 4, "Ex4"),
    ("src/runtime.rs", "dispatch_keyword_head", 1, "Ax1"),
    ("src/runtime.rs", "dispatch_keyword_head_value", 3, "Ax3"),
    ("src/runtime.rs", "eval_inner", 3, "Ax3"),
    ("src/runtime.rs", "eval_tail", 1, "Ax1"),
    ("src/runtime.rs", "is_builtin_primitive", 1, "Ex1"),
    ("src/runtime.rs", "is_match_canonical", 1, "Ax1"),
    ("src/runtime.rs", "parse_verify_algo_keyword", 1, "Ax1"),
    ("src/runtime.rs", "resolve_verify_payload", 1, "Ax1"),
    ("src/runtime.rs", "step_list", 1, "Ax1"),
    ("src/runtime.rs", "try_match_pattern", 5, "Ax5"),
    ("src/types/defstruct.rs", "splice_target", 1, "Ax1"),
];

fn manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// (file, fn) -> count, after the allowlist and the per-offense runes are applied.
/// (file, enclosing fn) -> (count, shape mix).
type LedgerMap = BTreeMap<(String, String), (usize, String)>;

fn ledger(root: &Path, c: &Census) -> (LedgerMap, Vec<String>) {
    let mut map: BTreeMap<(String, String), (usize, BTreeMap<char, usize>)> = BTreeMap::new();
    let mut lines = Vec::new();
    for s in &c.sites {
        if !s.prov.heretical() {
            continue;
        }
        if ALLOWLIST.iter().any(|(f, n, _)| *f == s.file && *n == s.func) {
            continue;
        }
        if has_rune(root, s) {
            continue;
        }
        let e = map.entry((s.file.clone(), s.func.clone())).or_insert((0, BTreeMap::new()));
        e.0 += 1;
        *e.1.entry(s.prov.shape().chars().next().unwrap()).or_insert(0) += 1;
        lines.push(format!(
            "{}:{}  fn {}  [{} / {}]  operand `{}`  vs {}",
            s.file, s.line, s.func, s.kind, s.prov.shape(), s.operand, s.literal
        ));
    }
    let shaped = map
        .into_iter()
        .map(|(k, (n, m))| {
            let mix = m.iter().map(|(c, v)| format!("{c}x{v}")).collect::<Vec<_>>().join("+");
            (k, (n, mix))
        })
        .collect();
    (shaped, lines)
}

/// ⛔ NON-VACUITY. Every number below is a claim about a tree the instrument actually reached.
/// A `violations.is_empty()`-style green over a walk that found nothing proves nothing
/// (`[[feedback_a_green_from_a_mis_aimed_probe_is_indistinguishable_from_a_working_gate]]`).
#[test]
fn the_discriminator_reaches_the_tree_and_its_passes_converge() {
    let root = manifest();
    let c = census(&root);

    assert!(
        c.parse_failures.is_empty(),
        "\n⛔ syn failed to parse {} file(s) under src/. A file this lint cannot parse is a file \
         it silently does not measure, so this is a hard error, never a skip:\n{}\n",
        c.parse_failures.len(),
        c.parse_failures.join("\n")
    );
    assert!(
        c.files > 300,
        "the walk of src/ found only {} .rs files (333 measured 2026-09-22) — it is not reaching \
         the tree, so any number below is about a directory, not the crate",
        c.files
    );
    assert!(
        c.fns > 3000,
        "only {} fn bodies collected (3690 measured 2026-09-22) — pass A lost the tree",
        c.fns
    );
    assert!(
        !c.sites.is_empty() && c.sites.len() > 300,
        "only {} decision sites found (364 measured 2026-09-22) — the shape detector is not firing",
        c.sites.len()
    );
    // The DOOR SET is derived, not declared: seeded at two roots and grown by fixpoint. If it
    // ever collapses to the seeds, every site reads heretical and the ledger is meaningless.
    assert!(
        c.doors.len() >= SEED_DOORS.len() + 3,
        "the derived door set collapsed to {:?} — the fixpoint found no normalizer beyond its \
         seeds, so `cured` can no longer be distinguished from heresy",
        c.doors
    );
    for must in [
        // the two seeds
        "canonical_identity",
        "ns_to_wat_path",
        // grown by the fixpoint — each is a door this arc actually built or moved
        "canonical_identity_of", // 255.9's cure, form_match.rs
        "head_fqdn",             // 251.9's one door, via THE DUAL-ARM RULE
        "type_denotation",       // 255.1's denotation door
        "denoted_type_path",     // 255.12's carve-out, types.rs
    ] {
        assert!(
            c.doors.contains(must),
            "`{must}` is not in the derived door set {:?} — the closure lost a normalizer this \
             arc landed, and every site downstream of it will now read heretical",
            c.doors
        );
    }
    // ⛔ `identity_text` reads BOTH spellings and normalizes NEITHER (its own doc: "then
    // `canonical_identity` is the one key"). If the closure ever calls it a door, the dual-arm
    // rule has broken and shape B becomes invisible — which is the 251.8d-ii defect.
    // rune:lint(loose-assert) — not a string match at all: `c.doors` is a `BTreeSet<String>` and
    // this is SET MEMBERSHIP on an exact key. `assert_eq!` has nothing to compare against — the
    // claim is "this one name is absent from a set whose other members are asserted by name three
    // lines above".
    assert!(
        !c.doors.contains("identity_text"),
        "`identity_text` was admitted to the door set. It returns the RAW text of either spelling \
         — treating it as a normalizer makes shape B (dual-raw) unrepresentable."
    );
    assert!(
        c.rounds < PASS_C_ROUND_CAP,
        "pass C hit its {PASS_C_ROUND_CAP}-round cap without converging (used {}), so the \
         parameter provenance below is a partial fixpoint, not a fixpoint",
        c.rounds
    );
}

/// ⭐⭐ THE GATE WITH TEETH. A ledger that misses a heretic we already know about is not a ledger,
/// and a ledger that convicts a site this arc CURED is worse than none. Every row is a site this
/// arc already judged, in a SCORE, with a probe.
#[test]
fn the_discriminator_separates_the_cured_from_the_open() {
    let root = manifest();
    let c = census(&root);
    let (map, _) = ledger(&root, &c);

    // ── MUST NOT BE FLAGGED — cured, each by a named stone ──
    // (file, fn, which stone cured it)
    // ⛔ Keyed per DECISION — (file, enclosing fn, the literal it is compared against) — never
    // per fn. `is_type_equatable` and `validate_pure_total` each hold a CURED decision AND a
    // separate raw one; a per-fn key would have hidden both facts behind one another.
    const CURED: &[(&str, &str, &str, &str)] = &[
        ("src/load/loader.rs", "match_load_form", ":wat::load-file!", "255.9 — the head routed through canonical_identity_of"),
        ("src/load/loader.rs", "scan_for_setter", ":wat::config::set-", "255.9 — the head routed through canonical_identity_of"),
        ("src/macros/eval.rs", "validate_pure_total", ":wat::core::quote", "255.10 — head_candidates carries both spellings, the symbol one through canonical_identity"),
        ("src/macros/eval.rs", "validate_pure_total", ":wat::core::quasiquote", "255.10 — same candidate list"),
        ("src/macros/eval.rs", "refuse_expand_only_in_program", ":wat::core::defmacro", "255.10/255.11 — reads head_fqdn"),
        ("src/edn/render.rs", "edn_to_typed_value_inner", ":wat::core::i64", "255.12 — the coerce table matched against denoted_type_path"),
        ("src/types.rs", "from_root_keyword", ":wat::core::Struct", "255.1 — matched against canonical_identity"),
        ("src/types.rs", "from_marker_keyword", ":wat::enum::Pure", "255.1 — matched against canonical_identity"),
        ("src/types.rs", "classify_type_decl", ":wat::core::structtype", "255.x — head read through head_fqdn"),
        ("src/types.rs", "is_subtype", ":wat::core::Value", "255.8 — compared through the denotation door"),
        ("src/check.rs", "is_type_equatable", ":wat::core::i64", "255.12 — the equatable table matched against `denoted`"),
        ("src/check.rs", "is_type_orderable", ":wat::core::i64", "255.12 — the orderable table matched against `denoted`"),
        ("src/check.rs", "walk_for_restricted_call", "", "255.11 — the capability wall reads both spellings and holds NO keyword literal comparison at all, so it must produce no site whatsoever"),
        ("src/rete/purity.rs", "intrinsic_meta", ":wat::core::+", "251.8d-ii FIFTH — classify_expr's Symbol arm takes canonical_identity, so the only head this table can be handed is an identity"),
    ];
    let mut wrongly_flagged = Vec::new();
    for (f, n, lit, why) in CURED {
        for s in &c.sites {
            if !s.prov.heretical() || s.file != *f || s.func != *n {
                continue;
            }
            if !lit.is_empty() && !s.literal.contains(lit) {
                continue;
            }
            if ALLOWLIST.iter().any(|(af, an, _)| af == f && an == n) || has_rune(&root, s) {
                continue;
            }
            wrongly_flagged.push(format!(
                "  {}:{}  fn {}  [{} / {}]  operand `{}` vs {}\n      CURED BY: {why}",
                s.file, s.line, s.func, s.kind, s.prov.shape(), s.operand, s.literal
            ));
        }
    }
    assert!(
        wrongly_flagged.is_empty(),
        "\n\n⛔⛔ THE DISCRIMINATOR CONVICTED A SITE THIS ARC ALREADY CURED. Either a cure was \n\
         reverted, or the discriminator cannot tell a normalized name from a raw one — and in the \n\
         second case every number it produces is noise.\n\n{}\n",
        wrongly_flagged.join("\n")
    );

    // ── MUST BE FLAGGED — still open ──
    //
    // ⛔ THE BRIEF IS CORRECTED HERE. It names `src/collection/transform.rs:304-340`. That range
    // holds NO keyword comparison — it is `sort$native`'s CALL of the purity gate (measured: zero
    // `":wat::` literals between lines 290 and 360, and every such literal in the whole file is an
    // `OP` label, a `#[wat_intrinsic]` attribute or message text). The comparison 251.8d-ii's
    // fourth draw convicted — "`wat.core/<` is not proven pure", 468 log occurrences, ONE callee —
    // lives in `src/rete/purity.rs`, and the data flow names it exactly: `classify_expr` reads its
    // head as `Keyword(k) => k.as_str()` / `Symbol(id) => id.as_str()` with NO door (shape B), and
    // that provenance reaches `intrinsic_meta` and `effectful_by_prefix`, whose tables are
    // keyword-keyed.
    const OPEN: &[(&str, &str, &str)] = &[
        ("src/rete/purity.rs", "effectful_by_prefix", "the effect-namespace prefix test — still open, but now shape A: its remaining raw feed is runtime.rs::step_list's keyword-only head, not classify_expr"),
        ("src/rete/purity.rs", "classify_expr", "the quote/quasiquote/holon-literal data guard, keyword-only"),
        ("src/function/subsume.rs", "value_matches_type_by_name", "the Aggregate arm 255.12 §6.4 left raw and declared 'a reading, not a probe'"),
    ];
    let mut missed = Vec::new();
    for (f, n, why) in OPEN {
        if !map.contains_key(&((*f).to_string(), (*n).to_string())) {
            missed.push(format!("  {f}  fn {n}\n      EXPECTED HERESY: {why}"));
        }
    }
    assert!(
        missed.is_empty(),
        "\n\n⛔⛔ THE LEDGER MISSED A HERETIC THIS ARC HAS ALREADY CONVICTED. A ledger that cannot \n\
         see a known defect is not a ledger — it is a number.\n\n{}\n",
        missed.join("\n")
    );

    // ⭐ And the SHAPE matters, not just the presence: the instrument must still be able to SEE
    // shape B — a symbol payload reaching a keyword-keyed decision — or every "cured" verdict it
    // hands out is worthless. 255.13 anchored this row on `rete::purity::intrinsic_meta`, which
    // 251.8d-ii's FIFTH draw then CURED (it is in the CURED table above now). ⛔ The row is
    // RE-ANCHORED rather than deleted: deleting it would have retired the instrument's only proof
    // that pass C still carries provenance across the call graph, at exactly the moment the cure
    // made that proof matter most. The new anchor is 255.13 §4.3's own find —
    // `rete/kernel/arm.rs::compile_acc_fold`, the accumulator lowering that reads both payloads
    // raw and then matches `head` against `":wat::rete::acc::count"`. 255.9 dispositioned it
    // *"no (already both) — fine"*; reading both spellings is HALF the cure, and this row is the
    // standing reminder that the other half is the door.
    let (n, mix) = map
        .get(&("src/rete/kernel/arm.rs".to_string(), "compile_acc_fold".to_string()))
        .expect("compile_acc_fold must be in the ledger — it is the shape-B calibration anchor");
    // rune:lint(loose-assert) — a targeted PRESENCE check over a shape-MIX summary, deliberately
    // independent of the count. The exact mix ("Bx1") is already pinned byte-for-byte by
    // FROZEN_LEDGER two tests over; this row asserts only that the CLASS is still detectable.
    assert!(
        mix.contains('B'),
        "`rete::kernel::arm::compile_acc_fold` is in the ledger as {n} site(s) [{mix}], but NOT as \
         shape B. Shape B is the claim that a SYMBOL-spelled head reaches a keyword-keyed table — \
         the exact mechanism of the `wat.core/<` failure. If no row can carry it, the \
         discriminator has stopped distinguishing the dangerous class from the blind one."
    );
}

/// ⭐ THE GUARD HAS FAILED ONCE, BY CONSTRUCTION, AND HERE IS THE CONSTRUCTION.
///
/// A gate that has never failed is not a gate (R59, `NISI FRANGAS, NIHIL PROBAS`). This runs the
/// discriminator over synthetic sources holding one heresy of each shape and one cure of each
/// shape, and asserts each verdict — so a refactor that quietly makes the analyzer answer `cured`
/// to everything cannot pass.
#[test]
fn the_discriminator_convicts_a_synthetic_heretic_and_clears_its_cure() {
    fn verdict(body: &str) -> Vec<(String, String)> {
        let file: syn::File = syn::parse_str(body).expect("synthetic source must parse");
        let mut a = PassA::default();
        a.visit_file(&file);
        let mut w = World {
            kw_consts: a.kw_consts.clone(),
            kw_lists: a.kw_lists.clone(),
            stringy: a.stringy.clone(),
            ..Default::default()
        };
        w.doors = SEED_DOORS.iter().map(|s| (*s).to_string()).collect();
        for (n, p, _) in &a.fns {
            w.fn_params.insert(n.clone(), p.clone());
        }
        derive_doors(&a, &mut w);
        let mut sc = Scan {
            w: &w,
            ctx: Ctx { w: &w, scopes: vec![], cur_fn: String::new() },
            file: "<synthetic>".into(),
            func: vec![],
            sites: vec![],
            calls: vec![],
        };
        sc.visit_file(&file);
        sc.sites.iter().map(|s| (s.func.clone(), s.prov.shape().to_string())).collect()
    }

    // shape A — a keyword-only head dispatch
    let a = verdict(
        r#"fn f(node: &WatAST) -> bool {
               match node { WatAST::Keyword(k, _) => k == ":wat::core::defn", _ => false }
           }"#,
    );
    assert_eq!(a, vec![("f".to_string(), "A-keyword-only".to_string())], "shape A not convicted: {a:?}");

    // shape B — reads BOTH spellings, then keys on the keyword one
    let b = verdict(
        r#"fn f(node: &WatAST) -> bool {
               let head = match node {
                   WatAST::Keyword(k, _) => k.as_str(),
                   WatAST::Symbol(id, _) => id.as_str(),
                   _ => return false,
               };
               head == ":wat::core::defn"
           }"#,
    );
    assert_eq!(b, vec![("f".to_string(), "B-dual-raw".to_string())], "shape B not convicted: {b:?}");

    // shape C — a symbol payload compared to a keyword literal
    let c = verdict(
        r#"fn f(node: &WatAST) -> bool {
               match node { WatAST::Symbol(id, _) => id.as_str() == ":wat::core::defn", _ => false }
           }"#,
    );
    assert_eq!(c, vec![("f".to_string(), "C-symbol-raw".to_string())], "shape C not convicted: {c:?}");

    // shape E — a TypeExpr path compared without the denotation door
    let e = verdict(
        r#"fn f(t: &TypeExpr) -> bool {
               match t { TypeExpr::Path(p) => p == ":wat::core::i64", _ => false }
           }"#,
    );
    assert_eq!(e, vec![("f".to_string(), "E-typepath-raw".to_string())], "shape E not convicted: {e:?}");

    // ⭐ THE CURES — the SAME four shapes, each through its door, must all read `cured`.
    let cured_b = verdict(
        r#"fn f(node: &WatAST) -> bool {
               let head = match node {
                   WatAST::Keyword(k, _) => k.as_str().to_string(),
                   WatAST::Symbol(id, _) => canonical_identity(id.as_str()),
                   _ => return false,
               };
               head == ":wat::core::defn"
           }"#,
    );
    assert_eq!(cured_b, vec![("f".to_string(), "cured".to_string())], "the dual-arm CURE was still convicted: {cured_b:?}");

    // ⭐ This one carries the DOOR CHAIN with it, so it also proves pass B: `denoted_type_path`
    // is NOT in `SEED_DOORS` — the fixpoint has to DERIVE it, two hops from `canonical_identity`,
    // exactly as it does in `src/`.
    let cured_e = verdict(
        r#"fn canonical_identity(s: &str) -> String { s.to_string() }
           fn type_denotation(s: &str) -> String { canonical_identity(s) }
           fn denoted_type_path(p: &str) -> String { type_denotation(p) }
           fn f(t: &TypeExpr) -> bool {
               match t { TypeExpr::Path(p) => denoted_type_path(p) == ":wat::core::i64", _ => false }
           }"#,
    );
    assert_eq!(cured_e, vec![("f".to_string(), "cured".to_string())], "the denotation CURE was still convicted (or pass B failed to DERIVE `denoted_type_path` from the seed): {cured_e:?}");

    // ⛔ AND THE NEGATIVE CONTROL FOR THE INSTRUMENT ITSELF: message text and an `OP` label are
    // NOT decisions. If these ever count, the ledger has become the orchestrator's 7 233.
    let noise = verdict(
        r#"fn f(n: usize) -> String {
               const OP: &str = ":wat::core::map";
               format!("malformed {OP} form: expected 2 args, got {n}")
           }"#,
    );
    assert!(noise.is_empty(), "message text and an OP label were counted as decisions: {noise:?}");
}

/// THE LEDGER ITSELF — frozen per (file, fn), with its shape mix.
#[test]
fn the_heresy_ledger_matches_its_frozen_census() {
    let root = manifest();
    let c = census(&root);
    let (map, lines) = ledger(&root, &c);

    let frozen: BTreeMap<(String, String), (usize, &str)> = FROZEN_LEDGER
        .iter()
        .map(|(f, n, ct, sh)| (((*f).to_string(), (*n).to_string()), (*ct, *sh)))
        .collect();

    let mut appeared = Vec::new();
    let mut grew = Vec::new();
    let mut shrank = Vec::new();
    let mut vanished = Vec::new();
    let mut shape_moved = Vec::new();
    for (k, (n, mix)) in &map {
        match frozen.get(k) {
            None => appeared.push(format!("  + {}  fn {}  {n} site(s) [{mix}]", k.0, k.1)),
            Some((fz, fmix)) if n > fz => {
                grew.push(format!("  ↑ {}  fn {}  {fz} [{fmix}] → {n} [{mix}]", k.0, k.1))
            }
            Some((fz, fmix)) if n < fz => {
                shrank.push(format!("  ↓ {}  fn {}  {fz} [{fmix}] → {n} [{mix}]", k.0, k.1))
            }
            // ⭐ 251.8d-ii FIFTH — THE SHAPE IS PART OF THE FREEZE, not decoration.
            //
            // ⛔ This arm did not exist and its absence was a hole in BOTH directions. A row whose
            // COUNT is unchanged while its SHAPE moves was invisible: `effectful_by_prefix` went
            // Bx8 → Ax8 under this stone's cure and the ratchet said nothing, and the same silence
            // would have covered Ax8 → Bx8 — a site SILENTLY ACQUIRING the strictly-worse
            // dual-raw class, which is the defect this whole ledger exists to count. 255.13 §7
            // says it outright: *"each row's shape tag IS its reason for being in the ledger"*.
            // A freeze that does not check the reason freezes a number.
            Some((fz, fmix)) if fmix != mix => shape_moved.push(format!(
                "  ⇄ {}  fn {}  {fz} site(s) [{fmix}] → [{mix}]  (count unchanged)",
                k.0, k.1
            )),
            _ => {}
        }
    }
    for (k, (fz, fmix)) in &frozen {
        if !map.contains_key(k) {
            vanished.push(format!("  − {}  fn {}  was {fz} [{fmix}], now 0", k.0, k.1));
        }
    }
    let total: usize = map.values().map(|(n, _)| n).sum();

    let regressed = !appeared.is_empty() || !grew.is_empty();
    assert!(
        !regressed,
        "\n\n🔥🔥🔥 NEW KEYWORD HERESY — the dual-support scaffold GREW.\n\
         The ledger was {LEDGER_TOTAL}; it is now {total}.\n\n\
         A decision made by comparing a name that can arrive in EITHER spelling against a\n\
         keyword-spelled literal, without passing it through the identity door first, is the\n\
         defect class arc 255 has convicted eight times (255.9 · 255.10 · 255.11 · 255.12 ·\n\
         251.8d-ii). THE FIX IS A DOOR, NEVER A SECOND ARM:\n\
         \n\
         · a form HEAD              → `form_match::canonical_identity_of` / `declare::parse::head_fqdn`\n\
         · a bare name string       → `edn::render::canonical_identity`\n\
         · a TYPE path              → `types::denoted_type_path` (NOT raw `type_denotation` —\n\
                                      it collapses `:wat::type::Infer`, see 255.12 §4)\n\
         · a parametric head        → `types::parametric_heads_unify`\n\
         · two whole TypeDefs       → `types::type_defs_same`\n\
         \n\
         If the site genuinely cannot arrive in the other spelling, say so ON IT:\n\
         `// rune:lint(keyword-heresy) — <reason>`, on the offending line or the one above.\n\
         \n\
         NEW OFFENDING FUNCTIONS:\n{}\n\
         GREW:\n{}\n\
         \n\
         Every offense, with its operand and the literal it was compared against:\n{}\n",
        if appeared.is_empty() { "  (none)".into() } else { appeared.join("\n") },
        if grew.is_empty() { "  (none)".into() } else { grew.join("\n") },
        lines.join("\n"),
    );

    assert!(
        shape_moved.is_empty(),
        "\n\n⇄⇄ A LEDGER ROW CHANGED SHAPE WITHOUT CHANGING COUNT.\n\
         The shape tag is the row's REASON for being in the ledger (A = keyword-only dispatch,\n\
         B = dual-raw — it SEES the symbol spelling and keys on the keyword anyway, the strictly\n\
         worse class; E = a type path without the denotation door). A → B is a regression the\n\
         count can never show; B → A is real progress the count can never show either. Re-freeze\n\
         the mix in FROZEN_LEDGER, and say in the commit which direction it moved.\n\n{}\n",
        shape_moved.join("\n")
    );

    let improved = !shrank.is_empty() || !vanished.is_empty();
    assert!(
        !improved,
        "\n\n⭐ THE LEDGER SHRANK — {LEDGER_TOTAL} → {total}. This is the good direction, and the\n\
         ratchet is deliberate: the frozen census below has to be tightened by hand so the number\n\
         can never drift back up silently. Update `LEDGER_TOTAL` and the rows named here.\n\
         \n\
         CURED:\n{}\n\
         GONE ENTIRELY:\n{}\n",
        if shrank.is_empty() { "  (none)".into() } else { shrank.join("\n") },
        if vanished.is_empty() { "  (none)".into() } else { vanished.join("\n") },
    );

    assert_eq!(
        total, LEDGER_TOTAL,
        "the per-key census agrees with the frozen table but the total does not — LEDGER_TOTAL is \
         out of step with FROZEN_LEDGER's own rows"
    );
    let frozen_total: usize = FROZEN_LEDGER.iter().map(|(_, _, n, _)| n).sum();
    assert_eq!(
        frozen_total, LEDGER_TOTAL,
        "FROZEN_LEDGER's rows sum to {frozen_total}, but LEDGER_TOTAL says {LEDGER_TOTAL}"
    );
}
