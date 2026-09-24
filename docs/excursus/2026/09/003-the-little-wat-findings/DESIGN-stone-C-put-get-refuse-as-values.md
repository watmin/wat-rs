# DESIGN — STONE C: `Lru/put` and `Lru/get` stop panicking

**Drawn 2026-09-23.** Builder's ruling: *"i want the least amount of panics possible (which may be
zero…)"*. Supersedes the arc-109 NOTE's "LEAVE put/get" (recorded there at `a8a3d4bd4`).

## The defect, driven — reachable from well-typed programs

```
an Lru handle as the key, direct            check=0  run=2  Rust panic
the same, laundered through a generic K      check=0  run=2  Rust panic
```
The NOTE's defence — *"the checker already rejects an opaque-typed key at most call sites"* — is
false for both. `src/rust_deps/cache.rs:151-170`: each verb calls `value_is_hashable(&k)` and then
`panic!`s.

## ⭐ The precedent is HashMap, and it splits by verb

`src/collection/eval.rs`, same predicate family (`value_is_key_hashable`):
- **insert** with an unhashable key → raises `RuntimeErrorKind::TypeMismatch` — a wat error.
- **`contains-key?`** with an unhashable key → **`false`**, commented *"never inserted"*: a key that
  cannot be stored is a guaranteed miss, so the answer is total and needs no error at all.

So: **`put` raises a wat `TypeMismatch`; `get` returns `None`.** Same shape as `i64::/` (raises
`DivisionByZero` with the user's span) — a refusal INSIDE the language.

## The ONE contract decision — raise, do not return a Result

Stone A made `Lru/new` return a `Result`, because its input arrives from DURABLE STORAGE at
rehydration with no caller in the frame. `put`/`get` are different: the key is in the caller's
hand at the call. The siblings RAISE there, and a `Result` would churn **56 call sites** across
`wat/cache.wat`, `wat-tests/cache/HolographicLru.wat`, `tests/rete/probe_arc278_cache_lru.wat` and
others for no gain in honesty. **Match HashMap: raise on `put`, miss on `get`. No signature change.**

## ⛔ A trap already avoided — `is_atomizable` is the WRONG check-time rule

The tempting "static layer" is `is_atomizable` (`src/check.rs:1586`). It answers *"can this become
a holon"*, NOT *"can this be hashed"*: the runtime predicate's own doc says structurally-hashable
non-atomizable values (`u8`, `Tuple`, `Option`) ARE hashable. Keying `Lru`'s `K` on
`is_atomizable` would refuse legal programs — a rule that outlaws a truth.

**No type-level hashability predicate exists anywhere in the tree** — measured: `HashMap` keyed by
an `Lru` handle is `check=0 run=1` today, refused only at runtime. So a check-time layer is not
"match the siblings", it is a NEW predicate over all three hashed containers. **Out of this stone.**

## Out of scope — REJECTED

- A type-level hashability predicate (all hashed containers at once; its own stone).
- Returning a `Result` from `put`/`get` (see the contract decision).
- F-083 (a re-put empties a `HolographicLru`) — a different defect; its board row must still pin it.
