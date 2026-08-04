# DESIGN-STONE — `send` must carry its cause, and the underscore is how it stopped

> **Status: DRAWN 2026-08-03, unbuilt.** Board **#70**. Blocks **#69**'s third test.
> Found while decomposing #69 (the `libc::raise` rebuild), not while looking for it.

## ★★ THE RULING THAT GOVERNS THE CLASS

> *"the only acceptable opaque error is the '500' that is served to clients and the real cause is
> shipped to the admin handle."* — the builder, 2026-08-03

There is **no exemption list** for a discarded cause. Exactly one legitimate shape exists:

> **REDACT OUTWARD, ROUTE THE TRUTH.** Opaque to the untrusted caller; the real cause travels up
> the admin/lineage channel. Opacity is legitimate *because it has a destination.*

A drop with no destination is not "an opaque error." It is a lost one.

This is the load-bearing correction to the obvious design. `tests/lint/unused_span_justified.rs` is
the right **structural precedent** — a co-located `// rune:lint(…)` rune per ignored `_span`, built
precisely because *"a one-time hand-audit is unreliable (this migration mis-classified it 3×)"* —
but it is the **wrong contract**. That lint offers *earned exemptions* (`infallible`, `located
elsewhere`). Here there are none. A wall for this class must demand the cause be **carried or
routed**, never merely annotated.

## The asymmetry, grounded

**Recv is faithful** (`src/runtime.rs`):

| site | behaviour |
|---|---|
| `:26480`, `:26538` | `PeerRecvError::Shutdown` → `recv_outcome_shutdown()` = `Lost[LociDiedError::Stopped]` — a **named** stop |
| `:26475`, `:26533` | carries `crash_reason` through |
| `:26528` | carries the decode-error text |

**Send throws it away:**

| site | behaviour |
|---|---|
| `runtime.rs:26108/:26139/:26185/:26192` | `Err(_) => send_outcome_lost("send: peer disconnected")` |
| `runtime.rs:26263/:26271` | `Err(_) => try_send_outcome_lost("try-send: peer disconnected")` |
| `channel/transfer.rs:76` | `Err(_) => SendOutcome::Disconnected` *(the Rust-internal enum)* |

