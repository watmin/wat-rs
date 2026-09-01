# BRIEF — STONES 4c + 4b: two confined vocabularies get their homes

Move 18 items out of `src/runtime.rs` into two new files: `src/freeze/stop.rs` (8) and
`src/process/died.rs` (10). DESIGN:
`docs/arc/2026/04/109-kill-std/DESIGN-STONE-the-died-error-cluster-decomposes.md`.

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. The orchestrator builds, floors
and clippies centrally — you do not run `cargo build`/`test`/`nextest`/`clippy` or `scripts/floor.sh`.
**You may not spawn sub-agents.** Work only in `/home/john/work/holon/wat-rs`; verify with `pwd`
first. Do not commit, push, stash, revert, or `git checkout --` anything. Tree clean, floor green at
5114, HEAD `0897658b1`.

## Read in order

1. The DESIGN — its § "The decomposition" and § "⬜ 4d", which says what must NOT come with you.
2. **`src/kernel/outcome.rs`** — same shape, two stones ago. Module header and `use` block standard.
3. `src/freeze.rs` (the `stop_failure*` call sites, ~195–213 and 1539) and `src/process/verbs.rs`
   (the `process_died_error_*` call sites) — the two homes' own callers.

## The work

### 1 — `src/freeze/stop.rs` (8 items)

```
STOP_FAILURES_PTR   stop_failure_value    stop_failure_from_panic   stop_failure_names
stop_failed_names   stop_failed_value     publish_stop_failures     take_stop_failures
```

★ **`STOP_FAILURES_PTR` is a member, not a dependency.** It is the static that
`publish_stop_failures` and `take_stop_failures` swap; nothing else in the tree touches it. The
state moves with the two functions that own it.

`src/freeze/` exists (`env.rs`, `validator.rs`) but has no `mod.rs` — it is declared from
`src/freeze.rs`. Add the `pub(crate) mod stop;` declaration wherever `env`/`validator` are declared,
matching that file's existing form.

### 2 — `src/process/died.rs` (10 items)

```
process_died_error_bad_return        process_died_error_bad_return_value
process_died_error_main_signature    process_died_error_main_signature_value
process_died_error_panic             process_died_error_panic_value
process_died_error_runtime           process_died_error_runtime_value
conj_died_chain                      conj_died_chain_value
```

★ **`conj_died_chain` and `conj_died_chain_value` are a pair** — the first has exactly one caller in
the tree, which is the second. They move together.

⚠ `conj_died_chain_value`'s doc comment says it exists for **`src/fork.rs`**. That file does not
exist; its real callers are `src/process/verbs.rs:125` and `:147`. Correct the comment as you move
it — this is why the pair belongs in `src/process/`.

Declare `pub(crate) mod died;` in `src/process/mod.rs`.

Bodies verbatim in both files. Every moved item becomes `pub(crate)`. Module headers in
`src/kernel/outcome.rs`'s register: what the vocabulary is, and **the measured fact that earns the
home** — for 4c, `freeze.rs` and `distribution/mod.rs` are its only callers; for 4b,
`process/verbs.rs` and `distribution/mod.rs`.

### 3 — one stays-side visibility bump

`failure_value_from_assertion_payload` (`runtime.rs:9861`) is private today, stays in `runtime.rs`,
and `process_died_error_panic` calls it. **Bump it to `pub(crate)`.** This is work, not a
contingency. It is the only one; the scan that produced it found no other, and found no imports
orphaned by the departures either.

### 4 — re-point and retire

`src/freeze.rs`, `src/process/verbs.rs`, `src/distribution/mod.rs` call these through
`crate::runtime::`. Re-point to the new modules. Leave a short retirement comment at each cut in
`runtime.rs`, in the shape stones A and B used.

## Blast radius

`src/freeze/stop.rs` + `src/process/died.rs` (new) · the module declarations · `src/runtime.rs`
(18 items out, 1 visibility bump) · `src/freeze.rs` · `src/process/verbs.rs` ·
`src/distribution/mod.rs` · whatever the compiler names. No `.wat` corpus change. No registrations.
**No verb changes behaviour.**

## STOP triggers — each REJECTS; ship nothing further on that point and report

**⛔ STOP-1 — THE 4d RESIDUE DOES NOT MOVE.** `fault_value` · `fault_names` · `fault_with_cause` ·
`fault_from_runtime_error` · `fault_from_panic_payload` · `failure_names` · `location_names` ·
`failure_value_from_assertion_payload` · `check_failed_cause` · `frame_names` ·
`format_panic_payload` · `value_from_frame_info` stay in `runtime.rs`. They are the substrate's
shared `Fault`/`Failure` diagnostic language — eight consuming homes — and their home is
**deliberately** unassigned. They will look like they belong with these two vocabularies; they are
the reason the recast refused to assign this cluster at all.
`grep -c "fn fault_value\|fn fault_with_cause\|fn check_failed_cause\|fn failure_names" src/runtime.rs`
must be **4**.

**⛔ STOP-2 — `thread_died_error_*` and `loci_died_*` ARE NOT YOURS.** They are 4a's, going to
`src/kernel/error.rs` in a later stone. They sit near your items and share the `died_error` name.
Leave them.

**⛔ STOP-3 — THE INTRUDER FENCE.** `no_field_names` and `builtin_enum_variant_names` stay —
10 and 8 consuming homes. `grep -c "fn no_field_names\|fn builtin_enum_variant_names" src/runtime.rs`
must be **2**.

**⛔ STOP-4 — IMPORT FROM THE CANONICAL HOME, NEVER THROUGH `runtime`'s FACADE.** `runtime.rs`
re-exports 22 `crate::value` names, so `use crate::runtime::Value` compiles and is a lie. Import
`Value`/`EnumValue` from `crate::value::`, spans from `crate::span`.

**STOP-5 — verbatim.** No signature tidying, no merging the `X`/`X_value` pairs that look redundant
— each pair's `_value` sibling exists so a cross-module call site reads naturally, and that is a
documented decision. Report any visibility change beyond § 3's one, with the compiler's reason.

**STOP-6 — run the orphaned-doc-block scan** over both cut regions; scan for contiguous plain `//`
too, not only `///`.

## Report

Per-file diff summary; both module headers verbatim; STOP-1's and STOP-3's grep counts; the corrected
`conj_died_chain_value` comment; every visibility change beyond § 3's one; each new module's `use`
block; before/after `wc -l src/runtime.rs`; the doc-block scan result. Then: **what surprised you** —
an item whose body did not belong with its module's siblings, a caller the DESIGN did not name, or a
dependency that turned out to be shared with the 4d residue.
