# SCORE — excursus 001 stone 6: Envelope into `:messages`

**STRUCK AS STOP-4.** Executor: grok, 2026-08-31. Envelope moved. The queue is green.
The floor is not, because the next Envelope-shaped hole is `:fanout::Outcome` on
`wat-scripts/fanout/circuit.wat`. **Circuit was not edited** (STOP-3). That is stone 7.

```
Summary [ 301.098s] 5126 tests run: 5125 passed (2 slow), 1 failed, 17 skipped
FLOOR=100
```

Log: `.floor/2026-08-31T03-02-37Z/`. ARM: `.floor/2026-08-31T03-02-37Z/ARM.txt`.
**Do not re-run.**

The move's own count, re-run, not taken from a report:

```
./target/release/wat wat-scripts/queue/sqs.wat
"bound=x;r1=a,b;r2=c;r3=;redel=b"
```

Byte-identical. `--check` on `sqs.wat` is **0**. `probe_ex001_queue` **PASS**.

## The scorecard

| # | what | expected | **measured** |
|---|---|---|---|
| 1 | `sqs.wat` freezes | `--check` = **0** | ✅ |
| 2 | ★ the summary is byte-identical | `"bound=x;r1=a,b;r2=c;r3=;redel=b"` | ✅ re-run, not a report |
| 3 | no rename | `:queue::Envelope` keeps its name; zero call-site edits | ✅ `git diff` is the block lifted into `:messages`; every `Envelope/id` / `Envelope/body` / ctor still spells `:queue::Envelope` |
| 4 | blast radius | `sqs.wat` only, plus SCORE | ✅ porcelain: `M wat-scripts/queue/sqs.wat` (10 lines, 5/5). Circuit **untouched** |
| 5 | `probe_ex001_queue` passes | lifecycle green | ✅ `PASS [0.849s] (3575/5126) wat::services probe_ex001_queue::queue_lifecycle_mem_and_sqlite_agree` |
| 6 | the loader gate passes | 528 files, 0 failures | ❌ 1 of 528: `circuit.wat` — STOP-4 |
| 7 | floor | **`FLOOR=0`** | ❌ STOP-4. One fail, not the queue |
| 8 | the guard still fires on the repros | both `repro/*.wat` still `--check = 1` | ✅ both EXIT=1, still name `:p::Item`. Stone 5 was not undone |
| 9 | prior stones | topic `"3 3"`; all `probe_ex001_*` PASS | ✅ `"3 3"`; delete / reput / write-opts / same-ns / sortkey / queue all PASS in this floor |

## STOP-4 — the floor cannot go green without editing circuit

```
FAIL [ 301.091s] (5126/5126) wat::lint wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime
```

Arm, `wat_scripts_fixes_load.rs:52`:

```
1 of 528 wat-scripts/ files do not load on the current runtime (rotted):
  wat-scripts/fanout/circuit.wat
      surface :fanout::Worker :messages type references :fanout::Outcome which is not
      declared in this surface's :messages — … Add a (defrecord :fanout::Outcome …)
      to :messages …  :location circuit.wat:70:3
```

`:fanout::Outcome` sits **beside** `:fanout::Worker` (`circuit.wat:30`).
`DrainResponse::Ok` carries `(:wat::core::Vector :- [:fanout::Outcome])` (`:73`).
**Same shape as Envelope.** Same guard, same message, same fix.

Stone 5's row 7 asked for this. After stone 5 the census was "none" — because
`circuit.wat` failed first on the *loaded* queue's Envelope, at `sqs.wat:41`, and
never reached Worker's own `:messages`. Moving Envelope unmasked the next one.
**Named, not fixed.** STOP-3 forbids touching anything outside `sqs.wat`.

## The move

`(:wat::core::defrecord :queue::Envelope …)` lifted from above the surface into the
front of `:messages`. Name kept. Placement: before `ReceiveResponse`, which
references it. Ordering did not matter in the sense that `--check = 0` with it
first; not probed in other slots.

First attempt closed `SendRequest` before its field vector (`UnexpectedRParen` at
the old `now-ns` line). Caught by `--check` before the floor. The landed diff is
the block move only.

## For stone 7 — two things in `circuit.wat`, change nothing here

**1. `:fanout::Outcome` is Envelope's twin.** A process worker that replies
`DrainResponse::Ok` of Outcomes will fail to freeze (now) and would have failed
the fork the same way Envelope did. The fix is the same move this stone just did
for Envelope. The guard's message already says so.

