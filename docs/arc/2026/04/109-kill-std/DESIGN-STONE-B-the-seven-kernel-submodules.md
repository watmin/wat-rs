# DESIGN — STONE B: the seven kernel sub-modules mirror their edge

> Second of the two stones drawn in `[[DESIGN-STONE-the-kernel-family]]`. Stone A landed at
> `a35f1bac7` (33 items, `runtime.rs` 24,103 → 23,830). This is the other half.
>
> ⚠ **Everything below was re-derived on the post-A file, by NAME.** Stone A moved 33 items out;
> every line number in the family DESIGN is now stale. Re-deriving is the whole reason this stone
> gets its own document rather than an amendment.

## ⛔⛔ THE INSTRUMENT FINDING — a transitive closure cannot decompose an interpreter

Stone A's miss (`reply_failed_reason`, found by the compiler and the rider rather than by me) had one
obvious remedy: run the dependency closure **to fixpoint** instead of one hop. I ran it. It returned
**162 candidates and declared 55 of them "exclusive to the kernel family"**, among them:

```
eval_walk · eval_subtype · eval_conforms · eval_match · parse_let_binding · LetBinding
step_list · step_let · step_do · step_if · step_match · step_user_call · step_to_watast
apply_value · apply_tracked_callee · try_match_pattern_ast · eval_form_{ast,edn,file,step,…}
```

**That is the eval spine** — the residue the recast returned a defensible **LEAVE** for, *"the
load-bearing evaluator, not several concerns wearing one name."* A stone built on that verdict would
have moved the interpreter into `src/kernel/` and called it kernel work.

★★★ **Why it lies, stated so it is never re-run:** "exclusive" means *no consumer outside the reached
set*. In an interpreter every verb calls `eval_inner`, and `eval_inner` reaches the whole file — so
the reached set becomes most of the module and the exclusivity test becomes a **tautology**. The
closure did not measure cohesion; it measured its own reach.

**The rule this stone fixes, and the one every later stone inherits:**

> **Membership comes from the EDGE, never from the call graph. Dependencies are not membership.**
> An item belongs to a family iff `src/intrinsic/<family>/` delegates to it. Its callees are
> *reach-backs* to be classified, never candidates to be absorbed. The dependency scan is run
> **one hop over the FINAL item set**, and re-run whenever that set changes.

⚠ And the one-hop scan's own failure mode, which is what bit stone A: it is only correct if it is
re-run **after** the set is amended. A's set grew 26 → 33 from a range sweep and the scan was not
re-run over the additions. **Extending the item set without re-running the instrument that produced
it is the defect** — not the hop count.
`[[feedback_validate_a_search_pattern_before_trusting_its_count]]`

## The measurement, correctly taken

One hop over the final 34 yields **14 dependencies, every one with a consumer outside the family.**
**Nothing extra moves.** Stone B is exactly its seed:

```
34 items · 4,152 lines · runtime.rs 23,830 -> 19,678
```

## The seven sub-modules, each mirroring its edge file

| sub-module | items | lines | first item (by name — line is indicative only) |
|---|---:|---:|---|
| `kernel/abort` | 1 | 59 | `eval_kernel_raise` |
| `kernel/ambient` | 3 | 79 | `eval_kernel_stopped` |
| `kernel/identity` | 5 | 312 | `eval_peer_pid` |
| `kernel/message` | 6 | 2,126 | `wrap_connect_request` |
| `kernel/resource` | 12 | 1,190 | `eval_listener_prime` |
| `kernel/serve` | 2 | 179 | `eval_retag_op` |
| `kernel/source` | 4 | 177 | `eval_kernel_here` |
| → existing `kernel/spawn.rs` | 1 | 30 | `extract_panic_payload` |

★ **`extract_panic_payload` gets no new module.** Its sole caller in the tree is
`src/kernel/spawn.rs:792` — it joins the file that already calls it. A new module for one function
whose only consumer is an existing sibling would be structure for its own sake.

