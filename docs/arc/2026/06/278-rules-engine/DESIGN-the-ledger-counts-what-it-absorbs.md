# DESIGN — the ledger counts what it absorbs

**Stone S30.** The instrument that makes `dup=0` mean something. Small, and it goes before 3d.

## WHY

3c struck with `total=8000; distinct=8000; dup=0; disrupts=24` on five runs. Row 4 was graded
**bounded, not withdrawn**, and the executor stated the bound better than I did:

> *"24 severs on idle connections and 24 severs that produced absorbed redeliveries print the same
> summary."*

`:fanout::seen` is `:durable []`. The claim arm returns `Dup` and writes nothing. The worker maps it
to `first? = false` (`circuit.wat:329`) and does not `conj`. So **`dup=0` means the outcome vector has
no second `First`** — not that a redelivery happened and was absorbed.

★ **R69 one layer up.** R69: `distinct` keyed on `queue/envelope-id`, which a retry *replaces*, so the
detector could not witness a duplicate. Here: we fixed the key and **inherited the blindness in the
counter.** An invariant credited with preventing something it cannot see — twice, at two levels, in
one campaign.

## ⛔ WHY THIS GOES BEFORE 3d, AND IT IS NOT TIDINESS

1. **3d cannot be graded without it.** 3d's point is *produce a duplicate, show it absorbed.* With no
   counter its result is `dup=0` and the same two indistinguishable worlds — **deliberately
   reproducing the bound we just put on row 4.** Stone D already ruled this class: *a bound that only
   says "timed out" is the empty ARM again.*
2. **★ It is a measurement about work already done.** Re-run 3c's existing chaos with the counter and
   we learn **today** whether those 24 severs produced absorbed redeliveries. That number does not
   exist and cannot be inferred.
3. **It may change what 3d is FOR.** If 3c's severs already exercise the dedupe path, 3d stops being
   *"produce duplicates"* and becomes specifically *"produce the **unknowable** state"* — a narrower
   and more honest stone. Drawing 3d first is drawing on an unmeasured premise.

## WHAT IT DELIVERS

```
:durable [firsts <- i64  dups <- i64]        ;; was []
```

plus a `stats` feature on `:fanout::Seen` returning both, and the circuit's summary line carrying
them. The claim arm already branches `Some`/`None` on the ledger — it increments on the branch it
already takes.

**The two worlds become different lines:**

```
disrupts=24; seen-firsts=8000; seen-dups=0      ← the severs never interrupted a claim
disrupts=24; seen-firsts=8000; seen-dups=17     ← 17 redeliveries, all absorbed
```

## ⛔ THE ONE CONTRACT DECISION

**The summary must distinguish "N redeliveries, all absorbed" from "no redelivery occurred."**

Not "a counter exists." If the two worlds still print the same line, the stone has added a field and
changed nothing — which is precisely the failure it exists to fix.

## FILES

`wat-scripts/fanout/circuit.wat` only — the `Seen` surface, its service, and the summary.

**No `wat/service.wat`. No `src/`. No codemod. No 3d.**

## OUT OF SCOPE = REJECTED

- **3d, the reply-drop.** Its own stone, and it is *why* this one goes first. `None` → `LOST` is
  already proven userland (`probe-reply-drop-is-userland.wat`).
- **Making `claimed` durable.** ★ It is `:ephemeral` today — **the dedupe ledger does not cross the
  wire and does not survive hibernation.** Restart `seen` and every message looks `First` again.
  Fine for a fixture; a real limit on the words *"idempotent consumer."* **S31**, named not braided.
- **Counting anything else.** Firsts and dups. Not retries, not severs-per-worker.

## THE PROOF

1. **★ Re-run 3c's chaos with the counter and REPORT WHAT IT SAYS.** This is the point of the stone.
   ⛔ **`seen-dups=0` is a RESULT, not a failure** — it would mean 24 severs never interrupted a
   claim in flight, which is a finding about 3c and makes 3d more valuable, not less. Do not tune the
   rate to manufacture a duplicate.
2. **★ The counter can be made to fire.** Drive a deterministic redelivery — `:user::redelivery-is-absorbed`
   already exists at `circuit.wat:1411` — and show `dups > 0`. **A counter that never counts is a
   deleted counter**, and row 1 alone cannot tell the difference.
3. **Rate 0 is unchanged.** `seen-firsts=8000; seen-dups=0`, and the invariant untouched.
4. **The floor**, Summary line, `5213/5213`.