**2. The foreign-read workaround is still in `drain` (`circuit.wat:127-137`).**
Stone 4 wrote it so the child freeze would not see `:queue::Envelope/id`:

```
e    = first of the received envelopes
fr   = read-foreign (edn/write e)
eid  = format (ForeignRecord/get fr :id)
ebody = format (ForeignRecord/get fr :body)
ack by eid
```

Envelope now ships with `Queue`'s `:messages`. A forked worker can name
`:queue::Envelope/id` and `:queue::Envelope/body`. The workaround is unnecessary
and may be the `_ acc` swallow: `read-foreign` of a now-properly-typed Envelope is
a different value than `Envelope/id` of that Envelope, and any non-`Value` arm
assertion-fails, any failed ack is `_ acc`. Stone 4's `total=0` with `empty=0`
is consistent with swallowing. Nothing in this floor *runs* the circuit, so this
is unmeasured — noted so stone 7 does not rediscover it.

Stone 7 is the re-attempt of the fan-out proof against a substrate that can no
longer send it at Envelope. It still can at Outcome, and the drain impl still
does not use Envelope. Both are circuit, both are stone 7.

---

# The queue is green on purpose

Envelope is in `:messages`. The lifecycle summary did not change. The guard still
refuses the two reproductions. The remaining red is circuit, named, not this
stone's.

---

# ORCHESTRATOR GRADING — re-run, not read

```
queue --check = 0        "bound=x;r1=a,b;r2=c;r3=;redel=b"   ← byte-identical
direct repro   = 1       parametric repro = 1                ← ROW 8: the guard was NOT weakened
Summary [ 297.268s] 5126 tests run: 5125 passed (2 slow), 1 failed, 17 skipped   FLOOR=100
PASS (3574/5126) probe_ex001_queue::queue_lifecycle_mem_and_sqlite_agree
FAIL (5126/5126) every_wat_scripts_file_loads   ← circuit.wat, :fanout::Outcome
```

**STRUCK AS STOP-4.** The move landed, the count is byte-identical, and the floor did not return
to green — because moving `Envelope` **unmasked a second instance of the same defect**, not
because the stone failed.

## Row 8 is the row that mattered

Both reproductions still `--check = 1`. The obvious way to make the queue freeze was to move
`Envelope`; the wrong way was to weaken stone 5's guard, **and the floor would have looked
identical either way.** It was checked because EXPECTATIONS asked for it before the strike.

## ⛔ AND IT CORRECTS MY GRADING OF STONE 5

I graded stone 5's row 7 as **zero** — no second latent fork-failure — and built a three-part
instrument argument for why that was believable: `cargo build` covers stdlib, the loader gate
covers 528 files, the floor covers the tests.

**That argument is sound about FILES and wrong about DEFECTS.** `every_wat_scripts_file_loads`
reports which **files** fail; a type-check **halts at the first error**, so it does not enumerate
what else is wrong inside one. Two files failed at stone 5, and one was hiding a second defect
behind the first: `circuit.wat` failed on the loaded queue's `Envelope` and never reached
`:fanout::Worker`'s own `:messages`, where `:fanout::Outcome` sits beside the surface (line 30
vs 69) and is referenced as `(Vector :- [:fanout::Outcome])` — **`Envelope`'s exact twin.**

Corrected: the widening exposed **two** latent fork-failures, one stone apart, because the first
shadowed the second. `SCORE-stone-5-…` now carries the correction above its original section.

★ **Fourth occurrence of one shape today**: `--include=*.rs` excluding a `.md`; `Vector|vec`
missing `HashSet`; a comparison completed from memory rather than run; and now a file-level
census read as a defect-level one. The pattern is not carelessness about counting — it is
**asking "did the instrument report a failure?" instead of "what can this instrument not see?"**

The executor settled it the only way that actually settles it: **moved the first defect and
looked again.** That is a better instrument than any argument I made.

## Owed — stone 7 now has three jobs

1. **`Outcome` into `:fanout::Worker`'s `:messages`** — the same lift, third time.
2. **Remove the foreign-read workaround** (`circuit.wat:127-137`) — `edn/write` + `read-foreign`
   + `ForeignRecord/get`, written when `Envelope` could not cross. It now can. The executor
   flags it as *possibly the `_` acc swallow behind stone 4's `total=0`* — **unmeasured**.
3. **Re-attempt the fan-out proof**, against a substrate that can no longer send it the wrong way.
