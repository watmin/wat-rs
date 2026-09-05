# DESIGN — no client call can hang

**Rung 3.** `wat/service.wat` — the one line every generated client method expands to.
Correctness. No perf work.

## WHY — one line, 220 surfaces

`wat/service.wat:2237`, inside the quasiquoted body of every generated client method:

```wat
~r-sym (:wat::kernel::recv c)          ;; a bare, unbounded receive
```

**That is the whole exposure.** Every Peer surface in the tree — 220 of them across 162 files —
gets a method whose body is `send`, then *that*. It is why `Seen/mark` hung a worker ~160 s, and
why `check` needed forty hand-rolled lines to avoid the same fate.

## ⛔ THE SHAPE CHANGED WHEN I READ THE MACRO — the migration is NOT needed

My scoping said this cost 164 files re-matching a new outcome arm. **Measured, it is worse than
that: 643 `RecvOutcome::Message` arms across 282 files.** A fifth arm is not a stone, it is a
season.

★ **So the deadline does not return — it RAISES.** On expiry the generated method calls
`assertion-failed!` naming surface, verb and deadline.

- **No type change.** `RecvOutcome` keeps its four arms.
- **Zero call sites change.** All 643 matches are untouched.
- An infinite silent hang becomes an immediate, named, diagnosable death.

★★ **This is already the method's contract, not a new behaviour.** `service.wat:963`: *"the
generated client method surfaces it as an unignorable raise carrying the cause's reason."* A
protocol-tier failure already raises through this exact path. A deadline joins it.

★★★ And it is honestly rung 3: **the generated method can no longer hang. The wrong thing has no
form.** A caller that genuinely wants to *handle* a timeout uses
`:wat::service::call-by-deadline` — which is what the circuit's four hot calls already do.

## ⛔ THE ONE CONTRACT DECISION

**The deadline must fire before the harness kills the process, or the diagnostic is destroyed.**

That is not a taste; it is this arc's most expensive lesson, paid twice — a `TIMEOUT [30.015s]`
with an empty ARM, and a `drained-never` that needed 64 s to print inside a 30 s cap.

**Default: 10 000 ms.** Enormously generous for any single round trip in this tree (store scans
and stats calls are milliseconds), and comfortably inside nextest's 30 s kill, so the raise
*prints* instead of being truncated into silence.

Tunable per feature by an optional `:deadline-ms`, following `:max-frame-bytes`'s existing
optional-with-default clause shape (`service.wat:572-578`).

⚠ **Optional-with-a-default, never optional-off.** Your ruling at `service.wat:372-377`: *"A knob
whose off-position is 'die on a malformed frame, for every client at once' is a non-option
surfaced as a choice."* A deadline whose off-position is *hang forever* is the same non-option.

## THE TOOLKIT — where wat-grep, rete and wat-fix actually earn their place

The corpus migration dissolved, but the **census did not**, and it cannot be a grep:

1. **wat-grep + rete — the census.** A generated-method call site is structurally
   *a `match` whose scrutinee is a `/`-headed call and whose arms are `RecvOutcome::` variants* —
   distinguishable from a raw `(:wat::kernel::recv …)` match by the scrutinee's head, and from a
   record accessor (`:ns::Rec/field`, the identical name shape) by the arms. **Grep cannot tell
   any of these apart.** The finder produces the true population.
2. **The floor is the second census.** 5215 tests exercise these methods; a default that is too
   short reds them, naming the surface. That is a stronger census than any static count.
3. **wat-fix — the migration, if the floor asks for one.** Any surface whose legitimate round
   trip exceeds the default needs `:deadline-ms` declared. **Do not guess which.** Let the floor
   name them, then record the insertions as a migration.

## FILES

`wat/service.wat`. Plus, only if the floor demands it, `:deadline-ms` declarations via a recorded
codemod.

## OUT OF SCOPE = REJECTED

- **A fifth `RecvOutcome` arm.** 643 sites, and each arm's body is a judgement, not a rewrite.
- **Routing a timeout into `Lost`.** `service.wat:2253-2258` records someone un-collapsing
  `Stopped` from `Lost` in this very arc. Re-collapsing a different condition into it, one arc
  later, is that mistake made deliberately.
- **Touching `call-by-deadline` or its four call sites.** They are the escape hatch and they work.
- All perf work.
