# SCORE — the crash surface is enumerated

**SCORED.** Executor: grok, 2026-09-12, branch `sns-sqs`, HEAD `9d71dca47` (DRAWN). Did not commit.

```
     Summary [ 518.246s] 5237 tests run: 5237 passed (7 slow), 22 skipped
```

`.floor/2026-09-12T22-46-49Z/` — `scripts/floor.sh` exit **0**, **no `ARM.txt`**.
**Nothing was re-run to make anything go away.** `5237` unchanged — no deftest added.
`git diff --stat -- src/ wat/` is **EMPTY**. STOP-2 held: the census did not become the cure.

The artifact is `FINDING-the-crash-surface.md`. This SCORE is the headline and the grading map.

---

## ⭑ THE HEADLINE — 16 fire, 11 cannot, the store still cannot be crashed

27 failure variants / 10 enums, re-derived. Controls pass (RecvOutcome = 6, Closed present, Item → NextOutcome, two distinct ReadFrameOutcome). A **fifth** instrument defect: `Unit("X")` does not match `Unit("X".into())`.

**16 FIRES** with a verbatim command. **11 UNREACHABLE** with a named missing mechanism. **0 NOT SWEPT.**

The four priority enums (Recv / Send / Call / Stop): 13/14 fire. The miss is Send Closed — use-after-`close'`, and `close'` is kernel-restricted.

**§2d store-fault is UNREACHABLE from the harness.** Six live arms (`sqs.wat:780/814/845` send, `:1215/1248/1279` ack). Chaos `inbox-lost=0;inbox-closed=0;inbox-timedout=0`. `drop-recv-bp`/`drop-ack-bp` suppress the queue's reply to its caller. The store process is never signalled. Recv Lost/Closed/TimedOut fire on *other* peers; they have never fired on a **store call**.

**disrupt-bp does not provoke Lost/Closed.** The poison is an oversized `Seen/check`. That path now returns **Malformed** (`a-big=Malformed`). The worker's Malformed arm is the placeholder `assertion-failed!`. At `disrupt-bp=10000` the circuit **filled-stalls** (`arrived=0`). At 500 bp, 3 draws 0 fires, zeros on inbox-lost/closed/timedout.

Do not mint `:Unknown` until a store-fault injector exists. That would be a painted brick.

---

## WHAT DID NOT LAND (correct)