★ `eval_retag_op` lands in `kernel/serve` — the item the recast named as an intruder inside the
**record** stone's proposed range, correctly excluded there. This is the stone it was excluded for,
and `src/record/mod.rs:31`'s *"`eval_retag_op` did NOT move"* becomes false here.

★ The scatter the recast flagged closes: `eval_kernel_here` and `eval_kernel_call_site` serve the
same three-delegate edge file and today sit **2,170 lines apart**.

## THE ONE CONTRACT DECISION — pinned

**One sub-module per edge file, named for the edge file, no exceptions and no consolidation.**

`kernel/abort` holds one function and `kernel/message` holds 2,126 lines. That asymmetry is not a
defect to correct — it is `src/intrinsic/kernel/`'s own split, which was made by decision at the
edge and is the thing being mirrored. Merging `abort` into `resource` because it is small, or
splitting `message` because it is large, substitutes my judgement for a ruling already made and
breaks the one property that makes this decomposition checkable: **every impl module has exactly one
edge file, and every edge file has exactly one impl module.**

## ⛔ The stays-side visibility list — derived, not discovered

Two stones running, the tree failed to compile or lint after the rider finished, for the same
structural reason: **the rider bumps the visibility of what it moves, and cannot see what the move
requires on the side that stays.** Holon cost three unused imports; stone A cost four private
functions and a hard compile error.

That failure class has now fired twice under briefs that said "visibility changes are expected on
both sides." That sentence is the **convention** rung. Here is the check rung — the exact list, from
the one-hop scan:

```
eval_tail (994) · try_match_pattern (7589) · record_field_by_name (11089)
value_from_frame_info (11141) · loci_died_disconnected (11802)
```

These five are private today, stay in `runtime.rs`, and the seven sub-modules import them. **The
brief names them and the rider bumps them.** The other nine reach-backs (`eval`, `eval_inner`,
`builtin_enum_variant_names`, `no_field_names`, `message_only_failure`, `value_from_span`,
`value_to_watast`, `format_panic_payload`, `KERNEL_STOPPED`) are already `pub`/`pub(crate)`.

## The 14 reach-backs, classified — this is the half-migration test

| reach-back | what it is | verdict |
|---|---|---|
| `eval` · `eval_inner` · `eval_tail` | the eval spine — the campaign's permanent LEAVE | ✅ legitimate |
| `builtin_enum_variant_names` · `no_field_names` | generic `Value`/`EnumValue` constructors, 8 and 11 consuming homes | ✅ ninth/tenth intruders, already fenced |
| `message_only_failure` · `loci_died_disconnected` · `record_field_by_name` | died-error cluster — map item 4, home deliberately unassigned | ✅ pending item 4 |
| `try_match_pattern` · `value_to_watast` · `value_from_span` · `value_from_frame_info` · `format_panic_payload` · `KERNEL_STOPPED` | shared machinery, 1–5 other homes each | ✅ legitimate |

**Not one is kernel-domain vocabulary.** ★ That is the test `src/numeric/` failed — it reached back
for `I64ArithErr` and `to_bigrational`, its *own tower*. Stone B reaches back only for the spine and
for vocabularies whose homes are shared or deliberately unassigned. **Measured, not asserted:** the
numeric home is now healed (`src/numeric/*.rs` imports only `eval_inner`/`eval_one_arg`), so the
comparison is against a fixed reference, not a broken one.

## The prose the move falsifies

```
src/record/mod.rs:31       "**eval_retag_op did NOT move.**"           ⛔ a stated law, falsified here
src/kernel/mod.rs § Scope  "…live in src/runtime.rs (runtime.rs:4206-4218)"
                           + rune:exigere(scope-affirmative) — ⛔ ITS REASON EXPIRES WITH THIS STONE.
                           Stone A correctly did NOT strike it (the rune's condition is verb-IMPL
                           homing). B satisfies it. Strike it here.
src/kernel/peer.rs:265     "(eval_peer_try_send_prime in runtime.rs maps this 1:1)"
src/kernel/peer.rs:471     "(runtime.rs::eval_kernel_serve_dispatch_op_tail)"
src/kernel/listener.rs:101,237 · address.rs:96,155   "former arm of eval_accept_prime/eval_connect_prime"
src/value/frame.rs:85 · src/types.rs:2097 · src/check.rs:4206,9898,10868,11636
src/rust_deps/custodia.rs:112 · src/intrinsic/kernel/message.rs:53
tests/comms/probe_select_flood_no_deadlock.rs:6   "(src/runtime.rs ~24755)"   ⛔ already dead
```

