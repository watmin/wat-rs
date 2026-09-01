# BRIEF — STONE 4a: `src/kernel/error.rs`, the eighth module of a seven-module stone

Move 16 items out of `src/runtime.rs` into a new `src/kernel/error.rs`, the impl home for the
`src/intrinsic/kernel/error.rs` edge file. DESIGN:
`docs/arc/2026/04/109-kill-std/DESIGN-STONE-the-died-error-cluster-decomposes.md` — read its
§ "AMENDED again, before briefing 4a".

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. The orchestrator builds, floors
and clippies centrally — you do not run `cargo build`/`test`/`nextest`/`clippy` or `scripts/floor.sh`.
**You may not spawn sub-agents.** Work only in `/home/john/work/holon/wat-rs`; verify with `pwd`
first. Do not commit, push, stash, revert, or `git checkout --` anything. Tree clean, floor green at
5114, HEAD `96fd1c688`.

## Read in order

1. The DESIGN's § "The decomposition", § "⬜ 4d", and the § "AMENDED again" note.
2. **`src/kernel/outcome.rs`** and **`src/kernel/serve.rs`** — this home's own siblings, shipped in
   stones A and B. Module header and `use` block standard.
3. **`src/intrinsic/kernel/error.rs`** — the edge file this module is the impl for. Its header states
   which four `runtime.rs` functions it delegates to; that header is the membership authority, and
   it becomes false when you move them.
4. `src/kernel/mod.rs` § "Layout" — stone B rewrote it to name all eight sub-modules. `error` is the
   ninth and belongs in that list.

## The work

### 1 — `src/kernel/error.rs` (16 items)

```
the edge's four   eval_died_error_message · eval_died_error_to_failure
                  eval_failure_message · eval_failure_location
the loci family   loci_died_error_from_reason · loci_died_from_send_error · loci_died_disconnected
the thread family thread_died_error_panic · thread_died_error_runtime · thread_died_error_shutdown
the chain/EDN     single_died_chain · thread_crash_panic_edn · thread_crash_runtime_edn
private helpers   died_error_payload_message · edn_is_loci_died_chain · failure_error_field
```

Bodies verbatim. Each becomes `pub(crate)` (seven already are). Add `pub mod error;` to
`src/kernel/mod.rs` alphabetically, and add `error` to its § "Layout" list.

The module header, in `outcome.rs`'s register, must carry **the measured fact that earns the home**:
every external call site of these sixteen is either `src/intrinsic/kernel/error.rs` (the edge) or
`src/kernel/{message,outcome,spawn}.rs` (this home) — seventeen sites, none anywhere else in the
tree. ★ And note the two that prove it hardest: `thread_crash_panic_edn` and
`thread_crash_runtime_edn` have **zero callers left in `runtime.rs`** — their only consumer is
`src/kernel/spawn.rs`.

### 2 — re-point and retire

`src/intrinsic/kernel/error.rs` (4 call sites) · `src/kernel/message.rs` · `src/kernel/outcome.rs` ·
`src/kernel/spawn.rs` — re-point to `crate::kernel::error::`. Several are already local-ish imports
from `crate::runtime`; those become `crate::kernel::error`. Leave a short retirement comment at each
cut in `runtime.rs`, in the shape stones A/B/4b/4c used.

### 3 — the prose this stone falsifies

`src/intrinsic/kernel/error.rs`'s header says the four verbs *"delegate to the SAME
`crate::runtime::eval_*` fn"*. After the move they delegate to `crate::kernel::error::`. Correct it.
⚠ Stone B left eight such headers false across the other seven edge files and the orchestrator had to
sweep them afterwards; this one is yours, in-stone.

Check `src/kernel/{outcome,message,spawn}.rs` for doc comments citing these sixteen as
`runtime.rs`'s — `outcome.rs`'s header names `loci_died_error_from_reason` and
`loci_died_from_send_error` explicitly as *"genuinely defined in `crate::runtime`"*, which this stone
makes false. **Cite by grep-token, never by line number.**

## Blast radius

`src/kernel/error.rs` (new) · `src/kernel/mod.rs` · `src/runtime.rs` (16 items out) ·
`src/intrinsic/kernel/error.rs` · `src/kernel/{message,outcome,spawn}.rs` · whatever the compiler
names. No `.wat` corpus change. No registrations. **No verb changes behaviour.**

## STOP triggers — each REJECTS; ship nothing further on that point and report

**⛔ STOP-1 — THE 4d RESIDUE DOES NOT MOVE.** `fault_value` · `fault_names` · `fault_with_cause` ·
`fault_from_runtime_error` · `fault_from_panic_payload` · `failure_names` · `location_names` ·
`failure_value_from_assertion_payload` · `check_failed_cause` · `frame_names` ·
`format_panic_payload` · `value_from_frame_info` stay in `runtime.rs`. Eight consuming homes; home
deliberately unassigned. They share this stone's naming and sit near it.
`grep -c "fn fault_value\|fn fault_with_cause\|fn check_failed_cause\|fn failure_names" src/runtime.rs`
must be **4**.

**⛔ STOP-2 — `eval_error_names` AND `runtime_error_to_eval_error_value` ARE NOT YOURS.** They read
like this stone's — an `_error_` name, a `_names` helper, adjacent lines — and they are the
`:wat::core::EvalError` / `EvalResult` vocabulary that serves `intrinsic/holon/atom.rs`'s `eval-*`
verbs, sitting beside `wrap_as_eval_result` and `eval_form_ast`. The DESIGN put `eval_error_names` in
this stone and a comment-stripped caller scan took it back out. **A `_names` suffix is a naming
convention, not a membership test** — three helpers have been misplaced by it this campaign.
`grep -c "fn eval_error_names\|fn runtime_error_to_eval_error_value" src/runtime.rs` must be **2**.

**⛔ STOP-3 — THE INTRUDER FENCE.** `no_field_names` and `builtin_enum_variant_names` stay — 10 and 8
consuming homes. `grep -c "fn no_field_names\|fn builtin_enum_variant_names" src/runtime.rs` must
be **2**.

**⛔ STOP-4 — IMPORT FROM THE CANONICAL HOME, NEVER THROUGH `runtime`'s FACADE.** `runtime.rs`
re-exports 22 `crate::value` names, so `use crate::runtime::Value` compiles and is a lie. Import
`Value`/`EnumValue`/`AggregateValue`/`RuntimeError` from `crate::value::`, spans from `crate::span`.
⚠ `src/kernel/{address,listener,spawn}.rs` carry pre-existing facade imports that belong to a
separate open sweep — **leave those three lines exactly as they are.**

**STOP-5 — verbatim.** No signature tidying, no merging the three `thread_died_error_*` arms that
differ only by variant, no folding `single_died_chain` into its two callers. Report any visibility
change beyond the nine private→`pub(crate)` bumps § 1 requires, with the compiler's reason.

**STOP-6 — run the orphaned-doc-block scan** over the cut region; scan for contiguous plain `//`
too, not only `///`.

## Report

Per-file diff summary; the module header verbatim; the corrected
`src/intrinsic/kernel/error.rs` header; STOP-1/2/3's grep counts; every doc comment you corrected in
`src/kernel/*`; the new module's `use` block; before/after `wc -l src/runtime.rs`; the doc-block scan
result. Then: **what surprised you** — an item whose body did not belong with its siblings, a caller
the DESIGN did not name, or a dependency shared with the 4d residue.
