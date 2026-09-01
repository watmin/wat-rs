# DESIGN — STONE: the kernel family comes home (two stones, A then B)

> **Builder, 2026-09-01:** *"kernel family next - draw it"*
>
> Map item 3 of `[[NOTE-partire-RECAST-on-the-current-runtime]]`. The recast said *"~30 items,
> 7 sub-modules MIRRORING `src/intrinsic/kernel/`'s 7 edge files. Home exists."* Both halves
> confirmed on the disk — and the item count is **short**, in exactly the way the numeric stone's
> was short. See "The count the map gave me was wrong" below.

## ★★ The destination home named this stone before I did

`src/kernel/mod.rs` § Scope boundary, written by arc 214:

> *"The polymorphic kernel verbs (`send'`/`recv'`/`close'`/`select'`/`poll'`) SHIPPED in Stone
> 4.6a-ii/4.6b — live in `src/runtime.rs`… **Homing those impls INTO this kernel home is the
> structurally-right next step**; that migration rides the runtime.rs flat-sea (Phoenix) warding
> campaign."*
> ```
> // rune:exigere(scope-affirmative) — verb-homing into kernel/ rides the
> // runtime.rs flat-sea (Phoenix) warding campaign, not this kernel home's scope.
> ```

Arc 214 scoped the homing out **affirmatively** and assigned it to this campaign. This stone is the
campaign arriving. ⚠ **Therefore the rune's reason expires when this lands** — an exemption is
weighed against present truth, and "rides a future campaign" is false once the campaign has ridden.
Striking it is part of the work, not a tidy-up.

## The count the map gave me was wrong — and it is the numeric shape exactly

The map says **~30 items**. Thirty is the count of functions the seven edge files *name*. The
**transitive closure** is 60, and the 30 it omits are the ones that would have made
`src/kernel/` reach back into `runtime.rs` for its own domain vocabulary — which is precisely
what `src/numeric/arith.rs:20` does today and what
`[[feedback_a_lesson_learned_and_then_dropped]]` was minted for.

```
30   functions the 7 edge files delegate to        (the map's number)
26   peer-outcome constructors                     ⬅ THE OMITTED VOCABULARY
 7   *_OUTCOME_TYPE type-path consts               ⬅ FOUND BY THE RANGE SWEEP, AMENDED IN
 4   exclusive helpers (SIGNAL_TYPE, bound_names, extract_panic_payload, wrap_connect_request)
───
67   items, ~4,500 lines.   runtime.rs 24,103 -> ~19,603
```

★ **I derived the closure rather than reading the edges' `use` lines**, because reading the edges is
what produced the 30. The instrument: for every family member, every runtime-local name its body
references, then for each candidate every caller in the tree — including `mod tests`.

## ⛔ Two contaminated instruments, caught before this was drawn

Both are recorded because each would have shipped a wrong stone, and neither was caught by re-reading.

1. **The size instrument's boundary regex omitted `mod`/`impl`/`trait`**, so any function followed by
   the 6,536-line `mod tests` absorbed it. It reported **10,473 lines for 30 functions** in a
   24,103-line file — arithmetically impossible, which is the only reason I looked.
   `[[feedback_validate_a_search_pattern_before_trusting_its_count]]`
2. **The caller census could not see `mod tests`.** It classified `reset_kernel_stop` as *exclusive
   to the family*; its only two callers are at `runtime.rs:19906` and `:19916`, inside the test
   module. A whole call region was invisible to a census that reported completeness.

## ⛔ THE ONE CONTRACT DECISION — pinned

**The impl mirrors the edge, one sub-module per edge file; and the outcome vocabulary is ONE module,
never scattered across the seven that use it.**

The seven sub-modules are not a taxonomy I invented — `src/intrinsic/kernel/` is *already split by
decision* at the edge, and each impl function is matched to the edge file that delegates to it. The
vocabulary is held out because `recv_`/`send_`/`try_send_`/`close_`/`signal_`/`accept_`/
`connect_outcome_*` is one enum-construction language whose members call each other; splitting it by
which verb happens to use each member today would fuse a shared vocabulary into one caller's file —
**the `peer_protocol` mistake the recast refused for item 4.**

