# DESIGN — STONE C: `Lru/put` returns a Result, `Lru/get` misses — no panic

**Redrawn 2026-09-23.** Builder's ruling: *"i want the least amount of panics possible (which may
be zero…)"* — and, on the first draft: *"why is using a boxed result via an enum not the path
forward for dealing with a panic?"* It is. The first draft is superseded; see "What went wrong"
at the bottom.

## The defect, driven — reachable from well-typed programs

```
an Lru handle as the key, direct            check=0  run=2  Rust panic
the same, laundered through a generic K      check=0  run=2  Rust panic
```
`src/rust_deps/cache.rs:151-170`: each verb calls `value_is_hashable(&k)` and then `panic!`s.

## The cure

- **`put`** → `Result<Option<(Value, Value)>, RawFault>`: `Err` on an unhashable key, naming the
  user-facing verb `:wat::cache::Lru/put` in the `diagnostic` field. **This is stone A's mechanism
  exactly** (`Lru/new`, `70f8e2cd5`); `#[wat_dispatch]` marshals `Result<T, E>` natively. No macro
  change.
- **`get`** → returns `None` on an unhashable key. Its return type is already `Option`, so no
  caller changes. Precedent: `HashMap`'s `contains-key?` returns `false` for an unhashable key,
  *"never inserted"* (`src/collection/eval.rs:203-207`) — a key that cannot be stored cannot be
  there, so the answer is total.

## The ONE contract decision — the Result PROPAGATES through `HolographicLru/put`

Same decision as stone A made for `HolographicLru/new`: `:wat::cache::HolographicLru/put` wraps
`Lru/put` and must return the `Result` rather than swallow it. Swallowing would re-create the
defect one level up.

⚠ **Service handlers are the exception the surface forces.** Stone A found that a `defservice`
`:init` cannot return a `Result`; its disposition, copied from `wat/query/sqlite-store.wat`, was
`Result/expect` with the verb name written into the message. If `lru-svc` / `hologram-svc`
handlers hit the same wall, do the same — and note that `Result/expect` DISCARDS the `Err`
payload (stone A's finding), so the verb name only survives if it is in the `expect` message.

## Blast radius — measured, `put` only

| file | code call sites |
|---|---|
| `wat-tests/cache/HolographicLru.wat` | 12 |
| `tests/rete/probe_arc278_cache_lru.wat` | 8 |
| `wat/cache.wat` | 7 |
| `wat-scripts/scratch-pad/255-struct-field-is-a-constant-projection.wat` | 2 |
| ⚠ `tests/lint/little_wat_findings_board__f083_holographic_lru_reput.wat` | 2 |

**31 sites.** `get`'s callers do not change. `src/remedy/retirement.rs` and
`wat-scripts/fixes/type-member-colon-to-slash.wat` NAME the verb in string tables — not call sites.

⚠ **The F-083 board fixture is a caller.** F-083 (a re-put empties a `HolographicLru`) is a
DIFFERENT defect; its row `(check 0, run 0, stdout "0")` must still pin it afterwards.

## Out of scope — REJECTED

- **Where RAISED errors say they happened.** `HashMap`'s own unhashable-key raise reports
  `:file "src/collection/eval.rs" :line 449`, and C-114 reports `wat/core.wat:66` — both F-006
  family, both real, both a separate stone. A `Result` has no location, so `put` does not wait.
- A type-level hashability predicate (`is_atomizable` is the WRONG rule — it asks "can this be a
  holon", and `u8`/`Tuple`/`Option` are hashable but not atomizable).

## What went wrong with the first draft

It made "match the sibling's RAISE" the goal instead of "no panic". The sibling, when driven,
raised with a `.rs` location — so copying it would have copied an F-006 defect. It also quoted
"56 call sites" for a `Result`, a number that counted `get`'s callers (which do not change) and
two string tables; the real `put` count is 31, and stone A had already recorded a codemod for
exactly this wrap. The executor stopped at STOP-1/STOP-2 correctly; the brief was wrong.
