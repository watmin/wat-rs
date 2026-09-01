# BRIEF — STONE B: the seven kernel sub-modules mirror their edge

Move 34 items out of `src/runtime.rs` into seven new `src/kernel/*.rs` modules, one per edge file in
`src/intrinsic/kernel/`. DESIGN: `docs/arc/2026/04/109-kill-std/DESIGN-STONE-B-the-seven-kernel-submodules.md`.

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. The orchestrator builds, floors
and clippies centrally — you do not run `cargo build`/`test`/`nextest`/`clippy` or `scripts/floor.sh`.
**You may not spawn sub-agents.** Work only in `/home/john/work/holon/wat-rs`; verify with `pwd`
first. Do not commit, push, stash, revert, or `git checkout --` anything. Tree clean, floor green at
5114, HEAD `2f5fa10a7`.

## Read in order

1. The DESIGN — especially § "THE ONE CONTRACT DECISION" and § "The 14 reach-backs, classified".
2. **`src/kernel/outcome.rs`** — shipped one stone ago into this very home. Its module header and
   `use` block are the standard.
3. **`src/kernel/mod.rs`** — its § "Scope boundary" and the `rune:exigere(scope-affirmative)` beneath
   it. **This stone is the migration that rune names**, and § 4 below strikes it.
4. **The seven edge files** `src/intrinsic/kernel/{abort,ambient,identity,message,resource,serve,source}.rs`
   — each header states which `runtime.rs` function it delegates to and calls it "pre-existing" or
   "in place, unmoved". Those headers are the membership authority for this stone.

## The work

### 1 — seven new modules, each named for its edge file

Move by NAME. Bodies verbatim. Each item becomes `pub(crate)`. Add seven `pub mod` lines to
`src/kernel/mod.rs`, alphabetically.

```
kernel/abort.rs     eval_kernel_raise
kernel/ambient.rs   eval_kernel_stopped · eval_user_signal_query · eval_user_signal_reset
kernel/identity.rs  eval_peer_pid · eval_peer_process · eval_peer_wire · eval_address_wire
                    eval_require_wire_address
kernel/message.rs   eval_peer_send_prime · eval_peer_try_send_prime · eval_peer_recv_prime
                    eval_peer_select_prime · eval_poll_prime · wrap_connect_request
kernel/resource.rs  eval_handle_pool_new · eval_handle_pool_pop · eval_handle_pool_finish
                    eval_kernel_after · eval_peer_close_prime · eval_signal · SIGNAL_TYPE
                    eval_listener_prime · eval_connect_prime · eval_accept_prime
                    eval_allow_prime · eval_deny_prime
kernel/serve.rs     eval_retag_op · eval_kernel_serve_dispatch_op_tail
kernel/source.rs    eval_kernel_here · eval_kernel_call_site · eval_kernel_macro_call_site
                    bound_names
```

**And one item gets no new module:** `extract_panic_payload` moves into the **existing**
`src/kernel/spawn.rs`, whose line 792 is its only caller in the tree.

Each module header, in `outcome.rs`'s register: what the module IS, and **the edge file it mirrors**,
named. `SIGNAL_TYPE` and `bound_names` are the two non-`eval_*` members — say in the header why each
sits where it does (`SIGNAL_TYPE` names the `:wat::kernel::Signal` argument enum that `eval_signal`
decodes; `bound_names` serves the source-position family).

### 2 — five stays-side visibility bumps

These five stay in `src/runtime.rs`, are private today, and the new modules import them. **Bump each
to `pub(crate)`; this is work, not a contingency:**

```
eval_tail · try_match_pattern · record_field_by_name · value_from_frame_info · loci_died_disconnected
```

### 3 — re-point the edge

Each of the seven edge files calls `crate::runtime::eval_*`. Those become `crate::kernel::<module>::`.
Thirty call sites. The compiler names any you miss. Leave a short retirement comment at each cut in
`runtime.rs`, in the shape stone A used.

### 4 — the prose this stone falsifies

- **`src/record/mod.rs:31`** states *"`eval_retag_op` did **NOT** move."* This stone moves it to
  `kernel/serve`. That sentence is a stated law, written by the record stone on correct evidence at
  the time; rewrite it to say where the function went and why it was right to exclude it there.
- **`src/kernel/mod.rs` § "Scope boundary"** says the kernel verbs *"live in `src/runtime.rs`
  (registered at runtime.rs:4206-4218)"* and carries
  `rune:exigere(scope-affirmative)` deferring the homing to this campaign. **The campaign has now
  arrived: rewrite the paragraph and STRIKE the rune.** An exemption whose reason has expired is the
  defect `excusare` exists to catch.
