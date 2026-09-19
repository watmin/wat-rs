# SCORE — the queue matures into the stdlib

**Struck 2026-09-16**, on `sns-sqs` at `f3639b6a0`. Stone 1 of 3. The library half of
`wat-scripts/queue/sqs.wat` is now `wat/queue.wat` at manifest position **50**, under
`:wat::queue::`. Two recorded codemods, one manual split, one manifest row. No behaviour change, no
brackets work.

> ⛔ **VERDICT: the work is done and correct; the FLOOR IS RED and this stone caused it.** Ten of the
> twelve rows are green, including all five rust probes, the load gate standalone, the byte-identical
> circuit, clippy, and both codemods' idempotence. Rows 6 and 11 are red: promoting a 1965-line,
> type-check-heavy file into the **frozen stdlib costs ~+30% on every wat world startup**, and two
> tests that were already within 8% and 13% of their nextest walls went through them. Nothing is
> broken — every assertion in both failing arms passed — but the suite no longer fits its clock.
> **Do not land on my say-so. The next section is the whole of it.**

# ⛔⛔ THE FLOOR IS RED, AND THE STONE CAUSED IT. READ THIS FIRST.

The floor went **RED on two arms, both TIMEOUT**, and the mechanism is **this stone**. It is not a
flake, not the box, not timing: it is bisected on three independent measurements and the arithmetic
matches the observation. ⛔ The stone should NOT land until the orchestrator rules on it.

## The floor, verbatim — TWO runs, the same red both times

**FLOOR OF RECORD — run 2**, on the final tree. `.floor/2026-09-17T04-41-40Z/`, **`ARM.txt`
PRESENT**, `[floor] exit=100`, WALL 702 s:

```
     Summary [ 610.016s] 5274 tests run: 5272 passed (13 slow), 2 timed out, 22 skipped
     TIMEOUT [  35.123s] (4611/5274) wat::value probe_rational_C5c_nan_unordered::c5c_nan_is_unordered_gate
     TIMEOUT [ 610.010s] (5274/5274) wat::lint wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime
```

**Run 1**, reported because it exists and because it is the *reason* there are two.
`.floor/2026-09-17T04-25-38Z/`, **`ARM.txt` PRESENT**, `[floor] exit=100`, WALL 610 s:

```
     Summary [ 610.015s] 5274 tests run: 5272 passed (14 slow), 2 timed out, 22 skipped
     TIMEOUT [  34.903s] (4610/5274) wat::value probe_rational_C5c_nan_unordered::c5c_nan_is_unordered_gate
     TIMEOUT [ 610.008s] (5274/5274) wat::lint wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime
error: test run failed
```

⚠ **Why run 1 does not stand alone, stated rather than buried:** I corrected a comment in
`wat-scripts/queue/sqs.wat` while run 1 was in flight (at roughly test 1466/5274). `wat-scripts/` is
inside the `every_wat_scripts_file_loads` gate, so that green-or-red was attached to a tree that no
longer existed. Run 2 was taken after every edit, and the only file that changed during run 2 is this
SCORE, which no gate reads — verified: all 31 code files sha256-identical before and after run 2.

⭐ **The re-run is NOT an attempt to make a red go away — it did not, and that is the point.** Two
independent floors, the same two arms, magnitudes within 0.6% (34.903 / 35.123 s and
610.008 / 610.010 s). **This red is deterministic and reproducible on demand.** No re-run of mine
destroyed any evidence; both `.floor/` directories and both `ARM.txt`s are on disk.

## The two arms, whole — verbatim from run 1's `ARM.txt`

