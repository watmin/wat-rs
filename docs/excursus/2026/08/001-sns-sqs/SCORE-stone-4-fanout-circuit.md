# SCORE — excursus 001 stone 4: the fan-out circuit

**STRUCK AS STOP-5.** Executor: grok, 2026-08-31. The app was wired. Process workers
cannot consume wat-queue. **topic/ and queue/ were not changed.**

```
Summary [ 302.780s] 5122 tests run: 5122 passed (3 slow), 17 skipped
FLOOR=0
```

Log: `.floor/2026-08-31T02-02-44Z/`. No new floor arm — adding one that asserted N×M
would have been a red, and STOP-5 forbids changing queue to make it green. Floor stays
**5122**. `every_wat_scripts_file_loads` type-checked `circuit.wat` in this run.

Diagnostic run of the wired circuit (`N=12 M=2 J=2`):

```
n=12;m=2;j=2;total=0;distinct=0;dup=0;workers=0;empty=0
```

Messages **reach the queues** (`empty=0` — a leftover `receive` is not empty). Process
workers **return zero outcomes**. Duplicate count is unmeasured: the workers never
pulled.

## The scorecard

| # | what | expected | **measured** |
|---|---|---|---|
| 1 | it is a circuit | `:user::main` is wiring only | ⚠ wiring lives in `:user::run` so two weights share one diagram; main prints |
| 2 | fan-out completeness | N×M outcomes | ❌ `total=0` — STOP-5 |
| 3 | no loss | final receive empty | ❌ `empty=0` — leftovers sit in the queues |
| 4 | parallelism by ids | all M×J ids appear | ❌ `workers=0` |
| 5 | duplicate count | reported | ⚠ `dup=0` is vacuous — nobody received |
| 6 | one queue service per queue | J workers dial ONE service | ✅ the wiring does this (M queue Handles, J workers per Handle/addr) |
| 7 | workers are processes | `:locus process` | ✅ and that is what exposed STOP-5 |
| 8 | standalone at weight | N=2000 M=4 J=3 | not run at weight; scaled diagnostic is the finding |
| 9 | floor fixture scaled | same code, smaller N,M,J | `:user::compute` is `run 12 2 2`; no `.rs` arm |
| 10 | drives the shipped program | `startup_from_file`, no second copy | ✅ `load-file!` of `sns-fanout.wat` and `sqs.wat` |
| 11 | blast radius | fanout/ + one `.rs` + SCORE; topic/queue untouched | ✅ no `.rs`. topic/ and queue/ **untouched**. `git diff -- wat/ src/ crates/` empty |
| 12 | floor | 5122 + new arm | ⚠ 5122, no new arm (would be a known-red) |
| 13 | prior stones | probe_ex001_*, SNS `"3 3"`, queue summary | not re-floored this stone; SNS/queue files untouched |

## STOP-5 — wat-queue is not process-client complete

`ReceiveResponse::Ok` carries `(:wat::core::Vector :- [:queue::Envelope])`, but
`Envelope` is declared **above** the surface in `wat-scripts/queue/sqs.wat`, not
inside `Queue`'s `:messages`. A forked child of a `:peers [:queue::Queue]` service
ships the surface's messages. It does **not** ship `Envelope` or `Envelope/id`.

Measured:

1. Parent freeze of `circuit.wat` type-checks (Envelope is in the parent's world via
   `load-file!` of `sqs.wat`).
2. `fanout::worker/start :locus process` then dies at child StartupError:
   `unknown callee: :queue::Envelope/id`.
3. Workaround `edn/write` + `read-foreign` let the child freeze. Drain then returns
   **empty** (`total=0`) while the owner's leftover `receive` is **not** empty
   (`empty=0`). The child's `Queue/receive` is swallowing the reply (the `_ acc`
   arm) — the response cannot be used without `Envelope` in the child.

**What would unblock:** put `:queue::Envelope` in `Queue`'s `:messages` (or otherwise
ship field-types of message payloads to process-tier clients). That is a change to
`wat-scripts/queue/sqs.wat`. Named, not done.

Thread-tier workers would see Envelope (same world as `load-file!`). Row 7 forbids
that dodge.

## What DID work — composition without editing topic/queue

`(:wat::config::set-redef! true)` then

```
(:wat::load-file! "../topic/sns-fanout.wat")
(:wat::load-file! "../queue/sqs.wat")
```

Measured: without `set-redef!`, `DefRedefForbidden` on `:user::main` (sqs.wat:335).
With it, both programs load and this file's main wins. **The grep-promotion split
(feature vs the program that uses it) is what these two files still lack**;
`set-redef!` is the workaround that lets them compose today.

Adapter: `:fanout::adapter` `:satisfies :demo::Sub`, holds a Queue peer, `deliver`
is `Queue/send`. That is the missing wire. Topic still fans out to Subs. Queue still
send/receive/ack. The circuit plugs them.

## Topology (STOP-1 did not fire)

M `queue/start` Handles. Worker `qi,wi` is started with
`:queue-addr (Handle/addr queues[qi])`. J workers per queue, one service instance.
Not J queue services.

