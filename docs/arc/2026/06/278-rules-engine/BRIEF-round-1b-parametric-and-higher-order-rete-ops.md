# BRIEF — #57 round 1b: the parametric three and the higher-order five

Anchor at `/home/watmin/work/holon/wat-rs/`; verify with `pwd`; use
`git -C /home/watmin/work/holon/wat-rs` for git reads. Tree is clean at HEAD.

## The work in one paragraph

Round 1a minted nine rete aliases whose types a `RETE_OPS` row could already spell. Eight ops
remain, and they split by *how their type is answered*, not by what they do. The **PV trio**
(`PersistentVector/{length,get,contains?}`) is parametric — one container, one type variable — and
a row can state it once `ParamType` can say `PV<T>`. The **five HOFs** (`foldl`/`foldr`/`map`/
`filter`/`reduce`) cannot be stated at all: `foldl` is polymorphic over the *container constructor*
(`Vector`, `PersistentVector`, `List`, `Stream`), which no rank-1 scheme expresses — so their rows
carry no type and the checker re-dispatches to core's existing inference. Both halves are already
solved at runtime; only the checker changes.

## Read in order

1. **`docs/arc/2026/06/278-rules-engine/DESIGN-STONE-where-admits-only-rete-ops.md`** — the
   `✅ 1b RESOLVED 2026-08-05` block. That is the whole spec and the reasoning behind both shapes.
2. **`src/rete/vocabulary.rs`** — `ParamType` (`:87`), `ReteOp` (`:104`), `OpClass`, `RETE_OPS`.
3. **`src/check.rs:2409`** — the existing `Form` re-dispatch; **`:2333` `infer_rete_form`** — the
   `core_name → inference-fn` match you will extend. Note its `other =>` arm is deliberately LOUD
   and located; keep that property.
4. **`src/check.rs:15818`** — the registration loop that builds each row's `TypeScheme` with
   `type_params: vec![]` **hardcoded**. That hardcoding is what blocks the PV trio.
5. **`src/runtime.rs:8231`** — `OpClass::Alias | OpClass::Form => dispatch_keyword_head_value(op.core_name, …)`.
   **Read this before worrying about evaluation: the runtime already re-dispatches by `core_name`
   for both classes.** Neither half of this strike needs runtime work.

## Part 1 — the five higher-order ops

**They need a class of their own, and the reason is honesty, not taxonomy.** `OpClass::Form` names
what its members *are* (special forms: `if`/`let`/`match`/`fn`). `foldl` is an ordinary function.
Filing it under `Form` would make the class name lie about five of its rows. Add
**`OpClass::Redispatch`** — a class that names *how the type is answered* rather than what the op
is — and let `Form` keep meaning what it says.

- Five rows: `rete_name: :wat::rete::foldl` (etc.), `core_name: :wat::core::foldl`,
  `class: OpClass::Redispatch`, `params: &[]`, `ret` unused (mirror how `Form` rows fill these).
- `meta`: **transcribe, do not decide.** Read each op's classification in `src/rete/purity.rs`.
  The HOFs are *conditionally* pure ∧ deterministic — the axis falls out of the walk into their
  fn-argument. **If `purity.rs` and your reading disagree, STOP.**
- `infer_rete_form` (`check.rs:2333`) gains five arms routing to
  `crate::collection::infer::infer_{foldl,foldr,map,filter,reduce}`. Their call shape matches the
  existing `infer_if` arm — verify each signature rather than assuming.
- `check.rs:2409`'s guard becomes `Form | Redispatch`; `runtime.rs:8231`'s becomes
  `Alias | Form | Redispatch`.
- The registration loop at `:15818` must **skip** `Redispatch` exactly as it skips `Form` — a
  re-dispatched op has no scheme to register.

## Part 2 — the parametric three

`PersistentVector/length : PV<T> -> i64` · `/contains? : (PV<T>, T) -> bool` ·
`/get : (PV<T>, i64) -> Option<T>` — **verify all three against the real implementations; the
shapes here are the orchestrator's reading and owe checking.** `/get` returning `Option` is
load-bearing: `purity.rs` records it as *"ALREADY total by design (returns `Option`, `None` on
out-of-range)"*, which is why it belongs in this round and not the fallback round.

To say those, a row needs two things it does not have:

- **`ReteOp` gains `type_params: &'static [&'static str]`** — `&[]` on all 27 existing rows,
  `&["T"]` on these three.