Every one an `Err(_)`: the error **value** discarded, replaced by a literal. So a
`RuntimeErrorKind::WriteStopped` (`io.rs:713` — the shutdown broadcast waking a **blocked** write,
the named stop arc-170 closure #5 was built to deliver) is, at the wat surface, indistinguishable
from an ordinary peer disconnect. **None of these is a client-facing 500.** They are the substrate's
own send path: no admin hop, no redaction, no destination. All unjustified under the ruling.

### ⚠ AND THE CARRIERS ARE NOT SYMMETRIC — this is what shapes the fix

```
RecvOutcome::Lost [cause <- :wat::kernel::LociDiedError]   <- a MATCHABLE enum
SendOutcome::Lost [cause <- :wat::kernel::Failure]         <- a FLAT message record
```

Recv was migrated to `LociDiedError` (the arc-278 LociDiedError stone). **Send was not.** So even
threading the real error into `send_outcome_lost`'s `String` would leave the caller reading prose
instead of matching a variant — the flat-String shape the substrate's own doctrine forbids
(*wat is EDN everywhere; a cause is a structured carrier, never a message*).

**And the right answer is already minted.** `:wat::kernel::LociDiedError` declares, among others:

```
Unit("Disconnected")     Unit("Stopped")     Panic{…}     RuntimeError{…}     StartupError{…}
```

`Stopped` and `Disconnected` — **exactly the two states the send path conflates** — already exist,
already distinguish, and are already consumed by recv. Nothing needs inventing; a carrier needs
widening.

## The strike

1. **`SendOutcome::Lost` and `TrySendOutcome::Lost` carry `LociDiedError`, not `Failure`** —
   symmetric with `RecvOutcome::Lost`. A caller matches the cause instead of reading it.
2. **Replace every `Err(_)` in the send path with a match on the error**, mapping
   `RuntimeErrorKind::WriteStopped` → `Lost[LociDiedError::Stopped]` and a genuine peer-loss →
   `Lost[LociDiedError::Disconnected]`, carrying the real reason for everything else.
3. **The corpus cascade is the worklist** — every `SendOutcome::Lost` match site whose arm binds a
   `Failure`. If it is a structural rewrite across many `.wat`, it is a **wat-fix codemod**
   (`wat/fix.wat` + a recorded `wat-scripts/fixes/<migration>.wat`), dry-run on a `/tmp` copy and
   diffed first — never hand-edits. Nearest shape on the shelf:
   `rename-diederror-to-loci-died-error.wat`.

### ⛔ The population is UNMEASURED — do not carry a number into the brief

`grep 'Err(_) *=>'` returns **5**. It **cannot reach** the `try-send` pair, whose arm is formatted
across lines — those were found by a different grep. **At least 7 are known.** 89 bare `Err(_)`
exist in `src/` overall and how many are cause-bearing has **not been measured**.

Treat 7 as a floor, never a census (`[[feedback_a_greps_count_is_not_an_enumeration]]` — violated in
this very investigation, twenty minutes after being quoted). **Impose the check and read the
screams**: make the carrier `LociDiedError`, let the compiler enumerate every site, and take *that*
as the worklist.

## ★ Why this matters beyond one enum — the underscore lineage

Four instances, one character, all in 170/IPC territory, every one hiding something real:

| | the discard | what it hid |
|---|---|---|
| R55 mask #4 | stdlib handlers bound `_cause` | the Lost reason, replaced by a static string |
| R59 | `let _ =` on the stop ask | **`Admin::Stop` had never once been delivered** |
| #67 (open) | `_sig` | slips every must-use gate that bare `_` catches |
| **this** | `Err(_)` ×7+ | `WriteStopped` indistinguishable from a disconnect |

R55 found and fixed the dropped cause **in the stdlib**. This is the same defect **one layer lower**,
in the runtime, still live — which is R57's shape exactly: *the LAW's "complete" was HALF.* The
send-wall (`8e46ace0`) did the big thing — `send` returns a matchable value instead of raising — and
never wired the value's cause. **A failure you can match but whose reason is a constant is a mask
wearing an outcome's clothes.**

And the sharpest asymmetry: wat's must-use gate closes **both** discard doors for wat's *users*
(`do`-non-final ✓, `let`-`_` ✓). The Rust runtime implementing wat has **no such wall**. The
substrate polices its callers harder than it polices itself.

## What this BLOCKS

**#69's third test.** `probe_arc170_writer_joins_lockstep` asserts that a blocked
`PipeWriter::write` wakes with a **named** `WriteStopped` rather than hanging forever. Rebuilt today
as a child-`wat` program, it could only assert *"the child exited"* — losing the discrimination that
**is** its subject. The probe's own STOP-2 note said so. I expected that note to be stale after the
send-wall landed, and checked. It is not.

#69's two recv-side rebuilds are **not** blocked: `Lost[Stopped]` is faithfully visible today, so
they rebuild cleanly and *stronger* than what they replace — asserting the wat-visible contract
instead of a Rust internal.

## STOPs

- **⛔ Do not annotate the drop.** The `unused_span` rune shape does not transfer; per the ruling
  there is no justification for a discarded cause, only redact-and-route.
- **⛔ Do not thread the error into the existing `String`.** That keeps a stringly cause the caller
  cannot match — the flat-String shape the substrate forbids. Widen the carrier.
- **⛔ Do not quote 5 or 7 as the population.** Impose the carrier change and let the compiler
  enumerate.
- **⛔ Do not invent a variant.** `Stopped` and `Disconnected` already exist on `LociDiedError` and
  are already consumed by recv.
- **⛔ Do not hand-edit the corpus.** Codemod, dry-run + diff, shadowdancers strike.
