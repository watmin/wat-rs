# BRIEF — strike 5: the MapContainer registry + route `assoc`

**Model:** sonnet. **cwd:** `/home/watmin/work/holon/wat-rs/` (run `pwd` first; reject any `.claude/worktrees/`
path → re-cd; use `git -C /home/watmin/work/holon/wat-rs`). No worktrees. **Read the DESIGN first:**
`docs/arc/2026/06/278-rules-engine/DESIGN-STONE-seq-container-registry-strike5-mapcontainer.md`.

## The work, in one paragraph

Mint `MapContainer` — the keyed-collection sibling of `SeqContainer` — for `{HashMap, PersistentMap, Record}`,
and route the keyed op `assoc` through it on BOTH sides (runtime + checker), using the strike-4 Form-1 pattern
(exhaustive `match map_container`, capability gate, named arms, **no `_`**). Behavior must not change — same
helpers, same error messages, same accepted set (`assoc` has no live drift: checker ≡ runtime, verified). This is
the dependency strike 6 needs; `get`/`contains?`/`length`/`empty?` are OUT of scope (strike 6).

## Room 1 (NEW) — `src/collection/map_container.rs`

Mirror `src/collection/seq_container.rs` structure (read it first for the house style — module doc, the
EXHAUSTIVENESS GUARANTEE comment, `#[derive(Debug, Clone, Copy, PartialEq, Eq)]`):

```rust
pub(crate) enum MapContainer { HashMap, PersistentMap, Record }   // keyed collections; Record = ordered tagged-map

impl MapContainer {
    /// Runtime classifier — the ONLY Value→MapContainer map. Pure.
    pub(crate) fn of_value(v: &Value) -> Option<MapContainer> {
        match v {
            Value::wat__std__HashMap(_)        => Some(MapContainer::HashMap),
            Value::wat__core__PersistentMap(_) => Some(MapContainer::PersistentMap),
            Value::wat__Record { .. } | Value::wat__holon__Record { .. } => Some(MapContainer::Record),
            _ => None,
        }
    }
    /// Checker classifier — the ONLY TypeExpr→MapContainer map. Takes the TypeEnv because Record is
    /// classified by SUBTYPE (user records are subtypes of :wat::Record / :wat::holon::Record).
    pub(crate) fn of_type(reduced: &TypeExpr, types: &TypeEnv) -> Option<MapContainer> {
        match reduced {
            TypeExpr::Parametric { head, .. } if head == "wat::core::HashMap"       => Some(MapContainer::HashMap),
            TypeExpr::Parametric { head, .. } if head == "wat::core::PersistentMap" => Some(MapContainer::PersistentMap),
            TypeExpr::Path(p) if crate::types::is_subtype(p, ":wat::Record", types)
                             || crate::types::is_subtype(p, ":wat::holon::Record", types) => Some(MapContainer::Record),
            _ => None,
        }
    }
    // Capability table — CURRENT TRUTH (mirror SeqContainer's `true`=supported / `false`=N-A-or-gap convention).
    pub(crate) fn can_assoc(self) -> bool    { match self { HashMap=>true,  PersistentMap=>true,  Record=>true  } }
    pub(crate) fn keyed_lookup(self) -> bool { match self { HashMap=>true,  PersistentMap=>true,  Record=>false } } // ○gap (get-by-keyword: strike 6+)
    pub(crate) fn has_key(self) -> bool      { match self { HashMap=>true,  PersistentMap=>true,  Record=>false } } // ○gap
    pub(crate) fn measurable(self) -> bool   { match self { HashMap=>true,  PersistentMap=>true,  Record=>false } } // ○gap
}
```
- Use the exact `TypeEnv` type `env.types()` returns (grep `pub fn types(` / its callers in `infer.rs`). Add
  `use` lines mirroring seq_container (`crate::types::TypeExpr`, `crate::value::Value`, the `TypeEnv` path).
- Document on the enum: **`Record` is also ORDERED** (declaration order; `struct_form` is a Vec) — a real
  property with no op consumer yet; promote to an `ordered()` capability when keys/vals/seq-over-pairs is built.
- `keyed_lookup`/`has_key`/`measurable` are **defined now, consumed in strike 6** (same staged fill SeqContainer
  did). If they warn dead_code, exercise them in the probe (Room 4) — do NOT `#[allow]` them away.

## Room 2 — `src/collection/mod.rs:129`

Add `pub(crate) mod map_container;` next to `pub(crate) mod seq_container;`.

## Room 3 — `src/runtime.rs:8699` (`eval_assoc`)

