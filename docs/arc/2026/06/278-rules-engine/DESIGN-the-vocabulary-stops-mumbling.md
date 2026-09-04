# DESIGN — the vocabulary stops mumbling

**Stone D2.** The residue of the `intueri` cast: names that lie or mumble, and one invisible
mutation. Stone D took the names with a defect behind them; these are the rest.

## WHY

Three `intueri` casts found the lying names concentrated in the **test-helper vocabulary**, not the
protocol. Stone D fixed the four that caused the deadlock. These remain, verified at current lines:

| site | finding |
|---|---|
| `circuit.wat:797-802` `:fanout::accept!` | **rewrites the caller's payload** — `format "{m}\|{t0}"` stamps epoch-nanos before delegating |
| `sns-fanout.wat:663,669-671` `:demo::accept!` | on `Full`: `nap-ms 1` and **recurse, unbounded** |
| `sns-fanout.wat:582` `face-start-tw` | framework-slot name + an unexpanded abbreviation |
| `circuit.wat:644` `face-start` | same |
| six `nap-ms` | `sqs.wat:962`, `sns-fanout.wat:595`, `circuit.wat:657`, and three probes |
| `sqs.wat:834-892` | six `do-` helpers; `do-stats` and `do-depth` issue the **identical** call |

## ⛔ THE ONE CONTRACT DECISION

**The body a caller passes is the body that is sent — or the name says otherwise.**

`(:fanout::accept! t "hello")` sends `"hello|1788493659848314946"`. Nothing at the call site says so;
only the delegate's name (`accept-stamped`) admits it, and the delegate is the *inner* function. A
reader budgeting that line sees a publish. It is a publish, a **payload rewrite**, and an **unbounded
retry loop**, and the name announces none of the three.

That is the Level-1 lie with teeth, and it is grep-checkable: after this stone, a helper that
transforms its argument says so in its name.

## ⛔ THE RETRY IS CORRECT BACKPRESSURE — BOUND IT, DO NOT SHORTEN IT

`accept!` retrying on `Full` is **right**. The queue is bounded; a producer that waits is the whole
design. **Giving up would lose messages.**

So this is a **LIVENESS BOUND** in the arc's own taxonomy
(`BRIEF-278-a-liveness-bound-only-catches-a-hang.md`): *only a hang may trip it.* Raise it to a value
only a stall can reach, and **make it report what it last saw** — depth, cap, attempts, elapsed.
Stone D's rung: where no wire event exists, the poll is bounded and speaks.

⛔ **Do NOT treat it as a WINDOW or shorten it.** A short bound here converts backpressure into loss,
which is strictly worse than the mumble it was meant to fix.

## THE NAMES

| now | becomes | why |
|---|---|---|
| `:fanout::accept!` | **`publish-stamped-until-accepted!`** | publishes, stamps, retries — all three visible |
| `:demo::accept!` | **`publish-until-accepted!`** | no stamp in this one |
| `face-start-tw` / `face-start` | **`start-topic-worker!` / `start-worker!`** | `face` names its slot in the `defservice` taxonomy, not its job; `tw` is expanded nowhere |
| `nap-ms` (×6) | **`await-timer-ms`** | it is a **timer-channel recv**, not a sleep — which is exactly why it is legal where `mora` forbids sleeping. The name buries the one fact that justifies it |
| `do-stats` / `do-depth` | **`read-call-counters` / `read-queue-counts`** | identical `Queue/stats` call; only the kept fields differ, and neither name says which |
| `do-send` / `do-receive` / `do-receive-wait` / `do-ack` | **drop the `do-` filler** | the `:user::` prefix already separates helper from protocol verb |

## OUT OF SCOPE = REJECTED

- **Promoting `nap-ms` to one stdlib verb.** Six byte-identical copies is a promotion question, and
  `sqs.wat:3-5`'s precedent makes promotion **the builder's ruling**. Rename in place; do not
  consolidate.
- **Merging `do-receive` / `do-receive-wait`.** Since Stone B the caller names its own mode
  (`:Immediate` / `:UpTo`), so one helper would do — but that is a *surface* change, not a rename.
  **S33.**
- **`nap-ms` swallowing `Lost`/`Stopped`/`Closed` into `nil`.** Real (a spin loop built on it cannot
  tell "slept" from "the world ended") and **not a rename.** **S34.**
- **`accept!`'s recursion depth.** TCO makes it a loop, not a stack risk. Not this stone's business.

## THE PROOF

1. **★ The rewrite is visible.** `git grep` shows no helper transforming its argument without saying
   so in its name.
2. **★ The bound reports.** Force `accept!`'s retry to expire; it must name depth, cap, attempts,
   elapsed. ⛔ **A bound that only says "gave up" is the empty ARM again.**
3. **★ Backpressure survives.** The circuit still completes `distinct=8000; dup=0`, five runs. If the
   bound trips in normal operation it is too short — that is a finding, and the fix is a longer
   bound, never a smaller queue.
4. **A `Lost`-is-ok arm gets its WHY.** `face-start-tw` treats `Lost` as success and `Stopped` as
   fatal. Whatever the reason, it is written down.
5. **The floor**, Summary line, `5213/5213`.
