# BRIEF — one naming rule for the rete surface, then `first` / `nth` / `to-string`

**Ruled 2026-08-05 by the builder**, twice:

> *"i assume we need first attached to each type like `=`, `not=`, `>`, `>=`, `<`, `<=` etc is
> attached to i64, f64?"* — yes, per-container.
>
> *"109 will cover a massive naming and constructor normalization…. just follow what core data and
> slap `::rete::` on it imo."*

So: **the rete name is the core name with `rete::` inserted after `wat::`.** One rule, no exceptions.
This is NOT the big normalization — arc 109 owns that. This is making the rete table consistent with
its own naming so the admission test can mean something.

Anchor `/home/watmin/work/holon/wat-rs/`; verify with `pwd`. Tree clean at HEAD `055389af`.
Floor **`4352 / 4352 / 0 / 262`**, clippy clean, `check-where-shapes.sh` → `9 pair(s), 98 rows`,
57 rows.

---

## Why — the module set was never the boundary it claims to be

`RETE_MODULES` is documented as *"the real boundary: the declared vocabulary SUB-namespaces"*, and
`rete_vocabulary_admitted` is `head.starts_with(module)` over it. **Measured 2026-08-05: 17 of 57
minted rows fail it** — every `String/*`, every `PersistentVector/*`, the five bare HOFs, and
`bool::`/`keyword::` (those four minted this morning).

They work today only by accident: `head_ok` early-returns into the rete branch *only if* admission
passes, so these fall through to `intrinsic_meta` — whose first action is `rete_op_for`, which finds
them anyway. **Arm the admission test as the third conjunct and 17 of our own rows are refused.**

The root is that the table carries **three naming rules at once**:

| core_name | rete_name today | rule |
|---|---|---|
| `:wat::core::and` | `:wat::rete::core::and` | insert `rete::` (keeps `core`) |
| `:wat::core::i64::>` | `:wat::rete::i64::>` | replace `core` with `rete` |
| `:wat::core::PersistentVector/get` | `:wat::rete::PersistentVector/get` | replace |

A hand-maintained module list standing in for a rule, already drifted from what it guards — and it
silently failed to grow four times.

**Under the insert rule the list becomes closed:** two entries, and every core verb is covered by
construction. No new container, scalar type, or module ever needs a `RETE_MODULES` edit again.

## Why NOW

**46 of 57 rows need the rename, and almost nothing outside the table references them yet** — because
**S6 has not run.** The migration that rewrites corpus `where` sites into rete spellings is still
ahead. Do this after S6 and every migrated site gets migrated twice.

---

# PHASE 1 — the rename (mechanical, behaviour-preserving)

