# BRIEF — the `Stopped` corpus sweep (#73)

> **The references are BUILT and on disk. This brief is the sweep that follows them.**
> Stone: `DESIGN-STONE-a-stop-is-not-a-death.md`. Read it first; this brief does not repeat its
> argument, only what changed when the lair was actually studied.

## ⚠ FOUR THINGS THE STONE GOT WRONG — corrected by grounding, read these before anything

The stone was written before the substrate was read. Its diagnosis holds; its *mechanism* does not.

1. **The stop fact was already produced, on BOTH sides.** `recv_outcome_shutdown()`
   (`runtime.rs:23488`) and `loci_died_from_send_error` → `thread_died_error_shutdown()`
   (`runtime.rs:23615` → `:22970`) have always built `LociDiedError::Stopped`. Nothing was
   silently dropped on those paths.
2. **The lie was `Lost`, not `Closed`.** A stop arrived as `Lost[LociDiedError::Stopped]` — a
   carrier whose payload type is literally named *died*. **So the migration is mostly
   `Lost`-arm → new `Stopped`-arm, NOT "add an arm beside Closed".** A rider hunting `Closed`
   arms will miss the point.
3. **TWO Rust wildcards destroyed the fact, not one.** The stone named `classify_peer_error`
   (`spawn.rs:260`). `classify_peer_death` (`spawn.rs:226`) had the identical
   `Err(_) => PeerDeath::Closed` shape and its own doc said so. Both are fixed.
4. **The scope is 496 sites / 207 files, not "~420 arms / 234 files"** — and the stdlib half of it
   collapsed to **9 files**, because `wat/service.wat` and `wat/test.wat` are macro templates.

## ✅ WHAT IS ALREADY DONE — do not redo any of it

**Rust:** `PeerDeath::Shutdown` minted; both classifiers stop flattening; `ProcessPeerBundle::recv`
reaches thread-tier parity; both `select'` arms map to the existing `ServiceEvent::Shutdown`;
`RecvOutcome::Stopped` + `SendOutcome::Stopped` registered (Unit, four precedents);
`recv_outcome_shutdown()` now builds `Stopped` instead of `Lost[Stopped]`; `send_outcome_from_error`
is the ONE door choosing the variant by a full match with no wildcard.

**Stdlib (all 9 files, clear):** `service.wat`, `test.wat`, `spawn.wat`, `bracket.wat`,
`stdio.wat`, `journal.wat`, `span.wat`, and — cleared *for free* by the `service.wat` template —
`cache.wat`, `query/mem.wat`, `query/sqlite-store.wat`.

## ★ THE FOUR REFERENCES — copy these, do not invent

| bucket | reference on disk | the rule |
|---|---|---|
| **1 · serve loop** | `wat/service.wat`, the `outcome-match` template | `Closed` = one client left → **keep serving**. `Stopped` = the world is ending → **return `nil`, do not recurse**. Sharing the body here is a live bug. |
| **2 · client call site** | `wat/service.wat`'s `send-recv-form`; `wat/telemetry/journal.wat` | Forward the stop **as itself**. Terminal, but named a stop — never "peer closed", never a death. |
| **3 · drain loop** | `wat/spawn.wat` `recv-all-loop'` | A stop truncates the drain → **NOT `Ok`**. `Err(:wat::kernel::LociDiedError::Stopped)`. Claiming a complete collection over a cut-short read is this fn's original sin restored. |
| **4 · test assertion** | `wat-tests/service-locus-parity.wat`; `wat/test.wat` harness | Assert the stop distinctly. The specimen is the point. |

**The single best worked example is `wat/kernel/services/stdio.wat`'s `read-frame`:** a nested
`match cause` that existed only to dig `Stopped` out of a death report collapsed into one top-level
arm — and took a doctrine-illegal `_` wildcard with it.

## THE LOOP — the checker is the worklist, and the binary is stale until you rebuild

```
edit  →  cargo build --release        # MANDATORY: the stdlib is BAKED; --check reads the OLD one until you rebuild
      →  ./target/release/wat --check <a file that reaches your edit>
      →  grep the errors for: missing arm(s) for variant(s): Stopped
```

⚠ **`cargo build --release` going green proves NOTHING here.** The bake does not run the
exhaustiveness sweep. It was green with the whole corpus red. **`--check` is the arbiter.**

## ⛔ THE TRAP THAT WILL BURN YOU — a generated arm reports at the CALLER

An error whose span is a `defservice` form, or a `deftest` form, is a **macro-generated** arm. There
is nothing to edit in that file. It clears when the template does — and both templates are already
fixed. **If you cannot find a `match` at the reported line, it is generated. Leave it and move on.**
(Proven: `wat-tests/service-locus-parity.wat:35` was the `defservice`; it cleared when
`service.wat` did.)

## THE SPLIT — codemod for the mechanical bulk, riders for the residue

Measured over the 207 files: **135 sites are one identical idiom** —
`(:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" …))` — and
**168 are uniform `SendOutcome` triples** ending `((:wat::kernel::SendOutcome::Lost _c) …)` with all
three arms sharing a body.

- **Those go through a recorded wat-fix codemod** (`wat-scripts/fixes/`), dry-run on a `/tmp` copy
  and `diff`'d, idempotent, committed as the migration. That is the standing doctrine for a
  structural rewrite across many `.wat` files, and it is safer than 200 hand-edits.
- **Everything else is a rider**, hand-edited against the four references.

The stone's two STOPs (*"no hand-edits, use the codemod"* and *"do not let a codemod author the arm
BODY"*) are reconciled exactly here: the codemod may insert an arm **only where the body is already
uniform and its precondition is already stated**; wherever a body is a decision, a human makes it.

## ⛔ STOPs

- **⛔ No `_` wildcard on an enum scrutinee.** Doctrine; the checker rule is unbuilt, so nothing
  stops you taking it. Taking it is a rejected strike. (`stdio.wat` just *removed* one.)
- **⛔ `Stopped` must never silently share `Closed`'s body in a serve loop or a drain.** Elsewhere a
  uniform body is legal **only with a comment naming the precondition** — the standing rule: a
  uniform match is legal, an *unexplained* uniform match is a discard.
- **⛔ Riders do NOT run `cargo nextest`.** Edit-only. The orchestrator runs ONE reduce. A rider that
  backgrounds a test run and returns early is the failure mode the record names twice.
- **⛔ One rider owns whole files.** Never two riders in one file.
- **⛔ Do not touch `TrySendOutcome`.** It did not gain a variant; its arms are correct as they are.
- **⛔ Do not "fix" a probe that exists to prove a refusal.** If a file's job is to be RED, it stays RED.

## The state this brief was written against

Rust + all 9 stdlib files done; **496 sites / 207 files remain**, enumerated by the checker into
`all_sites.txt`. The tree is RED by construction and stays red until the sweep completes — this
lands as ONE atomic unit, weighed by the orchestrator's own `cargo nextest run --release` against
the pre-change floor of **4347 / 4347 / 0 / 262**.