## The two stones, and why A must precede B

| | stone | items | lines | what it delivers |
|---|---|---|---|---|
| **A** | the outcome vocabulary → `src/kernel/outcome.rs` | **33** | ~385 | **dissolves 8 existing `crate::runtime::` call sites in `src/kernel/`** |
| **B** | the seven edge-mirroring sub-modules | 34 | ~4,113 | `abort · ambient · identity · message · resource · serve · source` |

★ **A-first is derived, not preferred.** B-first would land the seven impl modules in `src/kernel/`
while their outcome constructors were still in `runtime.rs` — the home reaching back into the
megafile for its own domain vocabulary. That is the numeric half-migration, created deliberately,
with a stone's gap in which to be forgotten. **A-first never constructs that state.**

⚠ A's own transient is the opposite direction and is benign: between A and B, `runtime.rs`'s
remaining kernel verbs import the vocabulary from `crate::kernel::outcome`. `runtime.rs` already
does exactly this with `use crate::holon::*;`, and the direction is the one the campaign wants —
things leaving, not arriving.

## Stone A — the vocabulary (33 items)

⚠ **AMENDED before briefing.** The span sweep this DESIGN's own range-trap rule demanded found
**seven `*_OUTCOME_TYPE` type-path consts** interleaved with the constructors —
`RECV_`/`SEND_`/`TRY_SEND_`/`CLOSE_`/`SIGNAL_`/`ACCEPT_`/`CONNECT_OUTCOME_TYPE`. Each is referenced
**only** by its own constructors (measured, every reference inside that constructor group's lines).
A vocabulary of 26 constructors that leaves its 7 type paths behind is the half-migration at const
granularity. `[[feedback_a_lesson_learned_and_then_dropped]]` — caught pre-flight this time, by the
sweep rather than by a rider five stones later.

The 26 constructors, all measured: `recv_outcome_{message,closed,lost,shutdown,from_decoded}` ·
`send_outcome_{sent,closed,stopped,from_error,lost}` ·
`try_send_outcome_{sent,would_block,closed,lost}` · `close_outcome_{closed,signaled,failed}` ·
`signal_outcome_{delivered,failed}` · `accept_outcome_{accepted,closed,failed}` ·
`connect_outcome_{connected,refused,rejected,failed}`.

★ **Seven of them have ZERO callers inside `runtime.rs`.** `accept_outcome_*` (3) and
`connect_outcome_*` (4) are called only from `src/kernel/listener.rs:470-473` and
`src/kernel/address.rs:340-344`. They are already orphaned in the megafile, serving a home they do
not live in — which is the clearest possible evidence for where the whole vocabulary belongs.

⚠ **One member is consumed by the died-error cluster that STAYS.** `recv_outcome_shutdown` is called
by `loci_died_from_send_error` and `thread_died_error_runtime` — map item 4, whose home is
deliberately unassigned. Those two import it from `crate::kernel::outcome` after A. **This is named,
not discovered later:** it is a stays-caller reaching into a moved home, which is ordinary, and it is
*not* grounds to hold the vocabulary back.

## Stone B — the seven, each mirroring its edge

| sub-module | edge file | items | lines |
|---|---|---|---|
| `kernel/abort` | `intrinsic/kernel/abort.rs` | 1 | 59 |
| `kernel/ambient` | `intrinsic/kernel/ambient.rs` | 3 | 79 |
| `kernel/identity` | `intrinsic/kernel/identity.rs` | 5 | 312 |
| `kernel/message` | `intrinsic/kernel/message.rs` | 6 | 2,126 |
| `kernel/resource` | `intrinsic/kernel/resource.rs` | 12 | 1,151 |
| `kernel/serve` | `intrinsic/kernel/serve.rs` | 2 | 179 |
| `kernel/source` | `intrinsic/kernel/source.rs` | 4 | 177 |
| `extract_panic_payload` | (no edge — called by `src/kernel/spawn.rs:792`) | 1 | 30 |

★ `eval_retag_op` lands in `kernel/serve` — the item the recast named as an **intruder** inside the
`record` module's proposed range, whose sole caller is `intrinsic/kernel/serve.rs`. It was correctly
excluded from the record stone; this is the stone it was excluded *for*.

★ The scatter the recast flagged is real and this cut closes it: `eval_kernel_here` (7343) and
`eval_kernel_call_site` (9519) serve the same three-delegate edge file **2,176 lines apart**.

## ⛔ Named non-movers — the intruder fence

| item | why it stays |
|---|---|
| **`reset_kernel_stop`** | ⛔ **Eleventh intruder, and the first found by COHESION not consumption.** Consumption says exclusive; it belongs to the `KERNEL_STOPPED` process-lifecycle trio at `runtime.rs:117-160` with `request_kernel_stop`/`set_kernel_sig*`/`reset_user_signals`, consumed by `freeze`, `distribution`, `process/child`, `host/entry`. |
| `KERNEL_STOPPED` · `KERNEL_SIGUSR1/2` · `KERNEL_SIGHUP` · `request_kernel_stop` | same trio; consumed by four other homes |
| `no_field_names` · `builtin_enum_variant_names` | ninth and tenth intruders; 10 and 7 consuming homes |
| `message_only_failure` · `record_field_by_name` | died-error cluster (item 4), home unassigned |
| ⛔ **`loci_died_error_from_reason` (11794) · `loci_died_disconnected` (11875) · `loci_died_from_send_error` (11900)** | **Twelfth, thirteenth, fourteenth intruders — woven THROUGH stone A's block**, between the `recv_` and `send_` constructors. Died-error cluster (item 4), home unassigned. A contiguous cut takes all three. |
| `thread_died_error_runtime` | item 4; CALLS into A's vocabulary, does not join it |

## The prose the move falsifies — half the stone, and the half that rots silently

**Every external reference to these 60 items is a doc comment, not a call** (measured: 12 candidate
"external consumers" resolved to 11 mentions + 1 real call, `extract_panic_payload` from the
destination home). So the code cost is near zero and the **comment** cost is the stone:

```
src/record/mod.rs:31    "**eval_retag_op did NOT move.** It sits between eval_variant and…"
                        ⛔ THIS STONE FALSIFIES A STATED LAW. Not a stale pointer — a claim.
src/kernel/mod.rs:43    "…live in src/runtime.rs (registered at runtime.rs:4206-4218)"
                        + the rune:exigere(scope-affirmative) whose reason expires
src/kernel/peer.rs:265  "(eval_peer_try_send_prime in runtime.rs maps this 1:1)"
src/kernel/peer.rs:471  "(runtime.rs::eval_kernel_serve_dispatch_op_tail)"
src/kernel/listener.rs:142 · address.rs:96,155 · listener.rs:101,237 — "former arm of eval_*_prime"
src/value/frame.rs:85   "(runtime.rs, beside eval_kernel_call_site)"
src/types.rs:2097 · src/check.rs:4206,11636,10868,9898 · src/edn/render.rs:3699
src/rust_deps/custodia.rs:112 · src/intrinsic/stream.rs:49,60 · src/intrinsic/kernel/message.rs:53
tests/comms/probe_select_flood_no_deadlock.rs:6  "(src/runtime.rs ~24755)"  ⛔ ALREADY DEAD — the
                        file is 24,103 lines. A line-citation that expired before this stone existed.
```

⚠ `src/kernel/mod.rs`'s own header boasts that after a 2026-06-08 vigilia *"zero intra-home line
cross-refs remain"* — the drift class was extirpated **inside** the home while these cross-**file**
line citations kept rotting. Cite by grep-token, never by line number.

## THE FOUR QUESTIONS — on the decomposition, flat YES/NO

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **A then B** | YES | YES | YES | YES | ✅ **ADMITTED** |
| one stone, all 60 items | YES | **NO** | YES | — | ⛔ DISQUALIFIED |
| B then A | YES | YES | **NO** | — | ⛔ DISQUALIFIED |
| eight stones (one per edge + vocabulary) | YES | YES | YES | **NO** | ⛔ DISQUALIFIED |
| the map's 30 items only | YES | YES | **NO** | — | ⛔ DISQUALIFIED |

- **one-stone Simple? NO** — seven new modules, a new vocabulary module, and ~4,500 moved lines in
  one diff; a red could not be attributed to a sub-module.
- **B-then-A Honest? NO** — measured above: it deliberately constructs the numeric half-migration.
- **eight-stones Good UX? NO** — the seven share one `src/kernel/mod.rs` edit and one import
  restructuring; eight stones means eight conflicting edits to that file and eight floor runs for
  one coherent change. Obvious/Simple/Honest all hold, so this is a genuine UX cut, not a dodge.
- **map's-30 Honest? NO** — it ships `src/kernel/` reaching into `runtime.rs` for its own outcome
  vocabulary. That is the numeric defect, knowingly reproduced.

## Out of scope = REJECTED (not deferred)

- **Map item 4, the died-error cluster (~55 items).** Its home is deliberately unassigned and this
  stone does not assign it. Two of its members call into A's vocabulary; that is a permitted
  direction, not a reason to fold them in.
- **`src/kernel/`'s pre-existing `crate::runtime::{EvalBreak, RuntimeError, RuntimeErrorKind,
  SymbolTable, Value}` facade imports** (`address.rs:34`, `listener.rs:36`, `spawn.rs:83`). Real
  instances of the facade artifact, and they belong to the **facade re-point sweep** already open on
  the seam. Folding them in here would make a red unattributable between two causes.
- **Arc 214 Stone 4.6's pending no-prime type registration.** A different rune, still live.

## Acceptance — rows chosen to be unfakeable

| what | command | expected |
|---|---|---|
| A: the vocabulary is whole | `grep -c "^pub(crate) fn .*_outcome_" src/kernel/outcome.rs` | **26** |
| A: its type paths came too | `grep -c "^pub(crate) const .*_OUTCOME_TYPE" src/kernel/outcome.rs` | **7** |
| ⛔ A: the woven intruders stayed | `grep -c "fn loci_died_error_from_reason\|fn loci_died_disconnected\|fn loci_died_from_send_error" src/runtime.rs` | **3** |
| A: the home stops reaching back | `grep -c "crate::runtime::\(accept\|connect\|send\)_outcome" src/kernel/*.rs` | 8 → **0** |
| A: none left behind | `grep -c "fn .*_outcome_[a-z]*(" src/runtime.rs` | **0** |
| B: each edge's delegations resolve to the new home | per edge file, `crate::runtime::eval_*` → `crate::kernel::*` | 30 → **0** in `intrinsic/kernel/` |
| ⛔ the intruder fence | `grep -c "fn reset_kernel_stop\|fn request_kernel_stop\|fn no_field_names\|fn builtin_enum_variant_names" src/runtime.rs` | **4** |
| the law that was falsified | `src/record/mod.rs:31` | rewritten; names the new home |
| the expired rune | `src/kernel/mod.rs` | `rune:exigere(scope-affirmative)` **struck**, scope-boundary rewritten |
| bodies verbatim | diff each moved item vs `git show HEAD:src/runtime.rs` | byte-identical |
| runtime.rs | `wc -l` | 24,103 → **~19,626** |
| floor | `scripts/floor.sh`, exit read UNPIPED | 5114/5114, 0 failed |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | 0 |

⚠ **The orchestrator expects one clippy red per stone** that the rider structurally cannot see —
imports left unused by the departing items, as at `runtime.rs:56` in the holon stone. That is the
tier working; it is the orchestrator's to fix, and it is not a rider finding.