**⛔ ONE RIDER, SEQUENTIAL PHASES.** Both phases edit `RETE_OPS` — one table, one file. Fanning is a
merge conflict, not a seam (the stone's own `THE ROUNDS CANNOT BE PARALLELISED`).

## The rule, stated once

```
rete_name  =  core_name.replace_first(":wat::", ":wat::rete::")
```

`:wat::core::i64::>` → `:wat::rete::core::i64::>` · `:wat::core::foldl` → `:wat::rete::core::foldl` ·
`:wat::holon::cosine` → `:wat::rete::holon::cosine` (already conforms).

**11 rows already conform** (the 7 `core::*` form mirrors + the 4 `holon::*`). **46 do not.**

⚠ **The `Form` rows' `core_name` is the mirrored core form**, and the four `Redispatch`/HOF rows and
the `bool::`/`keyword::`/`String/` equality rows point at *generic* core verbs — so **derive each new
name from that row's OWN `core_name`, never from its current `rete_name`.** Deriving from the old
name reproduces whichever of the three rules it was already using.

## `RETE_MODULES` collapses

```rust
pub(crate) const RETE_MODULES: &[&str] = &[
    ":wat::rete::core::",
    ":wat::rete::holon::",
];
```

Update its doc comment: the set is closed *because* the naming rule makes it closed — state the rule
there, since that is what a future reader needs in order not to re-grow the list.

## Sites outside the table — 21 files, measured

`src/runtime.rs` · `src/rete/compiled_rhs.rs` · `src/rete/purity.rs` · `wat/rete.wat` ·
8 × `wat-scripts/scratch-pad/probe-*.wat` · 7 × `tests/rete/*` · 2 × `tests/types/*`

**⛔ Do NOT sed the tree.** Several of these files also contain *core* spellings that must not move,
and `wat/rete.wat` is the engine's own API (`fire-rules`, `insert`, `compile`) which shares the
`:wat::rete::` prefix and **must not be touched at all**. Change only strings that name a `RETE_OPS`
row.

## Phase-1 acceptance — it is a RENAME, so the floor must not move

| | expected |
|---|---|
| row count | **57**, unchanged |
| ★ floor | **`4352 / 4352 / 0 / 262`** — exactly; a rename that moves the floor moved behaviour |
| ★ gate | `9 pair(s), 98 rows` |
| ★★ **admission is now total** | **every** row satisfies `rete_vocabulary_admitted(row.rete_name)` — assert it as a **unit test over `RETE_OPS`**, not a grep. This is the defect's permanent ward: it must be impossible for a future row to be minted outside an admitted module |
| ★ the rule is enforced, not just applied | a second unit test: for every row, `rete_name == core_name.replacen(":wat::", ":wat::rete::", 1)`. **This is the actual extirpare rung** — with it, the three-rules drift cannot recur |
| clippy | clean |

**Those two unit tests are the point of Phase 1.** The rename fixes today; the tests make the class
unrepresentable. Without them this is cosmetics.

---

# PHASE 2 — mint `first`, `nth`, and the `to-string` family

Only after Phase 1 is green. Under the insert rule these need no admission thought.

## `to-string` — three `Alias` rows

| rete_name | core_name | params → ret |
|---|---|---|
| `:wat::rete::core::i64::to-string` | `:wat::core::i64::to-string` | `[I64] -> String` |
| `:wat::rete::core::f64::to-string` | `:wat::core::f64::to-string` | `[F64] -> String` |
| `:wat::rete::core::bool::to-string` | `:wat::core::bool::to-string` | `[Bool] -> String` |

All three exist in core and are already `total` (`bool::to-string` is in the total list; confirm the
other two rather than assuming — if either is absent, that is a finding, report it, do not classify
it yourself).

There is **no generic `num-to-string`** — grounded, zero hits. Per-type is the only spelling.

## `first` — three `Fallback` rows, per container

`:wat::core::first` is **partial** — proven by run: an empty sequence raises
`MalformedForm: "sequence has 0 element(s); no element at index 0"`. So it carries `:undefined`.

| rete_name | core_name | params → ret |
|---|---|---|
| `:wat::rete::core::PersistentVector/first` | `:wat::core::first` | `[PersistentVectorOf("T"), Keyword, Var("T")] -> Var("T")` |
| `:wat::rete::core::Vector/first` | `:wat::core::first` | `[VectorOf("T"), Keyword, Var("T")] -> Var("T")` |
| `:wat::rete::core::List/first` | `:wat::core::first` | `[ListOf("T"), Keyword, Var("T")] -> Var("T")` |

`ParamType` has `PersistentVectorOf`; **`VectorOf` and `ListOf` are new variants** — same shape,
added the way round 1a added `String`/`F64`.

**⛔ AFFIRMATIVELY CUT, with reasons — record them in the table's comment, do not silently omit:**
- **`Tuple`** — heterogeneous; element-0's type depends on the tuple's shape, so it cannot be spelled
  `C<T> -> T` at all.
- **`WatAstList`** — a `Value::wat__WatAST` wrapping an AST node. R17 recorded this exact member
  breaking a container abstraction; it is not a homogeneous sequence.
- **`Stream`** — laziness in a rule condition is a ruling nobody has made. Not a row until it is.
- **`HashSet`** — `indexable()` is already `false`; no first element by nature.

**⚠ Per-type here is NOT the comparators' reason.** For `i64::>` the per-type form *deletes* the
domain hole. For `first` it deletes nothing — an empty `PersistentVector` still has no first element.
Per-type is what makes the row **schemable**, which is what makes `Fallback` available. Say this in
the comment; otherwise the next reader infers per-type ⇒ total.

## `nth` — one `Fallback` row

`:wat::core::nth<T>` is a **wat-level `defn`** (`wat/core.wat:1349`), `Vector<T> × i64 -> T`, defined
as `Option/expect (get v i)`.

**⚠ Its own header says `TOTAL` and its body raises** — *"the positional, TOTAL accessor … RAISING on
out-of-range."* That header is wrong and is the kind of lie that propagates: a reader trusting it
would mint an `Alias`. **Fix the comment in this stone** (say it returns `T`, raising on
out-of-range — the *contract* is total-looking, the *function* is partial; `get` is the total one).

| rete_name | core_name | params → ret |
|---|---|---|
| `:wat::rete::core::nth` | `:wat::core::nth` | `[VectorOf("T"), I64, Keyword, Var("T")] -> Var("T")` |

⚠ **Grounding owed before you write it:** `nth` is `defn`-defined in wat, not a Rust intrinsic — every
existing `Fallback` row surfaces a Rust-dispatched core op. **Verify `dispatch_rete_op`'s
`dispatch_keyword_head_value` reaches a wat-level `defn`, and that the raise it produces is
catchable by the arm's `Err` path.** If it is not, STOP and report — that is a real mechanism gap and
the orchestrator owns the re-scope. Do not invent a path.

## Phase-2 acceptance

| | expected |
|---|---|
| row count | **64** (57 + 3 + 3 + 1) |
| ★★ `first` fallback FIRES | `(… PersistentVector/first <empty> :undefined 0)` → `0`, not a raise |
| ★★ **non-vacuity** | same empty expression, `:undefined 0` then `:undefined 99` → `0` then `99` |
| ★ `first` happy path | a non-empty PV returns its element, fallback not taken |
| ★ all three containers | Vector and List rows work, not just PersistentVector |
| ★ `nth` fallback FIRES | out-of-range index → the caller's value |
| ★ `to-string` | `(… i64::to-string 42)` → `"42"` |
| ★ admission still total | the Phase-1 unit test still passes with the new rows |
| ★ floor | ≥ `4352`, nothing lost |
| ★ gate | `9 pair(s), 98 rows` |

## ⛔ STOPs

- **⛔ Phase 1 must be green before Phase 2 starts.** A rename tangled with new capability is two
  review problems in one diff.
- **⛔ Do not touch `wat/rete.wat`'s engine API** (`fire-rules`/`insert`/`compile`/`Session`) — it
  shares the prefix and is not vocabulary.
- **⛔ Do not sed.** Change only strings naming a `RETE_OPS` row.
- **⛔ Do not mint Tuple/WatAstList/Stream/HashSet rows.**
- **⛔ Do not classify a core verb yourself.** If `i64::to-string` or `f64::to-string` is missing from
  the total list, report it.
- **⛔** No `_` wildcard arm on an enum scrutinee.
- **⛔** Do not commit, stash, push, or touch git.

## Verify — FOREGROUND, block, SOLO, after each phase

```
cargo build --release
cargo nextest run --release
cargo clippy --release --all-targets
./wat-scripts/perf/grid/check-where-shapes.sh
```

Read the **Summary line**, never a piped exit code.

**Trap doors:**
1. **Deriving the new name from the old `rete_name`.** Derive from `core_name`. The old names use
   three different rules.
2. **sed-ing the tree.** Several files carry core spellings that must not move.
3. **Shipping the rename without the two unit tests.** Then it is cosmetics and the drift recurs.
4. **Assuming `nth` dispatches like the others.** It is wat-level. Verify or STOP.
5. **Skipping the non-vacuity row.** A `first` fallback returning a constant passes every other row.
