# BRIEF — STONE A: the kernel outcome vocabulary comes home

Move the 33-item peer-outcome vocabulary out of `src/runtime.rs` into a new `src/kernel/outcome.rs`.
DESIGN: `docs/arc/2026/04/109-kill-std/DESIGN-STONE-the-kernel-family.md` (read § "Stone A").

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. The orchestrator builds, floors
and clippies centrally — you do not run `cargo build`/`test`/`nextest`/`clippy` or `scripts/floor.sh`.
**You may not spawn sub-agents.** Work only in `/home/john/work/holon/wat-rs`; verify with `pwd`
first. Do not commit, push, stash, revert, or `git checkout --` anything. Tree clean, floor green at
5114, HEAD `811925431`.

## Read in order

1. The DESIGN's § "Stone A" and § "Named non-movers" — the second one is the whole risk here.
2. **`src/holon/outcome.rs`** — the same name, the same shape, shipped one stone ago. Its module
   header and `use` block are the standard to copy.
3. **`src/kernel/mod.rs`** — the destination's contract. Its § "Scope boundary" names this migration
   and carries a rune about it; read both before you add a `mod` line.
4. `src/runtime.rs:11706–12194` — the block you are cutting from. **Read the whole span before you
   move one item.** Three functions that must NOT move are woven through it.

## The work

### 1 — create `src/kernel/outcome.rs` with the 33

**26 constructors:** `recv_outcome_{message,closed,lost,shutdown,from_decoded}` ·
`send_outcome_{sent,closed,stopped,from_error,lost}` ·
`try_send_outcome_{sent,would_block,closed,lost}` · `close_outcome_{closed,signaled,failed}` ·
`signal_outcome_{delivered,failed}` · `accept_outcome_{accepted,closed,failed}` ·
`connect_outcome_{connected,refused,rejected,failed}`.

**7 type-path consts:** `RECV_OUTCOME_TYPE` · `SEND_OUTCOME_TYPE` · `TRY_SEND_OUTCOME_TYPE` ·
`CLOSE_OUTCOME_TYPE` · `SIGNAL_OUTCOME_TYPE` · `ACCEPT_OUTCOME_TYPE` · `CONNECT_OUTCOME_TYPE`.

Bodies verbatim. All 33 become `pub(crate)` — most are bare `fn`/`const` today; the seven
`accept_`/`connect_` constructors already are. Add `pub mod outcome;` to `src/kernel/mod.rs`.

Write a module header in `src/holon/outcome.rs`'s register: what the vocabulary IS (the
`:wat::kernel::*Outcome` enum-construction language), and the measured fact that earns its home —
**every one of its consumers is `src/kernel/` or a kernel verb; it has none anywhere else in the
tree.**

### 2 — re-point the call sites

The compiler names them. Two directions, both expected:

- `src/runtime.rs` — the kernel verbs still living there import from `crate::kernel::outcome`.
- **`src/kernel/{listener,address,peer}.rs` — eight `crate::runtime::` call sites become local.**
  `listener.rs:470,472,473` · `address.rs:340,342,343,344` · `peer.rs`. This is the stone's point:
  the home stops reaching into the megafile for its own vocabulary.

Leave a short retirement comment at the cut, in the shape the previous stones used.

### 3 — the prose the move falsifies

`src/intrinsic/stream.rs:49,60` and `tests/kernel/probe_arc278_close_outcome_wall.rs:105` name these
constructors in doc comments as living in `runtime.rs`. Correct them to the new home. **Cite by
grep-token, never by line number** — `src/kernel/mod.rs`'s own header records a vigilia that
extirpated line-citation drift inside the home, and these cross-file citations are the same class
still rotting.

## Blast radius

`src/kernel/outcome.rs` (new) · `src/kernel/mod.rs` (one `mod` line) · `src/runtime.rs` (33 items out) ·
`src/kernel/{listener,address,peer}.rs` (call sites go local) · `src/intrinsic/stream.rs` +
`tests/kernel/probe_arc278_close_outcome_wall.rs` (doc text) · whatever else the compiler names.
No `.wat` corpus change. No registrations. **No verb changes behaviour.**

## STOP triggers — each REJECTS; ship nothing further on that point and report

**⛔ STOP-1 — THREE FUNCTIONS ARE WOVEN THROUGH THE BLOCK AND MUST NOT MOVE.**
`loci_died_error_from_reason` (11794), `loci_died_disconnected` (11875),
`loci_died_from_send_error` (11900) sit *between* the `recv_` and `send_` constructors. They are the
died-error cluster, whose home is deliberately unassigned. A contiguous cut takes all three.
`grep -c "fn loci_died_error_from_reason\|fn loci_died_disconnected\|fn loci_died_from_send_error" src/runtime.rs`
must still be **3**. Move item by NAME; never by line range. Fourteen intruders have been found
inside proposed ranges in this campaign, and three of them are in this one span.

**⛔ STOP-2 — `SIGNAL_TYPE` (12053) IS NOT YOURS.** It sits between `close_outcome_failed` and
`SIGNAL_OUTCOME_TYPE` and looks like the eighth const. It names the Signal *argument* enum, not an
outcome; its single consumer is `eval_signal` at `runtime.rs:15509`, which is stone B's. It stays.

**⛔ STOP-3 — `recv_outcome_shutdown` HAS A CALLER THAT STAYS BEHIND.**
`loci_died_from_send_error` and `thread_died_error_runtime` call it and are STOP-1 non-movers. After
the move they import it from `crate::kernel::outcome`. That is correct and expected — a stays-caller
reaching into a moved home. It is **not** a reason to hold `recv_outcome_shutdown` back. If you find
a caller the DESIGN did not name, report it with the line.

**⛔ STOP-4 — IMPORT FROM THE CANONICAL HOME, NEVER THROUGH `runtime`'s FACADE.**
`runtime.rs:758-784` re-exports 22 `crate::value` names, so `use crate::runtime::Value` compiles and
is a lie. Import `Value`/`EnumValue`/`SymbolTable` and friends from `crate::value::`, spans from
`crate::span`. ⚠ `src/kernel/{address,listener,spawn}.rs` already carry facade imports at
`address.rs:34`, `listener.rs:36`, `spawn.rs:83` — **leave them exactly as they are.** They belong to
the facade re-point sweep, a separate open piece of work; touching them here would make a red
unattributable between two causes.

**STOP-5 — verbatim.** No signature tidying, no merging two constructors that look alike, no
`impl`-ing the vocabulary into a type. Visibility changes forced by the boundary are expected on both
sides; report each.

**STOP-6 — run the orphaned-doc-block scan** over the cut region of `runtime.rs`. ⚠ A prior rider
found a block that mixed `///` with a plain `//` rationale above an `#[allow]` and whose extraction
silently truncated it — scan for contiguous `//` too, not only `///`. The seven consts each carry a
doc comment; they are the likeliest place for this to bite.

## Report

Per-file diff summary; the module header you wrote, verbatim; confirmation that STOP-1's three are
still at 3 and `SIGNAL_TYPE` still in `runtime.rs`; the eight `src/kernel/` call sites that went
local, with their new form; each touched file's `use` block; before/after `wc -l src/runtime.rs`; the
doc-block scan result. Then the part the orchestrator cannot reconstruct: **what surprised you** — a
constructor whose body did not match its siblings, a consumer the DESIGN did not name, or a const
that turned out to be referenced from outside its own group.