## THE FOUR QUESTIONS — flat YES/NO

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **seven modules mirroring the edge, one stone** | YES | YES | YES | YES | ✅ **ADMITTED** |
| the fixpoint closure's 89 items | YES | YES | **NO** | — | ⛔ DISQUALIFIED |
| merge `abort`+`ambient`+`source` (all small) | **NO** | YES | YES | — | ⛔ DISQUALIFIED |
| split `kernel/message` (2,126 lines) | **NO** | YES | YES | — | ⛔ DISQUALIFIED |
| `extract_panic_payload` gets its own module | YES | YES | YES | **NO** | ⛔ DISQUALIFIED |

- **fixpoint Honest? NO** — measured above: it absorbs the eval spine the recast ruled a LEAVE.
- **merge-the-small Obvious? NO** — breaks the one-edge-one-module property; a reader can no longer
  find an impl from its edge. Size is not the organising principle here; the edge is.
- **split-message Obvious? NO** — same, from the other direction. `eval_poll_prime` (766) and
  `eval_peer_select_prime` (714) are two thirds of it and both serve `message.rs`.
- **own-module Good UX? NO** — one function, one caller, and that caller is `kernel/spawn.rs`.

## Out of scope = REJECTED (not deferred)

- **Map item 4, the died-error cluster.** Three of its members are reach-backs here. Its home stays
  unassigned; this stone does not pick one.
- **`src/kernel/{address,listener,spawn}.rs`'s facade imports** — the re-point sweep's, still open.
- **`reply_failed_reason`** — surfaced by stone A's rider: single caller, ~3,000 lines from it,
  reads as `src/services/`'s. Recorded on the map; not this stone's.
- **The eval spine.** Named here only so the fixpoint result cannot be mistaken for a mandate.

## Acceptance — rows chosen to be unfakeable

| what | command | expected |
|---|---|---|
| one module per edge file | `ls src/kernel/{abort,ambient,identity,message,resource,serve,source}.rs` | 7 files |
| the edge stops naming the megafile | `grep -c "crate::runtime::eval_" src/intrinsic/kernel/*.rs` | 30 → **0** |
| ⛔ the spine did not move | `grep -c "^fn eval_tail\|^pub(crate) fn eval_inner\|^pub fn eval\b" src/runtime.rs` | **3** |
| ⛔ the intruder fence | `grep -c "fn reset_kernel_stop\|fn request_kernel_stop\|fn no_field_names\|fn builtin_enum_variant_names" src/runtime.rs` | **4** |
| the five stays-side bumps | `grep -c "^pub(crate) fn \(eval_tail\|try_match_pattern\|record_field_by_name\|value_from_frame_info\|loci_died_disconnected\)" src/runtime.rs` | **5** |
| the falsified law | `src/record/mod.rs:31` | rewritten; names `kernel/serve` |
| the expired rune | `src/kernel/mod.rs` | `rune:exigere(scope-affirmative)` **struck** |
| bodies verbatim | diff each moved item vs `git show HEAD:src/runtime.rs` | byte-identical |
| runtime.rs | `wc -l` | 23,830 → **~19,678** |
| floor | `scripts/floor.sh`, exit read UNPIPED | 5114/5114, 0 failed |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | 0 |

★ **If the visibility list above is complete, the tree compiles on the rider's first pass** — and
that, not the line count, is this stone's real result. Two stones have proved the class exists;
this one tests whether deriving the list mechanically removes it.