Replace the `match &arg0_val { … }` (lines ~8716-8728) with the Form-1 gated dispatch:
```rust
use crate::collection::map_container::MapContainer;
match MapContainer::of_value(&arg0_val) {
    Some(m) if m.can_assoc() => match m {                       // exhaustive over MapContainer, no `_`
        MapContainer::HashMap       => crate::collection::eval::hashmap_assoc_inner(&arg0_val, &arg1_val, &arg2_val),
        MapContainer::PersistentMap => crate::collection::eval::persistentmap_assoc_inner(&arg0_val, &arg1_val, &arg2_val),
        MapContainer::Record        => record_assoc_inner(arg0_val, arg1_val, arg2_val, list_span, sym), // OWNED — same call as today
    },
    Some(_) => Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
        op: OP.into(), expected: "HashMap<K,V>, PersistentMap<K,V>, or :wat::Record",
        got: Box::new(ValueSnapshot::of(&arg0_val)) } }.into()),     // can_assoc()==false (none today)
    None => Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
        op: OP.into(), expected: "HashMap<K,V>, PersistentMap<K,V>, or :wat::Record",
        got: Box::new(ValueSnapshot::of(&arg0_val)) } }.into()),
}
```
- The `of_value(&arg0_val)` borrow ends before the inner `match m` (MapContainer is `Copy`), so the `Record` arm
  can move `arg0_val`/`arg1_val`/`arg2_val` into `record_assoc_inner` exactly as today. Same error message text.

## Room 4 — `src/collection/infer.rs:353` (`infer_assoc`)

The checker currently hand-rolls `match &reduced { Parametric "wat::core::HashMap" => …, Parametric
"…PersistentMap" => …, Path if is_subtype(:wat::Record|:wat::holon::Record) => … }`. Route the **classification**
through `MapContainer::of_type(&reduced, env.types())` + `can_assoc`, exhaustive over the enum:
```rust
match MapContainer::of_type(&reduced, env.types()) {
    Some(m) if m.can_assoc() => match m {                       // exhaustive, no `_`
        MapContainer::HashMap | MapContainer::PersistentMap => { /* existing K/V extraction + unify arg1~K, arg2~V */ }
        MapContainer::Record => { /* existing keyword-key + free-∀T-value Record arm */ }
    },
    Some(_) => { /* existing TypeMismatch */ }
    None    => { /* existing Var-defers + TypeMismatch arms — keep the unresolved-Var backstop policy */ }
}
```
- **Keep ALL existing projection/unification bodies byte-identical** — only the *classification* moves to
  `of_type`. The `reduced` TypeExpr stays in scope; each arm extracts its type-args from it as today.
- Preserve the `TypeExpr::Var(_) => None`-style unresolved-Var backstop (defers to runtime) — that's the `None`
  arm now (of_type returns None for a Var); keep the existing behavior.

## Room 5 (NEW) — `tests/probe_map_container.rs`

Mirror `tests/probe_seq_container_registry.rs`. Reachability + behavior net:
- Every `MapContainer` variant is produced by `of_value` for a real `Value` (HashMap, PersistentMap, `wat__Record`,
  AND `wat__holon__Record` both → `Record`).
- `assoc` round-trips on each map kind + on a record (a wat program per kind: build, assoc, read back).
- Exercises the capability methods (`can_assoc`/`keyed_lookup`/`has_key`/`measurable`) so none are dead_code.
- A positive + negative: `assoc` on a non-keyed value (e.g. a Vector) → teaching TypeMismatch.

## Verify (do all; report numbers)

1. `cargo build --release` — green, no new warnings (baseline 26).
2. **Compile-forcing proof:** temporarily add `    ProbeMapDummy,` to `MapContainer`, `cargo build` → confirm it
   errors at `eval_assoc`'s `match m`, `infer_assoc`'s `match m`, AND the 4 capability methods. Record the sites.
   Then REMOVE it; confirm green. (Do not commit ProbeMapDummy.)
3. `cargo test --release 2>&1 | grep "test result:"` — lib baseline **941 passed; 36 failed; 1 ignored** must
   hold (36 stays 36, 941 stays 941). `probe_map_container` green. `probe_seq_container_*` still green.
4. `cargo clippy --release` — no new warnings.

## STOP triggers (reject + report; do NOT improvise)

- **STOP-1:** if `MapContainer::of_type` cannot cleanly replace `infer_assoc`'s classification without changing a
  projection/unification body or the unresolved-Var backstop — STOP, report. (This strike moves classification
  only.)
- **STOP-2:** if routing changes any `assoc` behavior (different error, different accepted set, a record path
  altered) — STOP. Behavior-preserving only.
- **STOP-3:** if the `TypeEnv` type/borrowing for `of_type` won't compose with `infer_assoc`'s `env.types()` —
  STOP, report the signature mismatch (don't clone the world to force it).

## Out of scope (affirmative cut)

`get`/`contains?`/`length`/`empty?` routing → strike 6. No `RecordContainer` (Record is in MapContainer). No new
`assoc` behavior. `keyed_lookup`/`has_key`/`measurable` defined-but-unused-by-ops until strike 6 (kept live by the
probe).