No `src/` or `wat/` edit. No injector built (STOP-2). No §2d ruling (the builder's).

Scratch probes under `wat-scripts/scratch-pad/probe-crash-surface-*.wat`, plus the pre-existing `probe-silent-stop-gaveup.wat`.

---

## GRADING MAP (EXPECTATIONS)

| # | expected | result |
|---|---|---|
| 1 | 27 / 10 | **27 / 10**, FINDING §0 |
| 2 | controls | RecvOutcome=6, Closed present, Item→NextOutcome, two ReadFrameOutcome. Fifth defect recorded. |
| 3 | every cell classified | no blanks; FIRES / UNREACHABLE only |
| 4 | every FIRES has a command | FINDING §1; three easy picks: recv-closed-lost, call-deadline, connect-refused |
| 5 | every UNREACHABLE names a mechanism | not "no test covers it" — close'-restricted, OnlyThisPeer, pidfd errno, store never signalled, … |
| 6 | §2d named | FINDING §4, six live lines, measured zeros |
| 7 | disrupt's reach | FINDING §3 — Malformed, not Lost/Closed; 100 % stalls fill |
| 8 | ranked injectors | FINDING §5, store-fault #1 |
| 9 | no substrate change | `src/` `wat/` empty |
| 10 | floor 5237 | `.floor/2026-09-12T22-46-49Z/` **5237 passed, 0 FAIL**, no ARM.txt |
| 11 | occurrence counts | `grep -o \| wc -l`, FINDING §1 |
| 12 | partial labelled | not partial |

---

## THREE FIRES TO SPOT-CHECK (row 4)

```
./target/release/wat wat-scripts/scratch-pad/probe-crash-surface-recv-closed-lost.wat
# recv-empty-child=Closed;recv-raise-child=Lost

./target/release/wat wat-scripts/scratch-pad/probe-crash-surface-call-deadline.wat
# call-by-deadline=DeadlineFired

./target/release/wat wat-scripts/scratch-pad/probe-crash-surface-connect-refused.wat
# connect-after-stop=Refused
```

Gate GaveUp takes ~10 s (`probe-crash-surface-gate-gaveup.wat`). Recv/Send Stopped need SIGTERM wrappers in the FINDING.

---

## RANKED NEXT STONE

**Store-fault injector.** It is the only cell that blocks a banked win and a builder ruling. Disrupt's Malformed arm is a one-line follow-up, not a substitute.

---

# ⭑ ORCHESTRATOR'S GRADING — claude, 2026-09-12

**Graded against my OWN reads and runs**, from `9d71dca47` + the working tree.

```
floor  .floor/2026-09-12T22-58-03Z/  Summary [ 513.930s] 5237 tests run: 5237 passed, 22 skipped
       0 lines matching ^ *(FAIL|TRY|TIMEOUT|ABORT|SIGSEGV) · no ARM.txt
tree   NO modified files — new files only. git diff --stat -- src/ wat/ EMPTY.
```

**STRUCK. 16 FIRES / 11 UNREACHABLE / 0 NOT SWEPT** — complete, where the BRIEF permitted partial.

## ⭑⭑ Row 4 — I ran six cells myself, not the three I promised

Four `FIRES` picked at random, plus the two `UNREACHABLE` cells that carry a probe which *tried and
failed* — those are the falsifiable ones, and an `UNREACHABLE` nobody can re-run is just an assertion.

```
recv-closed-lost      recv-empty-child=Closed;recv-raise-child=Lost
connect-refused       connect-after-stop=Refused
try-send-wouldblock   WouldBlock-after=376              (FINDING says 377 — queue depth, run to run)
frame-cap             a-small=Message/Ok;a-big=Malformed;b-other=Message/Ok;a-again=Closed;
                      send-torn=Lost;call-torn=Closed;call-big=Malformed
send-closed-thread    thread-recv-after-exit=Closed;thread-send-after-exit=Lost   ← Closed did NOT fire ✓
try-send-after-stop   see below
```

All true (unpiped) exit codes. Every cell behaved as the FINDING says.

## ⛔ One observation is a RACE, and the FINDING presents it as fixed

TrySend Closed's evidence column reads `try-send-after-stop=Lost`. My first run printed **`Sent`**. Five
runs:

```
Lost · Lost · Lost · Sent · Lost        4-of-5 Lost, 1-of-5 Sent
```

**The verdict survives** — `Closed` never appears on any run, so `UNREACHABLE` stands. But the cell is
race-dependent and the matrix does not say so.

★ **And the minority branch is a finding in its own right:** a `try-send` **and** a `send` to a peer that
was **cleanly stopped** return **`Sent`**, about one run in five — the stop has not been observed locally
yet, so the byte enters the local cell and reports success. A send reporting success to a peer that is
gone. Defensible as async semantics; undocumented, unstable, and exactly the class this campaign exists
to find. Not fixed here.

## ★★★ THE HEADLINE IS ROW 7, AND IT CLOSES A LOOP ACROSS THREE SESSIONS

`disrupt-hits`'s own comment (`circuit.wat:387`) says it counts *"the poisoned call came back
**lost/closed**: it TORE."* I verified the whole chain myself:

```
the poison      an oversized Seen/check — pad 10×200 — against :max-frame-bytes 256   (circuit.wat:102)
what it yields  Malformed   ← my own frame-cap run: a-big=Malformed
the arms        Message→"message"  Lost→"lost"  Closed→"closed"  TimedOut→"lost"
                Stopped→raise      Malformed→assertion-failed! "… UNMIGRATED PLACEHOLDER"
tore?           (or (= poisoned "lost") (= poisoned "closed"))     hits' increments only when tore?
```

⛔ **So `disrupt-hits` can NEVER increment.** The mechanism produces a variant the counter does not
count — and the arm it lands on **raises**, so the worker dies and the run cannot report anyway.

★★★ **Two independent reasons for the same zero, and the earlier session fixed only one.** That session
found the chaos gate inert at 200 bp, attributed it to **shared seeds** (P(no injection)≈0.65), fixed the
seeds, and added `disrupt-fires` precisely to separate *"never injected"* from *"injected and did
nothing"* — the comment at `:391` says so in as many words. With seeds fixed and the poison actually
sent, the answer is still zero, for a second reason nobody had looked for. **Fifth instance of
`[[feedback_the_first_failure_hides_the_rest]]`**, and the most expensive: a chaos knob has been shipped
and believed for two sessions.

★★ **And the arm that breaks it is one of the 61 placeholders the builder put LAST.** That is a measured
re-prioritisation signal, not an argument: at least one of those 61 is not cosmetic — it is the reason a
chaos injector is unusable. The FINDING's ranked item 2 ("fix disrupt's Malformed arm — one arm") is
therefore worth more than its position suggests.

## ⚠ The FINDING corrected MY stale line numbers

My DESIGN cited the §2d arms as `sqs.wat:745/779/810` and `1208/1241/1272`. **Those are wrong now** — I
copied them from the older SCORE instead of re-deriving against the disk, and `the-store-says-what-it-
deleted` shifted them. Verified: `:745` is an `Outcome::Continue`, `:1241` is a comment. The correct
lines are **780/814/845** and **1215/1248/1279**, which is what the FINDING says. ★ The line-number form
of *verify from the code, not from my sentence* — and the third time this session one of my own artifacts
was under-derived.

## ⚠ Row 11 — the arm counts are NOT reproducible, and that is my brief's fault

The FINDING says `RecvOutcome::Closed` = 585 arms. I get **584** over the same stated directories, 587
over the whole repo, 588 including the `.jsonl` carrier, 569 for `.wat` alone. **No glob I can construct
reproduces 585.**

**Immaterial to every verdict** — no cell's FIRES/UNREACHABLE depends on an arm count, and I reproduced
six verdicts directly. But the column cannot be checked, and the reason is mine: my BRIEF specified the
**method** (`grep -o … | wc -l`, never `grep -c`) and never the **scope**. ⭑ A census column must carry
its exact invocation, not just its technique.

## The rows

| # | row | my verdict |
|---|---|---|
| 1 | surface re-derived | ✅ **27** / **10**, matching my own independent count |
| 2 | instrument controls | ✅ `RecvOutcome`=6 · `Closed` present · `Item`→`NextOutcome` · two distinct `ReadFrameOutcome`. A **fifth** defect recorded (`Unit("X".into())`) — theirs, not a correction of mine; my pattern already stopped at the quote |
| 3 | every cell classified | ✅ 0 blanks, 0 `NOT SWEPT` — complete where partial was permitted |
| 4 | ⭑⭑ `FIRES` commands run by me | ✅ **six** cells, all behaved |
| 5 | `UNREACHABLE` names a mechanism | ✅ all 11 name one (kernel-restricted `close'` / `ReservedPrefix`; `peer_cred` io error; `OnlyThisPeer` pid mismatch; accept(2) io error; non-EINVAL/EBADF errno) — never "nothing drives it". Two spot-verified by running their failed probes |
| 6 | §2d cell explicit | ✅ and it **corrected my stale line numbers** |
| 7 | `disrupt`'s reach | ✅ established, and it is the headline — see above |
| 8 | ranked injector list | ✅ 6 entries, store-fault first, with the reason it blocks a banked patch |
| 9 | no substrate change | ✅ zero modified files |
| 10 | probes type-check | ✅ my own floor, 5237/5237, 0 FAIL |
| 11 | occurrences not lines | ⚠ method right, **scope unstated** — 584 vs 585, immaterial, my brief's defect |
| 12 | partial labelled | ✅ N/A — complete |

## What I'd credit above all

The `UNREACHABLE` cells carry **probes that tried and failed** — `send-closed-thread`,
`try-send-after-stop` — so the negative claims are falsifiable, and one of them promptly falsified part of
itself under my re-runs. A census whose negatives can be re-run is worth several that cannot.

## What this hands the next stone

**The store-fault injector, ranked #1**, unchanged. ⛔ And a second candidate that did not exist before
this sweep: **`disrupt`'s `Malformed` arm** — one arm, and until it is fixed, `disrupt-bp` is not a chaos
knob at any rate. `:Unknown` for §2d stays un-minted until the injector exists.
