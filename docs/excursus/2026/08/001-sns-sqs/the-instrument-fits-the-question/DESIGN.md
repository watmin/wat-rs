# DESIGN — the instrument fits the question

**Stone D.** The helpers stop lying — about what they consume, and about what they saw.

## WHY — this is the stone that kills the live race

`.floor/2026-09-03T09-14-58Z/` timed out at 30 s with an **empty ARM**. The mechanism is proven and
reproducible on demand, `probe-refused-retry-self-consumes.wat`, 3/3:

```
gap=0    recovered-after-naps=0    would-return
gap=300  recovered-after-naps=-1   SPINS-FOREVER
```

⛔ **It has passed three times since. It is not fixed.** A race whose failure mode is an
unfalsifiable hang is worse than one that asserts: the losing run leaves no evidence and every
winning run reads like a fix.

The mechanism, in one line: **`after-drain` asks "is anything here?" with a verb that takes it away.**

```wat
dummy-id    <- take-one subq        ;; drain the filler
ack-one subq dummy-id               ;; subq is now FREE — the worker can deliver
after-drain <- take-one subq        ;; "assert nothing is here" — DESTRUCTIVE, hides it ~1000 s
nap-ms 350
wait-pending subq                   ;; unbounded spin for the message just eaten
```

Two verbs, each doing the other's job:

| the question | the honest instrument | what it uses |
|---|---|---|
| *"nothing is here"* (absence) | **`stats`** — non-destructive, point-in-time | **destructive `receive`**, hides 1000 s |
| *"wait until something is here"* (presence) | **one `receive` with `:wait :UpTo`** | **spin on `stats`** at 1 ms |

And `:demo::q-depth` — the non-destructive instrument — was **already on the disk**, defined at
`sns-fanout.wat:462` and used by the very spin that hangs. Same shape as R69's `body-key`: the right
tool present, defined, and not connected.

## WHAT IT DELIVERS

**The ladder, applied honestly — shape where a wire event exists, check where none does.**

### 1. Presence → one blocking receive (the spin dies)

Two sites, both `wait-* then take-one` on the **same** queue:

- `sns-fanout.wat:798-799` — `wait-pending subq` → `take-one`
- `sns-fanout.wat:838-839` — `wait-pending q0` → `take-one q0`

Both collapse to **one `Queue/receive` with `:wait (:queue::Queue::Wait::UpTo …)`** — Stone B's arm,
arriving on the wire. No spin, no double-read, nothing to eat.