Grant-before-dial at this size: store grants queue; queue grants adapter; adapter
grants topic; queue grants each worker. SNS's 3-subscriber hook scaled to M via
foldl. That part composed.

## Parallelism / duplicates — unmeasured

Kick-then-recv (`kernel/send` Drain to every worker, then `recv` each) is the
concurrency plan — no sleep. It never ran against real messages. `dup=0` does not
count as "the actor serializes."

Visibility is `10^12` ns so a leftover would have been redelivery-vs-duplicate
**if anyone had pulled**. Trap-door 4 is armed in the summary shape
(`total` vs `distinct` vs `dup`) and unused.

## Placement

`wat-scripts/fanout/` is right: it composes topic and queue; it is not either of
them. Do not move topic/ or queue/.

## Blast radius

- `wat-scripts/fanout/circuit.wat` — the wiring + the measurement
- `wat-scripts/fanout/README.md`
- this SCORE

Zero `wat/`. Zero `src/`. Zero `crates/`. Zero edits to `wat-scripts/topic/` or
`wat-scripts/queue/`.

---

# ORCHESTRATOR GRADING — re-run, not read

```
Summary [ 299.177s] 5122 tests run: 5122 passed (2 slow), 17 skipped     FLOOR=0
wat-scripts/topic/sns-fanout.wat  → "3 3"
wat-scripts/queue/sqs.wat         → "bound=x;r1=a,b;r2=c;r3=;redel=b"
```

Summary reproduced **exactly**: `n=12;m=2;j=2;total=0;distinct=0;dup=0;workers=0;empty=0`.
Blast radius: `topic/` **0**, `queue/` **0**, substrate **0**. STOP-5 fired and was honoured.

**STRUCK AS STOP-5.** The circuit is wired; process workers cannot consume the queue; the cause
is named and not fixed. That is the stone delivering exactly what a STOP is for.

## The executor DID self-troubleshoot — the order is the interesting part

Its report names the mechanism completely: *"`:queue::Envelope` sits next to the service, not
inside `Queue`'s `:messages`. A forked child of a `:peers [:queue::Queue]` worker never gets
`Envelope/id`."* Correct, and it is the whole cause.

The sequence was **error → foreign-read workaround → still broken → diagnosis.** The workaround
step is the one worth explaining, and it is not an executor failure:

`UnresolvedReference` carries `path`, `span`, and `context: &'static str` — **a fixed phrase,
one of two literals** (`src/resolve/error.rs:13`). So the message can never carry anything
computed: not that this world was frozen from a service bundle, not which surfaces `:peers`
shipped, not that the same name resolves in the parent. What the child said, in full, was *"this
name is not a builtin or a registered function."*

**The correct inference from "the name isn't available here" is "make it available another
way."** That is the foreign read. The diagnostic did not mislead; it under-specified, and the
workaround follows from the under-specification.

★ The same file already shows the need being felt: the *other* literal reads *"macro call
survived expansion (expansion pass ran before this check?)"* — a **hypothesis about the cause**,
encoded because "unresolved" was not enough. But as a `&'static str` it cannot say which pass,
or why here. The need was recognised; the mechanism could not express it.

`docs/SUBSTRATE-AS-TEACHER.md` step 1 is literally *"Add a hint to the relevant error variant."*
The discipline exists. It was never applied to this error because nobody had walked this path —
`wat-queue` is the first userland surface with a userland type in its messages.

## Three things the executor did that are worth repeating

1. **It called `dup=0` vacuous.** That number is the property the stone existed to measure, and
   it looks green. Grok wrote *"dup=0 is vacuous"* rather than banking it.
2. **It named the fix and did not apply it** — `Envelope` into `:messages` is a change to
   `queue/`, and the brief said a needed change there is a finding.
3. **It named the escape hatch it declined**: *"Thread workers would dodge row 7."* Thread
   workers would have produced a green summary and proven nothing about process parallelism,
   which was the entire point.

## ⛔ MY VERIFICATION WAS CONFOUNDED

I tried to prove the diagnosis by moving `Envelope` into `:messages` on a scratch copy and
re-running. It went from silent-drain to `peer crashed` — **and that establishes nothing**,
because the circuit still carried the foreign-read workaround written for the *other* state. I
changed one variable with its compensator still installed.

The clean experiment is: move `Envelope` **and** remove the workaround, then run. **Untested.**
The NOTE records the fix as *named, not verified*.

## A second finding, seen but not diagnosed

That scratch run produced:

```
"peer crashed (abnormal far-side crash — no reason; the crash reason is administrative
 and travels only to the owner's crash channel)"
```

A forked child died and **its reason did not reach the caller** — the law
`probe_arc278_dead_child_speaks.rs` exists to enforce. Whether this is that defect or a sibling
on a different door is **unexamined**, and it is recorded because it was seen, not because it
was understood.

## The real finding

`NOTE-a-userland-peer-surface-must-carry-its-domain-types-in-messages.md`. Every existing
example is correct and **none exercises the failing combination** — the userland surfaces carry
only builtins, and the surfaces with real domain vocabulary are stdlib, where
`wat/query.wat:500`'s *"they cross via stdlib"* exemption genuinely applies. `wat-queue` copied
that shape without its precondition.
