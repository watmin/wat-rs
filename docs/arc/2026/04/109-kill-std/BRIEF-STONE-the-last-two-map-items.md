# BRIEF — STONE: the map's last two items

Move 11 items out of `src/runtime.rs` into four destinations. DESIGN:
`docs/arc/2026/04/109-kill-std/DESIGN-STONE-the-last-two-map-items.md`.

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. The orchestrator builds, floors
and clippies centrally — you do not run `cargo build`/`test`/`nextest`/`clippy` or `scripts/floor.sh`.
**You may not spawn sub-agents.** Work only in `/home/john/work/holon/wat-rs`; verify with `pwd`
first. Do not commit, push, stash, revert, or `git checkout --` anything. Tree clean, floor green at
5114, HEAD `438c405e0`.

## Read in order

1. The DESIGN — its contract decision (directories, not files) and § on the `expect` pair.
2. **`src/kernel/error.rs`** — shipped one stone ago; module header and `use` block standard.
3. **`src/assertion.rs`** — it owns `AssertionPayload` and `eval_kernel_assertion_failed`; two of
   your items join it. Read it before you place them.
4. `src/intrinsic/option.rs` and `src/intrinsic/result.rs` — the two edges. Their headers name the
   functions they delegate to and become false when you move them.

## The work

### 1 — `src/option/mod.rs` (new directory, 3 items)

`eval_some_ctor` · `eval_option_expect` · `eval_option_try`

### 2 — `src/result/mod.rs` (new directory, 4 items)

`eval_ok_ctor` · `eval_err_ctor` · `eval_result_expect` · `eval_try`

⚠ **Directories with a `mod.rs`, not top-level `src/option.rs` files.** The DESIGN pins why: a
directory is the partition line the crate migration consumes; a new top-level `.rs` is a step this
campaign exists to undo. Declare both in `src/lib.rs` beside the other home modules.

### 3 — `src/assertion.rs` gains the shared `expect` machinery (2 items)

`expect_panic` · `extract_panics` — both private today, both called only by `eval_option_expect` and
`eval_result_expect`. They build and destructure `AssertionPayload`, which `src/assertion.rs` owns.
Both become `pub(crate)`; both new homes import them from `crate::assertion`.

### 4 — `src/rete/purity.rs` gains the classifier pair (2 items)

`effectful_by_prefix` · `is_effectful_op` — a two-tier classifier: `is_effectful_op` consults
`crate::intrinsic::registry()` first and falls back to `effectful_by_prefix`. They move together.
`src/rete/purity.rs` already calls `registry()`, so this adds no new dependency.

Bodies verbatim throughout. Module headers in `src/kernel/error.rs`'s register, each naming the edge
or the concept that earns the home.

### 5 — re-point, retire, and correct the prose

`src/intrinsic/{option,result}.rs` (7 call sites) · `src/intrinsic/mod.rs` (the census test's
`crate::runtime::effectful_by_prefix`) · `src/rete/purity.rs:994`. Retirement comments at each cut
in `runtime.rs`, in the shape the prior stones used.

**⛔ And correct `src/intrinsic/rete.rs:15`.** It says `:wat::rete::` is *"deliberately ABSENT from
`effectful_by_prefix`"*. The function's body contains `head.starts_with(":wat::rete::")` — the claim
was true when written (`e01428497`) and was overturned by `2bc1135aa`, whose own commit title says
so. Rewrite it to state what is true now and that W5b widened it; keep the historical fact that W5a's
nine read-only predicates needed no widening. Also fix its `(src/runtime.rs)` pointer — the function
is moving. **Cite by grep-token, never by line number.**

Check `src/intrinsic/{option,result}.rs` and `src/intrinsic/mod.rs` for other doc comments naming
these eleven as `runtime.rs`'s, and correct those too.

## Blast radius

`src/option/mod.rs` + `src/result/mod.rs` (new) · `src/lib.rs` · `src/assertion.rs` ·
`src/rete/purity.rs` · `src/runtime.rs` (11 items out) · `src/intrinsic/{option,result,mod,rete}.rs` ·
whatever the compiler names. No `.wat` corpus change. No registrations. **No verb changes behaviour.**

## STOP triggers — each REJECTS; ship nothing further on that point and report

**⛔ STOP-1 — `tests/value/probe_rational_C4_mixed_float.rs` DEFINES ITS OWN `fn eval_try`.** It is a
local test harness helper that shares the name; it is not a consumer of the intrinsic and must not be
touched. A grep for `eval_try` will find it and it will look like a call site.

**⛔ STOP-2 — THE `expect` PAIR IS NOT `result`'s.** `expect_panic` and `extract_panics` are called
by BOTH `eval_option_expect` and `eval_result_expect`. Giving them to `src/result/` because
`extract_panics` reads an error chain leaves a reader with no true answer to why Option's verb
machinery lives under Result. They go to `src/assertion.rs`, with the type they construct.

**⛔ STOP-3 — THE CLASSIFIER PAIR MOVES TOGETHER OR NOT AT ALL.** `is_effectful_op` calls
`effectful_by_prefix`. Splitting them puts one tier of a two-tier classifier in each of two homes.

**⛔ STOP-4 — THE SPINE AND THE 4d RESIDUE STAY.** `eval`/`eval_inner`/`eval_tail` and
`fault_*`/`failure_*`/`check_failed_cause`/`location_names`/`frame_names` remain in `runtime.rs`.
`grep -c "^pub(crate) fn eval_tail\|^pub(crate) fn eval_inner\|^pub fn eval\b" src/runtime.rs` = **3**;
`grep -c "fn fault_value\|fn fault_with_cause\|fn check_failed_cause\|fn failure_names" src/runtime.rs` = **4**.

**⛔ STOP-5 — IMPORT FROM THE CANONICAL HOME, NEVER THROUGH `runtime`'s FACADE.** Import
`Value`/`Environment`/`SymbolTable`/`EvalBreak`/`RuntimeError` from `crate::value::`, spans from
`crate::span`, AST from `crate::ast`. ⚠ `src/assertion.rs:34` already carries such a facade import —
**leave that line exactly as it is**; it belongs to a separate open sweep and touching it here would
make a red unattributable between two causes.

**STOP-6 — verbatim.** No signature tidying, no merging `eval_option_expect` with
`eval_result_expect` because they now share a home for their helpers. Report any visibility change
beyond § 3's two, with the compiler's reason.

**STOP-7 — run the orphaned-doc-block scan** over every cut region; scan for contiguous plain `//`
too, not only `///`.

## Report

Per-file diff summary; the four module/section headers verbatim; **the rewritten
`src/intrinsic/rete.rs` claim**; STOP-4's two grep counts; every doc comment you corrected; each new
module's `use` block; before/after `wc -l src/runtime.rs`; the doc-block scan result. Then: **what
surprised you** — a body that did not belong with its siblings, a caller the DESIGN did not name, or
a doc claim you found false that the brief did not list.
