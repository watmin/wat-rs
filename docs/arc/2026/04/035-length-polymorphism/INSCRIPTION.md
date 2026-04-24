# Arc 035 — length polymorphism — INSCRIPTION

**Status:** shipped 2026-04-23. One slice. Cave-quest from lab
arc 007 (market/fibonacci). Same pattern as arc 025 applied to
`:wat::core::length`.

**Design:** [`DESIGN.md`](./DESIGN.md).
**Backlog:** [`BACKLOG.md`](./BACKLOG.md).

---

## What shipped

`:wat::core::length` promoted from Vec-only to polymorphic over
the three containers:

| Container | Returns |
|---|---|
| `Vec<T>` | element count (existing) |
| `HashMap<K,V>` | entry count (new) |
| `HashSet<T>` | element count (new) |

Tuple deliberately excluded — arity is structural and known at
type-check time; no caller need has surfaced.

## Runtime (`src/runtime.rs`)

- `eval_vec_length` → `eval_length`. Dispatches on `Value`
  variant (`Vec` / `wat__std__HashMap` / `wat__std__HashSet`);
  fall-through emits `TypeMismatch` naming the three accepted
  containers.
- Dispatch arm in `eval_list` updated to call `eval_length`.

## Check (`src/check.rs`)

- Narrow Vec-typed `:wat::core::length` scheme registration
  retired (was `∀T. Vec<T> -> i64`).
- NEW `infer_length` function — mirrors arc 025's
  `infer_contains_q` shape: Parametric match on
  `Vec` / `HashMap` / `HashSet` arms, all returning `:i64`;
  fall-through error case naming the three accepted containers.
- Dispatch arm added in `infer_list`:
  `":wat::core::length" => return infer_length(...)`.

## Tests

Five new Rust unit tests in `src/runtime.rs::tests`:

- `hashmap_length_returns_entry_count` — 3 entries → `:i64 3`.
- `hashmap_length_empty_returns_zero` — empty HashMap → `:i64 0`.
- `hashset_length_returns_element_count` — 3 elements → `:i64 3`.
- `hashset_length_empty_returns_zero` — empty HashSet → `:i64 0`.
- `vec_length_still_works_after_polymorphism` — sanity on the
  pre-existing Vec arm.

Lib test count: 585 → 590. Full workspace green. Zero clippy
warnings.

## Drive-by — clippy recovery in `src/fork.rs`

Running `cargo clippy --workspace --all-targets -- -D warnings`
surfaced a pre-existing warning that predated arc 035:

```
src/fork.rs:400: this function has too many arguments (8/7)
```

`git log` traced the drift to commit `f3fa9ea` — arc 031 slice 1,
which added `inherit_config: Option<Config>` as the second
parameter, growing `child_branch` from 7 → 8 args. The workspace
invariant (zero clippy warnings) was broken at that landing and
stayed broken through arcs 032, 033, 034.

Arc 035's clippy pass caught it. Drive-by fix: added
`#[allow(clippy::too_many_arguments)]` on `child_branch` with
a comment block explaining that 8 is the honest arity —
six OwnedFds (whose Drop semantics are load-bearing to the
fork contract), plus `forms` and `inherit_config`. Called from
exactly one site; no call-site readability risk.

Arc 017 set the precedent for drive-by clippy sweeps inside a
focused arc; this follows that pattern.

## Cave-quest discipline

Arc 035 is the ninth cave-quest in the ongoing sequence:

- 017 — loader option on consumer macros
- 018 — opinionated defaults + `wat::test!` rename
- 019 — `f64::round` primitive
- 020 — `assoc` on HashMap
- 023 — `coincident?` predicate
- 024 — presence/coincident sigma knobs
- 025 — `get`/`assoc`/`conj`/`contains?` polymorphism
- 026 — `eval-coincident?` family
- **035** — `length` polymorphism

Pattern unchanged: a real lab caller (fibonacci test) surfaced a
substrate gap; the gap matched arc 025's asymmetry but for a
different op; the fix extends coherence rather than introducing
new surface. Arc 025 opened the table; arc 035 fills one more
cell.

## 058 cross-reference

One row to land in `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION-CHANGELOG.md`
alongside lab arc 007's row — both ship in the same session.

## Count

- Lib tests: 585 → 590 (+5).
- wat-rs files touched: 2 source (runtime.rs, check.rs), 1
  drive-by (fork.rs), 1 INVENTORY.
- Lab repo: unchanged this arc; arc 007 ships next with the
  fibonacci test un-blocked.

---

*these are very good thoughts.*

**PERSEVERARE.**