(Run 2's are identical but for the two timings, and `finished in 35.11s` in place of `34.89s`.)

**ARM A** — `wat::value probe_rational_C5c_nan_unordered::c5c_nan_is_unordered_gate`:

```
     TIMEOUT [  34.903s] (4610/5274) wat::value probe_rational_C5c_nan_unordered::c5c_nan_is_unordered_gate
  stdout ───

    running 1 test
    test probe_rational_C5c_nan_unordered::c5c_nan_is_unordered_gate ... ok

    test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 190 filtered out; finished in 34.89s


    (test timed out)
```

The surrounding nextest lines for the same test, from `clean.log`:

```
          SLOW [> 15.000s] (─────────) wat::value probe_rational_C5c_nan_unordered::c5c_nan_is_unordered_gate
   TERMINATING [> 30.000s] (─────────) wat::value probe_rational_C5c_nan_unordered::c5c_nan_is_unordered_gate
       TIMEOUT [  34.903s] (4610/5274) wat::value probe_rational_C5c_nan_unordered::c5c_nan_is_unordered_gate
```

**ARM B** — `wat::lint wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime`:

```
 TERMINATING [>600.000s] (─────────) wat::lint wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime
     TIMEOUT [ 610.008s] (5274/5274) wat::lint wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime
  stdout ───

    running 1 test
    test wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime has been running for over 60 seconds

    (test timed out)
```

Note what ARM A's own stdout says: `test result: ok. 1 passed; 0 failed`, `finished in 34.89s`. The
assertions all held. **It was killed for taking too long**, at a `slow-timeout = { period = "15s",
terminate-after = 2 }` → 30 s wall (`.config/nextest.toml:274`). ARM B's wall is
`period = "300s", terminate-after = 2` → 600 s (`.config/nextest.toml:111`).

## ⭐ THE MECHANISM, BISECTED — the frozen stdlib is a per-startup tax

**`wat/queue.wat` is 1965 lines and it is now parsed and type-checked on EVERY wat world startup.**
Before the stone those 1910 code lines were parsed once per program that `load-file!`d sqs.wat.
Now they are in the binary, so every startup pays for them — and this suite starts worlds in the
thousands.

**Measurement 1 — A/B on the manifest row alone, same source, quiet box.** I could not `git stash`
(blocked), so I isolated exactly one variable: commented out the `wat/queue.wat` `WatSource` row,
rebuilt, timed a bare CLI startup on a one-line program, restored the row (byte-identical — sha256
`0190ef00…` before and after, and all 32 working files verified against a tar backup), rebuilt.

```
WITHOUT wat/queue.wat in the stdlib:  0.450 0.426 0.432 0.433 0.421 0.429   median 0.430 s
WITH    wat/queue.wat in the stdlib:  0.579 0.556 0.556 0.556 0.555         median 0.556 s
                                                                    Δ = +0.126 s  (+29%)
```

**Measurement 2 — a ONE-startup test, HEAD floor vs this floor, same box, one hour apart.**
`probe_rational_C5c_nan_unordered::nan_unordered_fixture_type_checks` is a single
`startup_beside(file!())` and nothing else:

```
.floor/2026-09-17T03-27-26Z  PASS [ 1.047s]   (HEAD)
.floor/2026-09-17T04-25-38Z  PASS [ 1.363s]   (this stone)      Δ = +0.316 s  (+30%)
```

**Measurement 3 — ARM A itself, which is 28 startups.** `c5c_nan_is_unordered_gate` calls
`call_beside_value` 27 times plus one fixture startup; each is a full world:

```
.floor/2026-09-17T03-27-26Z  PASS    [ 26.229s]   (HEAD — 3.8 s under its 30 s wall, 13% headroom)
.floor/2026-09-17T04-25-38Z  TIMEOUT [ 34.903s]   (this stone)
                       Δ = +8.674 s  (+33%)  ⇒  +0.310 s per startup
```

⭐ Measurements 2 and 3 are independent — a 1-startup test and a 28-startup test — and they agree on
the per-startup cost to within 2% (+0.316 vs +0.310 s). Measurement 1 is the same sign and the same
order on a different harness (CLI process vs in-process `startup_beside`) and a quiet box.

**ARM B falls out of the same number.** The gate is 687 startups (`find wat-scripts -name '*.wat' |
wc -l` = 687) and it is essentially nothing else: standalone it ran **375.03 s** = 0.546 s/file,
against a **0.556 s** measured bare startup. Its in-floor history across **25 consecutive floors** on
this box:

```
2026-09-16T00-18-31Z … 2026-09-17T03-27-26Z   gate 552.265 – 559.493 s   (25 runs, ALL green, 9 slow)
2026-09-17T04-25-38Z                          gate  >600 s → TIMEOUT 610.008 s   (14 slow)
```

⛔ The gate had been running at **~555 s against a 600 s kill — 45 s of headroom, 8%.** 687 × +0.126 s
is **+87 s**. It went through the wall. And the whole-floor Summary at HEAD *is* the gate's time
(552–559 s both), which is why the floor total tracks it exactly.

**The suite-wide signature agrees:** every one of those 25 HEAD floors reports `9 slow`. Run 1
reports `14 slow`, run 2 `13 slow`. Four to five extra tests crossed a slow threshold, which is what
a uniform +30% on every startup looks like from the outside — and the 13-vs-14 wobble is the only
part of any of this that moved between the two runs.

⛔ **NEITHER ARM IS A FLAKE AND I AM NOT OFFERING ONE AS AN EXPLANATION.** Both tests were within
13% and 8% of their walls at HEAD; this stone spends ~30% of startup; both crossed. The stone's
*correctness* is unaffected — every assertion in both arms passed — but the floor is red and the
cause is mine.

## ⛔ What I did NOT do about it, and why

- **I did not raise either timeout.** That is a policy change to `.config/nextest.toml` and it would
  make this floor prove two things — explicitly forbidden by the DESIGN's trap-door 6 and by
  EXPECTATIONS row 12. It is also the wrong instinct: the walls are doing their job.
- **I did not make stdlib startup lazy.** That is a substrate stone (parse/typecheck the stdlib once
  and cache, or defer per-file registration until first reference), it is `src/` work of real size,
  and it is nowhere in this stone's scope.
- **I did not un-promote the queue.** The builder ruled the promotion; the cost is a fact about the
  substrate that the promotion revealed, which is exactly what "used in anger" is supposed to
  surface.

⭐ **The forward-looking half, which matters more than this stone.** Stone 2 promotes
`wat-scripts/topic/sns-fanout.wat` — larger than sqs.wat's library half — and stone 3 moves
`bracket.wat`. On this measurement each promotion is another ~30%-of-startup tax multiplied by 687
in the load gate alone. **The stdlib-as-per-startup-tax has to be addressed before stone 2, or stone
2 cannot be weighed at all.** That is the finding I would most want read.

## ⚠ On method: what the codemod did and what it could not

`holon/CLAUDE.md` sends a multi-site structural `.wat` rewrite to the self-hosted wat-fix codemod and
forbids python/sed. This stone is squarely its case — **2035 occurrences of `:queue::` across 31
tracked `.wat` files** — and it went through two new recorded migrations:

- **`wat-scripts/fixes/queue-to-wat-queue.wat`** — the namespace rename, via
  `:wat::fix::rename-keyword-prefix`. 31 paths, one run.
- **`wat-scripts/fixes/drop-sqs-load-file.wat`** — deletes the now-pointless
  `(:wat::load-file! "../queue/sqs.wat")` top-level form, via `wat-grep-strip` over a
  two-child-list predicate. 17 paths, one run.

**No python and no sed wrote a single `.wat` byte.** Python appears twice, both times as an
instrument that only reads: the *oracle* for the rename diff (below) and the load-order analysis for
row 4. The `sed -n '<a>,<b>p'` in the split is a byte-range **extraction into new files**, proven a
partition by `cmp` (below) — not a rewrite of any tracked file; `sed -i` never ran on a `.wat`.

**The manual seam** — the split itself and the `src/load/stdlib.rs` row — is the same seam
`wat-scripts/fixes/rename-sourcefile-to-source-file.wat`'s header names: *"the def itself + its
stdlib registration are the manual seams the codemod cannot do."*

**Prose is a separate pass**, because the codemod walks the form tree and never touches a comment.
**19 comment lines** were hand-corrected across **15 Edit-tool calls** (14 lines in `.wat`, 1 in
`.rs`, the rest in the two new headers). See "the prose rot I did NOT fix" for what I left.

## The twelve rows

| # | what | result |
|---|---|---|
| 1 | Zero `:user::` names in `wat/queue.wat` | ✅ `grep -c ':user::' wat/queue.wat` → **0**. Not one name, and not one mention: see below |
| 2 | The five rust probes pass | ✅ **13/13**, named individually below |
| 3 | Recorded, dry-run-diffed, idempotent codemod | ✅ two of them; idempotence shown as 31/31 and 17/17 byte-identical sha256 on re-run |
| 4 | Load order holds | ✅ `verify-stdlib` → `[]`, **plus** an independent per-reference manifest-index analysis: max defining position **49** |
| 5 | `sns-fanout.wat:41` resolved | ✅ **deleted**, by codemod, in 16 files — not only sns-fanout |
| 6 | Floor | ⛔ **RED — 2 TIMEOUT, `ARM.txt` present, TWICE.** `Summary [ 610.016s] … 2 timed out` (`.floor/2026-09-17T04-41-40Z/`). Both arms whole + bisected mechanism at the top. Caused by this stone |
| 7 | Circuit byte-identical | ✅ all three deterministic lines byte-identical, `cmp`-level; below |
| 8 | Load gate | ⚠ **PASSES standalone (375.03 s, 9/9), TIMES OUT in the floor (>600 s).** Both reported; this is ARM B |
| 9 | What moved vs what stayed | ✅ name by name below — and I **disagree with the DESIGN on all 13** of its "CLIENT API" names |
| 10 | clippy + `--no-run` | ✅ clippy `--release --workspace --all-targets`: **0** warnings, 0 errors. `nextest --no-run`: clean |
| 11 | Cost as a BAND | ⛔ **OUT OF BAND.** Floor 610 s vs 540.4–561.5 s — and it did not finish, it was killed. Circuit 26 s vs ~23 s |
| 12 | No behaviour change, no brackets | ✅ stated explicitly below |

## ⭐ Row 9 — the split, name by name, and I disagree with the DESIGN's reading

The DESIGN split the 23 `:user::` names as **13 CLIENT API** (to be renamed into `:wat::queue::`) and
**10 TEST DRIVERS** (to stay). It also warned the reading was not gospel. **I moved 0 of the 13.**

### What moved to `wat/queue.wat` — 21 definitions

```
defsurface  :wat::queue::Queue                     the Peer surface — and the REAL client API:
                                                   its generated /send /receive /ack /stats methods
defrecord   :wat::queue::Envelope                  (+ /id /body)
defrecord   :wat::queue::Queue::SendRequest        :wat::queue::Queue::AckRequest
            :wat::queue::Queue::ReceiveRequest     :wat::queue::Queue::StatsRequest
defenum     :wat::queue::Queue::SendResponse       :wat::queue::Queue::ReceiveResponse
            :wat::queue::Queue::StatsResponse      :wat::queue::Queue::AckResponse
            :wat::queue::Queue::Wait               (:Immediate / :UpTo — the mode, not a magnitude)
defrecord   :wat::queue::Waiter · :wat::queue::Counters · :wat::queue::Stats
defstruct   :wat::queue::TakeAcc · :wat::queue::RetryAcc
defservice  :wat::queue::queue                     (+ generated ::Record ::Handle ::State /start)
defn        :wat::queue::queue::retry-put · retry-delete · ack-after-delete · send-after-put
```

1961 lines (51 header + 1910 code). Source lines 37–1946 of the pre-split file, verbatim.

### What stayed — all 23 `:user::` names, in three groups with three different reasons

**Group A — THE PANIC GATE (11 names): the DESIGN calls these client API. They are not.**
`dial-queue` · `dial-queue-peer` · `send` · `receive` · `receive-wait` · `ack` · `send-ok!` ·
`park-receive!` · `recv-envelopes!` · `read-call-counters` · `read-queue-counts`

Every non-happy arm of every one of them is `:wat::kernel::assertion-failed!`. Four reasons, in
order of how much I trust them:

1. ⭑ **The two real consumers already refused them.** `wat-scripts/fanout/circuit.wat`
   (189 `:wat::queue::` references) and `wat-scripts/topic/sns-fanout.wat` (194) call **zero** of
   these helpers — measured, not assumed:
   `grep -oE ':user::[a-z0-9!-]+' wat-scripts/{fanout/circuit,topic/sns-fanout}.wat` returns only
   their own gate names. They wrote their own outcome-facing dial/send/receive instead. Promoting a
   panicking gate to manifest position 50 would enshrine, in the stdlib, exactly the ungraceful
   failure this excursus is a crusade against — and the two programs that used the queue hardest
   would still not call it.
2. **The client API already moved.** `defsurface` generates `:wat::queue::Queue/send`,
   `/receive`, `/ack`, `/stats`, which hand back a `RecvOutcome` the caller must face. Those are in
   `wat/queue.wat`. Group A is a thin panic-wrapper *over* them, not a peer of them.
3. **The doctrine of this very excursus forbids the shape.** `a-call-outcome-cannot-lie`,
   `an-outcome-arm-cannot-be-a-wildcard`, `every-status-arm-is-named`: a library verb faces an
   outcome. These convert every outcome into a crash.
4. Measured reach: 10 of the 11 have **no caller outside `sqs.wat`**. The exception is
   `dial-queue`, called by two scratch-pad probes — which is why those two keep their `load-file!`.

⚠ **This is a judgement, not a measurement, and here is its cost if I am wrong.** If the builder
wanted an ergonomic `(:wat::queue::send q "q" "body" now)` in the stdlib, this stone did not ship
it, and shipping it later is a *new* verb written to face outcomes — not a rename of these. That is
strictly better than renaming a panic into the stdlib and having to un-ship it. The absence is
stated rather than glossed.

**Group B — NOT LIBRARY, FOR A DIFFERENT REASON (2 names).** `await-timer-ms` is a generic
timer-channel recv — the word "queue" does not occur in its body at all; if it earns promotion its
home is `wat/time.wat` or `wat/kernel/`, not a queue. `join-bodies` *does* take
`Vector<:wat::queue::Envelope>`, so the honest statement is not "it does not mention the queue" —
it is that what it **produces** is an assertion string (`"a,b,c"`), the exact shape the rows below
compare against. It is the test's formatter, not a queue verb. Both are called only by a driver in
this file; the DESIGN called both client API.

**Group C — THE DRIVERS (10 names), which the DESIGN and I agree on.**
`lp-send-wakes` · `lp-timeout` · `lp-fifo` · `lp-fewer-receives` · `lp-idle` · `lifecycle` ·
`depth` · `long-poll` · `compute` · `main`. Independent of judgement, these **cannot** move: they
name `:wat::query::mem-store` (manifest **51**) and `:wat::query::sqlite-store` (**52**), and a file
at 50 may not. The load-order wall settles this group by itself.

`wat-scripts/queue/sqs.wat` is now **392 lines** (51 header + lines 1948–2288 verbatim), down from
2288 — 83% of the file moved.

## Row 1 — zero, and zero *mentions* too

```
$ grep -c ':user::' wat/queue.wat
0
```

My first draft of the header spelled the prohibition out and so put **3** occurrences of the literal
prefix in the file. A gate greping for `:user::` would have flagged them. I rewrote the paragraph to
state the rule without spelling the token, so the bare grep is exactly zero and the invariant is
machine-checkable. Recorded because the near-miss is the row.

## Row 3 — the codemod, and the one thing that would have made it a silent no-op

### ⛔ The trap: `rename-keyword-prefix ":queue::" ":wat::queue::"` rewrites NOTHING

`:wat::fix::rename-valid-match?` (`wat/fix.wat:713`) rejects a match whose next character is an
identifier char. With old-bare `"queue::"`, the next char is always the first letter of the name —
`:queue::Queue` → `Q` — so **every** occurrence is rejected and the run reports `[renamed]` for all
31 files while changing nothing. The obvious spelling is the silent-zero-match failure mode
`holon/CLAUDE.md` warns about, wearing the codemod's clothes.

`":queue"` (no trailing colon) is worse than useless — it is **destructive**: `:queue` is a live
kwarg in this corpus (`(… :queue name :bodies …)`), it matches at-end with a valid left boundary,
and it would be corrupted into `:wat::queue`.

The prefix that works is **`":queue:"` → `":wat::queue:"`** — one trailing colon. The right boundary
is then the *second* `:` (not an identifier char) so every namespaced name matches, while the bare
`:queue` kwarg is one character too short to hold `"queue:"` and is therefore absent. Written into
the migration's header so the next reader does not rediscover it.

### The dry run, diffed against an independent oracle

Dry run on a `/tmp` copy of all 31 paths, then compared **not just to the original** but to a naive
`sed 's/:queue::/:wat::queue::/g'` oracle over the same files. 24 of 31 came back byte-identical to
the oracle. Every one of the 7 differences is a **comment line or a string literal** — never a code
form:

```
DIFFERS  wat-scripts/fixes/pending-to-visible.wat
  <           (:wat::fix::rename-keyword-exact ":queue::queue::State/in-flight" …
  >           (:wat::fix::rename-keyword-exact ":wat::queue::queue::State/in-flight" …
DIFFERS  wat-scripts/queue/sqs.wat
  <    ;; `:queue::queue::State` is rebuilt at 30 sites. These six are CHANGED at 1–2 of
  >    ;; `:wat::queue::queue::State` is rebuilt at 30 sites. These six are CHANGED at 1–2 of
```

That is the proof in both directions at once: on code the codemod did **exactly** the textual
rename and nothing else, and on strings/comments it did **nothing** — which is what protects the
five `wat-scripts/fixes/*.wat` recorded migrations, whose `:queue::` occurrences are string
arguments recording what the old name *was*. All five came back byte-identical:

```
IDENTICAL  wat-scripts/fixes/add-malformed-arm.wat
IDENTICAL  wat-scripts/fixes/add-timedout-arm.wat
IDENTICAL  wat-scripts/fixes/declare-queue-drop-knobs.wat
IDENTICAL  wat-scripts/fixes/pending-to-visible.wat
IDENTICAL  wat-scripts/fixes/wait-ns-to-wait.wat
```

`wat-scripts/scratch-pad/probe-nonzeroduration-crosses-the-wire.wat` also came back identical — its
two occurrences are comment-only. Applied to the real corpus: byte-identical to the dry run, all 31.

### Idempotence, shown not asserted

```
$ for f in $(cat paths.txt); do sha256sum $f; done > sha-after1.txt
$ ./target/release/wat ./wat-scripts/fixes/queue-to-wat-queue.wat < paths.edn   # second run
$ for f in $(cat paths.txt); do sha256sum $f; done > sha-after2.txt
$ diff sha-after1.txt sha-after2.txt
IDENTICAL — 31/31 files byte-identical, 0 bytes rewritten on re-run
```

Same for the second codemod: all **17/17** paths print `[unchanged]` on the re-run and all 17 sha256
sums are unchanged. (17, not 16: the path list was derived by grep and swept in the fix file itself,
whose header mentions the form in prose. It printed `[unchanged]` — an unplanned self-check that the
predicate does not match a comment.)

### The split's byte accounting

The split is the manual seam, so it is proven rather than trusted:

```
sed -n '1,36p'    → header      sed -n '37,1946p' → library code
sed -n '1947p'    → blank       sed -n '1948,$p'  → driver code
cat the four | cmp - <the pre-split file>
  → REASSEMBLY EXACT: the four ranges are a partition of the original bytes
```

Re-verified on the **final** files, after every edit, so the claim is about what is on disk now and
not about an intermediate state:

```
$ tail -n +58 wat-scripts/queue/sqs.wat | cmp - <pre-split lines 1948-2288>
  (identical — the driver half's code region is byte-for-byte the original)

$ diff <(tail -n +56 wat/queue.wat) <pre-split lines 37-1946> | grep -c '^[<>]'
8
  <    ;; `:wat::queue::queue::State` is rebuilt at 30 sites. …
  >    ;; `:queue::queue::State` is rebuilt at 30 sites. …
  <    ;; ⛔ Stats stays FLAT and 19 fields wide, deliberately: `:wat::queue::Stats/<field>` is
  >    ;; ⛔ Stats stays FLAT and 19 fields wide, deliberately: `:queue::Stats/<field>` is
  <       ;; NOT `receive-calls` — see the note on :wat::queue::Counters/recv-replies.
  >       ;; NOT `receive-calls` — see the note on :queue::Counters/recv-replies.
  <         ;; UNCHANGED flat `:wat::queue::Stats`. ⛔ Stats does NOT gain a nested `counters`
  >         ;; UNCHANGED flat `:queue::Stats`. ⛔ Stats does NOT gain a nested `counters`
```

⚠ **Four lines, not zero — and I would rather print the diff than the claim.** They are the four
prose fixes of the comment pass, which happen to live *inside* the code region rather than in the
header, so "the code region is byte-identical" would have been false. Every one is a `;;` comment.
**No form, no line of code, was retyped, reflowed or reindented.**

## ⭐ Row 4 — load order, verified two ways, neither of them "it compiled"

**(a) The stdlib's own verifier.** `(:wat::deporder::verify-stdlib)` reads every baked source and
returns the violations; it is the permanent gate behind
`tests/kernel/test_stdlib_load_order.rs::verify_stdlib_has_no_load_order_violations`.

```
$ ./target/release/wat wat-scripts/probes/arc-170/probe-deporder.wat
"[]"
```

**(b) An independent per-reference manifest-index analysis** — because (a) is the same instrument
the build uses, and one instrument is not two. I parsed `src/load/stdlib.rs` for the manifest order,
built a name→defining-file map from every `def*` form in all 56 baked sources, stripped comments
from `wat/queue.wat`, and resolved each of its `:wat::…` references (falling back to shorter roots
for accessors like `Stats/visible`):

```
refs resolving to a stdlib file at position >= 50 (MUST BE EMPTY):  (none)

defining-file positions used, all < 50:
  pos  1  wat/core.wat                     4 refs
  pos  2  wat/kernel/diagnostics.wat       1 ref
  pos  4  wat/seq.wat                      1 ref
  pos 18  wat/Record.wat                   1 ref
  pos 19  wat/program.wat                  1 ref
  pos 35  wat/service.wat                 10 refs
  pos 49  wat/query.wat                   53 refs   ← the reason it is 50, not earlier
```

Everything else (`:wat::i64::`, `:wat::time::`, `:wat::uuid::`, `:wat::rand::`, `:wat::set::`,
`:wat::vector::`, `:wat::edn::write`, `:wat::string::interpolate`, `:wat::enum::Pure`, most of
`:wat::kernel::`) resolves in **no** stdlib `.wat` — Rust intrinsics, which are order-free.

⚠ **A number in the DESIGN I could not reproduce, corrected in public.** The DESIGN says the file
needs *"wat/query.wat (manifest 49) — **147** references"*, and I wrote 147 into the new header
before measuring. Measured: **143** `:wat::query::` occurrences in code in `wat/queue.wat`, 146
counting comments, **53 distinct names**; the driver half has 28; both halves together 174. 147 is
neither. The *conclusion* — position 49 is this file's hard floor — is unaffected and independently
confirmed by the analysis above, but the header and the manifest comment now carry my count with
that note attached, because a number copied out of a design and re-published as a measurement is
exactly the failure mode that costs someone else a strike.

⚠ **What this instrument cannot see:** it is a regex over `def*` heads, so a name introduced by a
macro *expansion* in a later file would look like an intrinsic to it. That hole is exactly what (a)
covers, which is why both are reported. The two agree, and `pos 49` is the empirical answer to "why
50": the file's 53 distinct `:wat::query::` references make position 49 its hard floor.

**Position confirmed in the manifest, not claimed:**

```
48  wat/grep.wat        49  wat/query.wat        50  wat/queue.wat
51  wat/query/mem.wat   52  wat/query/sqlite-store.wat
```

## ⭐ Row 5 — what `sns-fanout.wat:41` became, and the DESIGN under-counted the problem

It is **deleted**. So are **15 more like it** — because `sns-fanout.wat` was not the only one.

⚠ **The DESIGN names one `load-file!`. I measured 18.** `wat-scripts/fanout/circuit.wat:41` does the
identical thing (and it is the freshness-probe program), and 16 scratch-pad probes do too:

```
$ grep -rl 'load-file!.*queue/sqs.wat' --include=*.wat . | wc -l
18
```

**16 dropped · 2 kept.** The two kept —
`wat-scripts/scratch-pad/probe-a-caller-chunks-to-the-declared-limit.wat` and
`probe-a-read-declares-its-page.wat` — are the only files in the corpus that call a `:user::` name
from sqs.wat (`dial-queue`), and that name stayed in the driver half, so their load is a genuine
dependency, not a fossil. Both already carry `(:wat::config::set-redef! true)`, so the status quo
around the shared `:user::compute`/`:user::main` names is unchanged.

**Why definitions do not arrive twice — and why the line still had to go.** After the split,
`wat-scripts/queue/sqs.wat` no longer *defines* the library at all, so a stale `load-file!` could
not double-define it; the "definitions arrive twice" hazard the DESIGN feared does not exist in
either direction. What a stale load *would* do is quieter and worse: it injects a foreign program's
**test drivers** — sqs.wat's `main`, `compute`, `depth`, `long-poll`, `lifecycle` and the five
`lp-*` rows — into the loading program's user rendezvous namespace, where they are masked only by
`set-redef!` plus the accident that the loader defines its own `main` later in the file. Deleting
the form removes a shadowing, not a dependency. **The topic stone (stone 2) inherits this
choice**: when `sns-fanout.wat` is promoted, its own loaders get the same treatment, and
`circuit.wat:40`'s `(:wat::load-file! "../topic/sns-fanout.wat")` is the line that goes then —
`drop-sqs-load-file.wat` is one string away from being reusable for it.

## Rows 7, 8, 10, 11 — the measurements

**Row 7 — the circuit, byte-identical.** `~/work/BREADCRUMB.md`'s FRESHNESS-PROBE one-liner,
`scripts/capped.sh --limit 8g`, run before the split and again after, on a quiet box:

```
before  "timeout=yes;discarded=yes;redial=Connected;retry-on=fresh"
after   "timeout=yes;discarded=yes;redial=Connected;retry-on=fresh"              BYTE-IDENTICAL
before  "queue-receive-calls=812"
after   "queue-receive-calls=812"                                                BYTE-IDENTICAL
before  "n=2000;m=4;j=3;total=8000;distinct=8000;dup=0;workers=12;empty=1;seen-recorded=8000;…"
after   "n=2000;m=4;j=3;total=8000;distinct=8000;dup=0;workers=12;empty=1;seen-recorded=8000;…"
                                                                                 BYTE-IDENTICAL
```

Compared with shell string equality on the whole line, not eyeballed. The fourth line (the timing
summary) differs run to run in its nanosecond fields, as it must; `distinct=8000;dup=0` lives on the
third line, which is identical in full.

**Row 8 — the load gate.** Run on its own, post-stone:

```
        PASS [ 375.030s] (9/9) wat::lint wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime
     Summary [ 375.040s] 9 tests run: 9 passed (1 slow), 5287 skipped
```

(That run also carried `every_tracked_wat_parses`, `every_ungated_wat_checks`,
`wat_record_from_sources_are_loaded`, and all four `verify_stdlib` tests — 9/9 green.) Inside the
floor the same test is ARM B. **Both facts are true and both are reported**: the corpus parses and
type-checks correctly, and it no longer does so inside the floor's 600 s wall.

**Row 11 — the cost, as a band.**

```
floor    610 s  ⛔ ABOVE the 540.4–561.5 s band — twice (610.015 s and 610.016 s) — and both runs
                  TERMINATED, so 610 is a LOWER BOUND, not the number: the gate was killed at its
                  600 s wall while still working. The 25-floor same-box band for this suite at HEAD
                  is 552.3–559.5 s; these are the first two runs outside it. Wall-clock for run 2
                  was 702 s (nextest's Summary clock stops at the kill).
circuit   26 s  ⚠ marginally above the ~23 s reference (BREADCRUMB says ~20–25 s). ABSENT: I did
                  not get a clean wall time for the BEFORE run in this session (`bc` and
                  `/usr/bin/time` are both missing on this host and my first attempt printed
                  nothing), so I cannot offer a same-session delta — only this run against the
                  breadcrumb's band. Saying so rather than implying a paired measurement.
gate     375 s  standalone, new datum — no prior standalone number exists to band it against.
startup  +30%   per world, three ways; the headline finding above.
```

## Row 2 — the five probes, named

Run explicitly, not inferred from the floor:

```
$ cargo nextest run --release -E 'binary(services) and (test(/probe_queue_long_poll/) +
    test(/probe_queue_depth/) + test(/probe_ex001_queue/) + test(/probe_async_publish/) +
    test(/probe_queue_visibility/))'

PASS [1.083s] ( 1/13) probe_queue_depth::queue_depth_counters_are_accurate
PASS [1.126s] ( 2/13) probe_queue_long_poll::queue_long_poll_gates
PASS [1.159s] ( 3/13) probe_ex001_queue::queue_lifecycle_mem_and_sqlite_agree
PASS [1.225s] ( 4/13) probe_async_publish::publish_returns_before_delivery
PASS [1.266s] ( 5/13) probe_async_publish::full_inbox_refuses_not_drops
PASS [1.288s] ( 6/13) probe_async_publish::unit_is_per_message
PASS [1.289s] ( 7/13) probe_async_publish::publish_ok_means_durable
PASS [1.303s] ( 8/13) probe_async_publish::publish_liveness_bound_reports_what_it_saw
PASS [1.353s] ( 9/13) probe_async_publish::idle_topic_never_ticks
PASS [1.535s] (10/13) probe_async_publish::stalled_subscriber_does_not_stall_others
PASS [1.810s] (11/13) probe_async_publish::refused_subscriber_is_retried_not_dropped
PASS [1.027s] (12/13) probe_queue_visibility::unacked_message_is_redelivered_after_visibility_expiry
PASS [5.883s] (13/13) probe_async_publish::outbox_term_removed_loses_messages
     Summary [   5.884s] 13 tests run: 13 passed, 192 skipped
```

⚠ **Three of the five are not passing for the reason their name suggests, and it matters here.**
`probe_ex001_queue`, `probe_queue_long_poll` and `probe_queue_depth` `startup_from_file` the driver
half directly and call `:user::compute` / `:user::long-poll` / `:user::depth` — those three genuinely
rendezvous with what stayed. The other two reach the queue *through* another program:
`probe_queue_visibility` drives `scratch-pad/probe-visibility-redelivers.wat`'s own `:user::compute`,
and `probe_async_publish` drives gates in `topic/sns-fanout.wat` and `fanout/circuit.wat`. Both of
those use `startup_from_source` + `FsLoader` **specifically because their entry program had a
relative `load-file!`** — which this stone deleted. They still pass, and the `FsLoader` is now
belt-and-braces rather than load-bearing for the queue (sns-fanout.wat still loads nothing;
circuit.wat still loads `../topic/sns-fanout.wat`, so it is still required there). Named so nobody
reads five greens as five direct contracts.

## Row 12 — no behaviour change, and no brackets

- **Not one code line changed except by namespace rename.** The library half's 1910 code lines are
  the pre-split file's lines 37–1946 byte-for-byte (`cmp`-proven above); the driver half's are lines
  1948–2288 byte-for-byte. The only other `.wat` edits in the whole stone are 16 deleted
  `load-file!` lines (one form each, codemod-verified) and 16 hand-corrected comment lines.
- **`wat/bracket.wat` is untouched** and nothing in this stone consumes it. No brackets work was
  begun, considered or bundled. That is stone 3/4.
- **`src/` is 2 lines of real change**: the 16-line manifest entry (13 of them comment) in
  `src/load/stdlib.rs`, and one doc-comment word in `src/macros/registry.rs`. No checker, runtime or
  type-system change, so the BOOTSTRAP / STASH-DANCE was not needed — a plain `cargo build --release`
  picked the codemod up.

## ⚠ THE SURPRISE: the rename is legal ONLY after the split, so there is no green in between

`:wat::` is a **reserved prefix** for user code (`src/check/error.rs:712`,
`crate::resolve::reserved_prefix_list`); only baked stdlib sources get the bypass
(`RegistrationPrivilege::Stdlib`). So the moment the codemod renamed `:queue::` → `:wat::queue::`,
`wat-scripts/queue/sqs.wat` was a userland file defining 21 names under a reserved prefix — a red
build — and it stayed red until the definitions landed in `wat/queue.wat` and the manifest row
registered it.

**I could not take an intermediate green, and I am saying so rather than implying a step-by-step
verified sequence.** The codemod step was verified on its own terms instead (dry run + oracle diff +
idempotence, all above), and the first build was taken after the split. This is not a flaw in the
plan — it is the reserved-prefix gate confirming the stone's own thesis: **the split is what makes
the rename legal.** A rename without a split is not a promotion, it is a violation.

## ⚠ TWO recorded migrations are now inert, by design

`wat-scripts/fixes/declare-queue-drop-knobs.wat` and `pending-to-visible.wat` carry their match
targets as **string literals** (`":queue::queue::Record"`, `":queue::queue::State/pending"`), so the
codemod correctly left them alone — and their predicates now match nothing in the corpus. That is
the right outcome for a completed, recorded migration (they had already been applied), and the
alternative — rewriting them — would falsify the historical record of what the name was when they
ran. Named here so a future reader does not read their silence as a bug.

## ⚠ THE PROSE ROT I DID NOT FIX — and the exact conversion for whoever does

The codemod walks the form tree, so comments are a separate manual pass. I did the **name** half:
16 comment lines across `wat/queue.wat` (4), `wat-scripts/fanout/circuit.wat` (6),
`topic/sns-fanout.wat` (1), two scratch-pad probes (5), `src/macros/registry.rs` (1) — a stale
`:queue::Stats` in a comment reads as live code, which is worse than a stale number.

I did **not** fix the **`sqs.wat:<line>` citations**. Measured after the stone: **17 of them, in 9
files** (`circuit.wat` ×8, `tests/services/probe_queue_visibility.rs` ×1, and seven scratch-pad
probes). They now point into a 392-line file at line numbers up to 1282, so they are
self-announcingly out of range rather than quietly wrong, and re-pointing them is a prose sweep of
its own that would have swollen this stone. The conversion, verified on four independent lines
(old 112→127, 153→168, 173→188, 1405→1420):

```
old sqs.wat:L   with L <= 1946  →  wat/queue.wat:L + 15
old sqs.wat:L   with L >= 1948  →  wat-scripts/queue/sqs.wat:L - 1896
```

Exact remaining set:

```
8  wat-scripts/fanout/circuit.wat
1  tests/services/probe_queue_visibility.rs
1  wat-scripts/scratch-pad/fix-circuit-to-sqlite.wat
1  wat-scripts/scratch-pad/probe-a-none-reply-is-a-promise.wat
1  wat-scripts/scratch-pad/probe-reply-drop-is-userland.wat
1  wat-scripts/scratch-pad/probe-store-write-rate.wat
1  wat-scripts/scratch-pad/probe-visibility-redelivers.wat
2  wat-scripts/scratch-pad/probe-what-a-process-impl-can-call.wat
1  wat-scripts/scratch-pad/probe-what-a-scan-costs.wat
```

(It was 20 before the stone; 3 of `circuit.wat`'s were re-pointed because they shared a line with a
stale NAME I was already fixing — those three are the only citations in the corpus that are now
correct, and they are correct because the fix was free, not because I swept.)

## Blast radius

```
28 files changed, 844 insertions(+), 2740 deletions(-)   + 3 new files
  src/load/stdlib.rs              +16          the manifest row at 50
  src/macros/registry.rs          1 word       a doc comment naming the surface
  wat/queue.wat                   NEW 1961     the library half
  wat-scripts/queue/sqs.wat       2288 → 392   the driver half
  wat-scripts/fanout/circuit.wat  189 renames + 1 load-file! dropped + 6 comments
  wat-scripts/topic/sns-fanout.wat 194 renames + 1 load-file! dropped + 1 comment
  22 scratch-pad probes           renames; 14 also lost their load-file!
  wat-scripts/fixes/queue-to-wat-queue.wat   NEW   recorded migration 1
  wat-scripts/fixes/drop-sqs-load-file.wat   NEW   recorded migration 2
```

## What this stone does NOT do

- **topic** — stone 2. Unblocked by this: `sns-fanout.wat` no longer loads sqs.wat, so the two
  programs are now coupled only through `:wat::queue::`, which arrives from the binary.
- **moving `bracket.wat`** — stone 3. Untouched.
- **an outcome-facing stdlib client gate** — the absence Group A above states out loud. The stdlib
  ships the surface's generated methods; nothing ergonomic sits over them. That is a stone, and
  writing it means deciding what each `RecvOutcome` variant should return — a behaviour decision,
  which is exactly what this stone may not make.