- `src/kernel/peer.rs:265,471` · `src/kernel/listener.rs:101,237` · `src/kernel/address.rs:96,155` ·
  `src/value/frame.rs:85` · `src/types.rs:2097` · `src/check.rs:4206,9898,10868,11636` ·
  `src/rust_deps/custodia.rs:112` · `src/intrinsic/kernel/message.rs:53` ·
  `tests/comms/probe_select_flood_no_deadlock.rs:6` — doc comments citing these functions as
  `runtime.rs`'s. Correct them to the new home.

⚠ **Cite by grep-token, never by line number.** `tests/comms/probe_select_flood_no_deadlock.rs:6`
says *"(src/runtime.rs ~24755)"* — a line that stopped existing before this stone was drawn. Do not
add another.

## Blast radius

Seven new `src/kernel/*.rs` · `src/kernel/mod.rs` · `src/kernel/spawn.rs` · `src/runtime.rs` (34 items
out, 5 visibility bumps) · the seven `src/intrinsic/kernel/*.rs` edge files · the doc-comment sites
above · whatever the compiler names. No `.wat` corpus change. No registrations. **No verb changes
behaviour.**

## STOP triggers — each REJECTS; ship nothing further on that point and report

**⛔ STOP-1 — THE EVAL SPINE DOES NOT MOVE, AND IT WILL LOOK LIKE IT SHOULD.** `eval`, `eval_inner`,
`eval_tail`, `eval_walk`, `eval_match`, `eval_subtype`, `eval_conforms`, `parse_let_binding`,
`LetBinding`, `apply_value`, `try_match_pattern_ast` and the whole `step_*` family stay in
`runtime.rs`. A dependency-closure run over this stone's items reaches all of them, because every
kernel verb calls `eval_inner` and `eval_inner` reaches the whole file. **Membership comes from the
EDGE, not from what a function calls.** If an item is not in § 1's list and no
`src/intrinsic/kernel/*.rs` header names it, it is not yours.
`grep -c "^fn eval_tail\|^pub(crate) fn eval_inner\|^pub fn eval\b" src/runtime.rs` must be **3**.

**⛔ STOP-2 — THE INTRUDER FENCE.** `reset_kernel_stop`, `request_kernel_stop`, `no_field_names`,
`builtin_enum_variant_names` stay — fourteen intruders have been found inside proposed ranges in this
campaign, and these four are fenced by name.
`grep -c "fn reset_kernel_stop\|fn request_kernel_stop\|fn no_field_names\|fn builtin_enum_variant_names" src/runtime.rs`
must be **4**.

**⛔ STOP-3 — ONE EDGE FILE, ONE MODULE. NO MERGING, NO SPLITTING.** `abort.rs` will hold one
function and `message.rs` will hold ~2,126 lines. That asymmetry is `src/intrinsic/kernel/`'s own
split and is the thing being mirrored. Do not merge the small modules; do not split `message`. If a
module looks wrongly sized, that is the edge's ruling, not an error to correct.

**⛔ STOP-4 — IMPORT FROM THE CANONICAL HOME, NEVER THROUGH `runtime`'s FACADE.** `runtime.rs`
re-exports 22 `crate::value` names, so `use crate::runtime::Value` compiles and is a lie. Import
`Value`/`EnumValue`/`Environment`/`SymbolTable`/`EvalBreak`/`RuntimeError` from `crate::value::`,
spans from `crate::span`, AST from `crate::ast`. ⚠ The pre-existing facade imports at
`src/kernel/address.rs:34`, `listener.rs:36`, `spawn.rs:83` belong to a separate open sweep —
**leave all three exactly as they are.**

**⛔ STOP-5 — the died-error cluster stays.** `message_only_failure`, `loci_died_disconnected`,
`record_field_by_name` are reach-backs, not members; their home is deliberately unassigned. Import
them; do not move them. `loci_died_disconnected` and `record_field_by_name` are two of § 2's
visibility bumps for exactly this reason.

**STOP-6 — verbatim.** No signature tidying, no merging lookalike functions, no converting free
functions into `impl` methods. Visibility changes beyond § 2's five are possible; report each one you
make and why the compiler forced it.

**STOP-7 — run the orphaned-doc-block scan** over every cut site. ⚠ Scan for contiguous plain `//`
too, not only `///` — a prior rider found a block mixing `///` with a `//` rationale above an
`#[allow]` whose extraction silently truncated it.

## Report

Per-file diff summary; the seven module headers verbatim; **the rewritten `src/kernel/mod.rs` scope
paragraph and confirmation the rune is struck**; the rewritten `src/record/mod.rs:31`; STOP-1 and
STOP-2's grep counts; every visibility change beyond § 2's five, with the compiler's reason; each new
module's `use` block; before/after `wc -l src/runtime.rs`; the doc-block scan result. Then the part
the orchestrator cannot reconstruct: **what surprised you** — an edge header that named a function
the list does not contain, an item whose body did not belong with its module's siblings, or a
delegation that did not resolve where the header said it would.