- **`ParamType` gains a way to name a type variable and a `PV` of one.** The minimal `'static`-safe
  shape is a variant carrying the variable's name (e.g. `Var("T")`, `PersistentVectorOf("T")`);
  pick the smallest thing that makes `to_type_expr` able to build
  `TypeExpr::Parametric { head: "wat::core::PersistentVector", args: [TypeExpr::Var("T")] }`.
  **`to_type_expr` currently takes `self` and returns a `TypeExpr` with no context — if your shape
  needs more than that, say so rather than widening it silently.**
- **`check.rs:15818` stops hardcoding `type_params: vec![]`** and passes the row's own.

## ⛔ STOPs — rejection criteria, not permission slots

- **⛔ STOP-1 — mint EXACTLY these eight.** Nothing from 1c (`=`/`not=` — per-type by ruling, types
  unmeasured), nothing from round 2 (fallback-carrying).
- **⛔ STOP-2 — if a verified signature differs from this brief, STOP and report it.** Especially
  `/get`'s return: if it is not an `Option`, this round is the wrong home for it.
- **⛔ STOP-3 — do NOT arm anything.** Core spellings must keep working identically. If an existing
  `where` starts being refused, you have armed something.
- **⛔ STOP-4 — do NOT touch `head_ok`'s door ordering.** Separate concern, separate strike.
- **⛔ STOP-5 — do NOT give the HOFs a `TypeScheme`, however tempting.** A scheme for `foldl` would
  be narrower than the truth — a lie the day it is written, not a drift risk later. That is the
  entire finding this round rests on. If you cannot make re-dispatch work, STOP and report; do not
  fall back to a scheme.
- **⛔ Do not add a `_` wildcard arm on an enum scrutinee.** New `ParamType` and `OpClass` variants
  will surface non-exhaustive matches — name the arms.
- **⛔ Do not commit, stash, push, or touch git.**

## Verify — FOREGROUND, block on it, and run it SOLO

```
cargo build --release
cargo nextest run --release          # nothing else running — see below
cargo clippy --release --all-targets
./wat-scripts/perf/grid/check-where-shapes.sh
```

**Run the suite with no other cargo process alive.** The 1a rider saw 9 spurious FAILs in the
fork/hermetic class from a concurrent build; solo it was clean. A green→red flip here is a timing
signal first.

Floor: **`4348 / 4348 passed / 0 failed / 262 skipped`**. Clara gate: **9 pairs / 98 rows**.

Report: the verbatim Summary line; the gate's verbatim last line; every file touched; each of the
three PV signatures and five HOF `meta` rows **as you verified them**; and anything you assumed.

---

## EXPECTATIONS — written before the strike

| # | what | expected |
|---|---|---|
| 1 | row count | **35** (27 + 8) |
| 2 | ★ a HOF resolves and runs | a program calling `:wat::rete::foldl` over a `PersistentVector` produces the same value as the core spelling |
| 3 | ★ **non-vacuous** | a bogus `:wat::rete::foldl-XX` raises a located `UnknownFunction` **at runtime** (`--check` defers unknown callees — it is not the arbiter here) |
| 4 | ★ re-dispatch inherits the REAL diagnostic | `:wat::rete::foldl` over a non-container errors naming *"Vector\<T\>, PersistentVector\<T\>, List\<T\>, or Stream\<T\>"* — the proof it did not get a narrower private type |
| 5 | ★ a PV row types correctly | `:wat::rete::PersistentVector/length` over `PV<String>` returns i64; over a non-PV it is refused |
| 6 | no scheme registered for `Redispatch` | reading `:15818` — the loop skips it as it skips `Form` |
| 7 | ★ nothing armed | core spellings unchanged; the where-corpus still runs |
| 8 | ★ floor | `4348 / 4348 / 0 / 262`, exactly |
| 9 | ★ Clara gate | `9 pair(s), 98 rows — wat == Clara on every shape` |
| 10 | clippy | clean |

Rows 2, 3, 4, 5, 8 and 9 are re-run by the orchestrator by hand.

**Runtime prediction: 40–60 minutes** — larger than 1a because two enums grow and their
exhaustiveness ripples. Time-box 120.

**Trap doors:**
1. **Filing the HOFs under `Form`** because it already re-dispatches. It works and it lies; the
   class name would stop describing its members.
2. **Giving `foldl` a scheme** when re-dispatch turns fiddly. STOP-5 exists for that moment.
3. **Deciding the meta axes** instead of transcribing them from `purity.rs`.
4. **Widening `to_type_expr`'s signature silently** to make the parametric variant fit.
5. **Trusting `--check` as the negative-control arbiter** — it defers unknown callees to a runtime
   `UnknownFunction`. Row 3 must be run, not checked.