Proven already: `probe-refused-retry-self-consumes.wat`'s lockstep cells, `gap=300 → delivered;
raced=yes-and-VISIBLE`.

### 2. Absence → `stats` (non-destructive)

Two sites where `take-one`'s result is compared to `""` to mean *nothing here*:

- `sns-fanout.wat:796` `after-drain` — the site that hung
- `sns-fanout.wat:840` `stalled` — "must still be held"

Both become depth reads. The race then surfaces as **`visible = 1` failing an assertion that names
it**, instead of eating its own wakeup.

### 3. ⛔ The irreducible polls — bounded, and they REPORT

Not every wait has a wire event, and pretending otherwise is how the last attempt aimed at the wrong
rung:

- **`sns-fanout.wat:793` `wait-inflight inbox`** — waits on the **inbox**, then takes from **subq**.
  A *cross-queue* synchronization. There is no "block until another queue's unacked ≥ 1".
- **`circuit.wat:489` `wait-drained`** — a **conjunction** across N subscriber queues *plus* the
  topic's inbox term. No single event means "all of these are empty".
- **`circuit.wat:515` `wait-pending-zero`**, **`sns-fanout.wat:444` `wait-inbox-zero`** — waiting for
  a count to reach **zero**. Departure has no arrival event.

These stay polls **and must therefore be bounded and must report what they last saw** — depth,
attempts, elapsed. This is the genuine content of the superseded 3b, correctly placed: it is the
*check* rung, taken only where the *shape* rung is unavailable.

★ `circuit.wat:486` already declares its own unboundedness — *"No attempts bound — if this hangs,
the drain condition is wrong."* That comment does not exist on `wait-pending` or `wait-inflight`, **the
two that actually hung.**

### 4. The names stop lying

| now | becomes | why |
|---|---|---|
| `take-one` | **`claim-one!`**, visibility a **required argument** | it consumes and hides for a caller-supplied window; the signature must show it. `!` per `docs/CONVENTIONS.md:516` — *"`!` is for forms that would otherwise read as pure"* |
| `wait-pending` / `wait-inflight` | **`poll-until-visible` / `poll-until-unacked`** | they poll; and `wait-pending` (≥1) reads as the opposite of `wait-pending-zero` (=0) in the same loaded program |
| `pending` / `in-flight` | **`visible` / `unacked`** | `sqs.wat:69` already defines them with those words. 16 consumers |

### 5. A failure must not read as a reading

`q-depth` returns **`(Tuple 1 1)`** on `Lost`, `Closed`, or malformed — a perfectly ordinary queue
state that **satisfies both wait predicates**. A dead peer reads as work present.

The same family, all in reach of the spins that read them:

| site | on failure | consequence |
|---|---|---|
| `sns-fanout.wat:469,470` `q-depth` | `(1, 1)` | satisfies both waits |
| `sns-fanout.wat:433,434` `depth-of-topic` | `1` | feeds `wait-inbox-zero` → spins forever |
| `circuit.wat:468,469` `topic-outbox` | `1` | same |
| `sns-fanout.wat:441,442` `ticks-of` | **`-1`** | ★ **already honest, four lines away** |
| `sns-fanout.wat:145-147` | comment says *"Do not invent a depth we did not read"*, next line returns `Ok 1 0` | the comment lies about the line under it — **on the redial-recovery path** |

One convention, out-of-band, stated once. `ticks-of` shows the file already knew.

## ⛔ THE ONE CONTRACT DECISION

**No helper may use `receive` to prove a negative.**

Absence is asked with `stats`; presence is asked with a blocking `receive`. A `receive` whose result
is compared against emptiness *to conclude nothing is there* is the defect, and after this stone it
must not exist in the helper vocabulary.

That rule is checkable, and it is the whole mechanism: a destructive read used as an observation is
what let a test eat the message it then waited for.

## FILES

`wat-scripts/topic/sns-fanout.wat` · `wat-scripts/fanout/circuit.wat` ·
`wat-scripts/queue/sqs.wat` (the `visible`/`unacked` surface) · three scratch-pad probes.

**Census: 67 helper occurrences across 5 files** (`take-one` 17, `wait-pending` 12, `q-depth` 12,
`depth-of` 9, `wait-inflight` 6, `wait-pending-zero` 4, `wait-inbox-zero` 4, `wait-drained` 3), plus
16 `StatsResponse::Ok` consumers.

⛔ **`.wat` corpus migration → `wat-fix` codemod** for the mechanical renames; the semantic changes
(presence→blocking, absence→stats, the bounds, the sentinels) are hand work in two files.
`wat-scripts/fixes/wait-ns-to-wait.wat` is the freshest recorded shape.

★ **This census is mine and my last four were each wrong in a different way** — omitted
constructors, an omitted directory, an empty grep reported as fact, a miscount of my own list.
**The finder's count is the fact; mine is a hypothesis.**

## OUT OF SCOPE = REJECTED — with homes

- **`accept!`** — doesn't accept; publishes, retries unboundedly on `Full`, and `circuit.wat:508`
  **rewrites the payload** with a timestamp. Real, and a different reason to change. **Stone D2.**
- **`face-start-tw`**, the `do-receive`/`do-receive-wait` merge, `nap-ms`'s six homes. **Stone D2.**
- **`Alarm :delay`, `Milliseconds`.** **Stone C**, last, closes no defect.
- **Chaos (3c/3d).** Next after this — and it *cannot be read* until this lands, which is why the
  order is what it is.

## THE PROOF

1. **★ The reproducer must flip.** `probe-refused-retry-self-consumes.wat`'s `gap=300` cell can no
   longer produce `SPINS-FOREVER`, because `wait-pending` will not exist. The probe is committed and
   currently reproduces the hang on demand — it is the acceptance criterion, already on disk.
2. **★ The race becomes an assertion, not a hang.** With a 300 ms gap induced, the test must **fail
   loudly naming the race**, or pass — never stall. Show which, with the message.
3. **A bounded wait reports.** Force one to expire and show it names *what it last saw*. A bound that
   only says "timed out" is the empty ARM again.
4. **No `receive` proves a negative.** The contract decision, grep-shown.
5. **A dead peer is distinguishable.** Kill a queue mid-wait; the helper must report a failure, not a
   depth.
6. **The circuit invariant.** `distinct=8000; dup=0`, five runs, publish **25.5–27.4 s** (the band
   widened by my own row-6 finding on Stone B, which recorded one run at 27388 ms).
7. **The floor**, Summary line, `5213/5213`.
