# DESIGN — STONE: `src/declare/` — the load-time declaration pass leaves the megafile

> **Builder, 2026-09-01:** *"let's keep it rolling — we've been preparing for the attack that slays
> the megafiles for months now."*
>
> `src/numeric/` took 892 lines. **This one takes 3,707** — the single largest module `partire`
> found in `runtime.rs`, and the cleanest seam of the ten.

## What moves, and why it is the right next cut

`partire`'s `declarations` module: `src/runtime.rs:789–4496`. **44 functions, 3,707 lines.** Its
reason to change, in the cast's words: *"the syntax/semantics of top-level declaration forms
(`defn`, `defstruct`, `defenum`, `defalias`, `extend`, `declare-acronyms`) — a load-time pre-pass
that populates the `SymbolTable` BEFORE any expression runs."*

★★ **Its independence claim is the strongest of the ten, and I verified every number:**

```
eval_inner calls in 3,707 lines ........ 3     (partire said 3)
dispatch_keyword_head calls ............ 0     (partire said never)
apply_function calls ................... 0
external call sites .................... 51 across 12 files
```

Three evaluator touches in 3,707 lines. **This is a load-time pass that barely knows the evaluator
exists** — which is exactly what makes it liftable, and why it goes before the modules that are
elbow-deep in `eval`.

⚠ And its blast radius is **smaller** than the 695-line `kernel_signal` module (51 sites / 12 files
against 73 / 12). Size and risk are not the same axis here.

## THE ONE CONTRACT DECISION — pinned

**`src/declare/` splits by PHASE, because that is what its reason-to-change already is.** The
existing precedent is `src/collection/` (concern files) and `src/numeric/` (concern files); this home
inherits the shape with its own axis:

```
src/declare/register.rs      13 fns  1,916 lines   populate the SymbolTable
src/declare/parse.rs         15 fns    918 lines   read a declaration form's shape
src/declare/preregister.rs    6 fns    526 lines   the earlier pass — stubs before bodies
src/declare/typevar.rs        4 fns     97 lines   free/bound type-variable walking
src/declare/mod.rs            + the remainder      metadata door, delegate-body builder
```

★ Phase is the honest axis: `preregister_*` runs before `register_*`, `parse_*` serves both, and
`typevar` is a helper family neither owns. A split by FORM (`defn.rs`, `defstruct.rs`, `defenum.rs`)
would multiply with every new declaration form — the same trap the numeric stone rejected.

## ⛔ THE SPAN TRAP, NAMED FOR THE THIRD TIME

`partire` ranges are LINE CLAIMS, and three separate casts have now put something in a range that
does not belong to the concern:

```
check.rs cast     is_atomizable        swept into `restricted_call` by span    (caught, corrected)
runtime.rs cast   dispatch_rete_op     sits inside `numeric_tower`'s range     (partire flagged it)
this stone        eval_tail            line 4497 — ONE PAST the range end
```

⚠ **The third was MY instrument, not partire's.** My bucketing loop was inclusive-off-by-one and
pulled `eval_tail` — the evaluator's own tail-call spine — into the "other" bucket. `partire`'s
`789–4496` is correct and excludes it.

**Therefore: move by the FUNCTION LIST, never by line span, and audit what a range CONTAINS rather
than where it ends.** `[[feedback_a_census_without_attribution_is_not_a_census]]`

## ★ THE PREDICTION — falsifiable

```
runtime.rs         33,260  ->  ~29,700   (-3,560 net of retirement comments)
src/declare/       ~3,700 lines, 5 files, split by PHASE — no per-FORM file
51 call sites      crate::runtime::X -> crate::declare::<phase>::X, across 12 files
eval_tail          UNTOUCHED, still in runtime.rs
behaviour          every declaration form registers identically
```

⚠ **This is the first cut that crosses 30,000.** `runtime.rs` has been above it since before the
campaign began.

## Out of scope = REJECTED (not deferred)

- **The other six evaluator modules** (`kernel_signal` · `defclause_dispatch` · `quasiquote` ·
  `reflection` · `pattern_matching` · `stepper` · `peer_protocol`). Named in
  `[[NOTE-partire-on-the-two-megafiles-runtime-and-check]]`, one stone each.
- **`register_defclause` / `preregister_stdlib_defclause_stub`.** `partire` named these a
  **practitioner's-call**: they can group by lifecycle (here, with `declarations`) or by feature
  (with `defclause_dispatch`). They ship HERE, where they already sit and where their neighbours are
  — and that choice is recorded, not silent.
- **The facade re-point sweep.** `crate::runtime::X → crate::value::X` for the 22 re-exported names.
  Cheap, independent, dissolves most remaining cycles — and NOT this stone.
- **`src/value/numeric_order.rs`**, and numeric stone 2's lattice. Untouched.

## THE FOUR QUESTIONS — flat YES/NO

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **`src/declare/`, split by PHASE** | YES | YES | YES | YES | ✅ **ADMITTED** |
| split by declaration FORM (`defn.rs`, `defstruct.rs`, …) | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |
| one flat `src/declare.rs` of 3,707 lines | YES | **NO** | YES | — | ⛔ **DISQUALIFIED** |
| take `kernel_signal` first because it is smaller | **NO** | YES | YES | — | ⛔ **DISQUALIFIED** |
| leave it; take the facade sweep instead | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |

- **split-by-FORM Honest? NO** — it grows one file per declaration form, and this substrate mints
  forms regularly (`defalias` and `declare-acronyms` are both recent). Same defect the numeric stone
  rejected: a layout that looks prepared while multiplying the surface.
- **one-flat-file Simple? NO** — it relocates the megafile problem rather than solving it; a
  3,707-line module is what we are attacking.
- **`kernel_signal`-first Obvious? NO** — it is smaller in LINES and LARGER in blast radius
  (73 sites vs 51), and it needs a new home name that neither `src/kernel/` (the peer home) nor
  `src/process/` (child-process primitives) can supply. Smaller is not safer here, and the reasoning
  is not obvious from the line count alone.
- **facade-sweep-instead Honest? NO** — it is genuinely cheaper and it moves ZERO lines out of the
  megafile. It is the right stone for the CRATE campaign, not for this one, and swapping them would
  be answering a different question than the builder asked.

## Acceptance

| what | command | expected |
|---|---|---|
| ★ the megafile crosses 30,000 | `wc -l src/runtime.rs` | ~29,700, from 33,260 |
| ★ split by phase, not form | `ls src/declare/` | register · parse · preregister · typevar · mod |
| ★ `eval_tail` did not move | `grep -c "fn eval_tail" src/runtime.rs` | **1** — still home |
| the impl does not know its edge | `grep -c "crate::intrinsic" src/declare/*.rs` | 0 |
| ★ no facade imports | each new file's `use` block | `crate::value::` direct, never `crate::runtime::` for a re-exported name |
| the 51 sites re-point | `crate::runtime::` for moved fns, outside `src/declare/` | 0 |
| behaviour unchanged | every declaration form | registers identically |
| floor | `scripts/floor.sh`, exit read UNPIPED | 5114/5114, 0 failed |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | 0 |
