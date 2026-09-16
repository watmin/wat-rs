# SCORE — every Status arm is named

**Struck 2026-09-15**, on `sns-sqs` at `0f91f036a`. The builder's item **1**, the macro-generated
half of the blindness class that piece A (`e03fc7f46`) proved send-side. One file, nine bespoke
sites, **no `src/` change**.

## ⚠ On method: hand-edits with the Edit tool were correct here, and why

`holon/CLAUDE.md` sends a **multi-site structural `.wat` migration** (a rename, a form flip across
many files) to the self-hosted **wat-fix codemod** and forbids python/sed outright. This stone is
neither: it is **nine bespoke, non-uniform edits in ONE file**, each one a different variant set
with a different success arm, a different `t0` sym and a different message. There is no rule to
record — a codemod would have to be nine special cases. So: the **Edit tool**, nine times, plus
three comment insertions. **No python and no sed touched `wat/service.wat`.** Python appears here
only as the *counting* instrument (below), which never writes.

## The floor, verbatim

```
     Summary [ 550.287s] 5251 tests run: 5251 passed (9 slow), 22 skipped
```

`./scripts/floor.sh`, **`.floor/2026-09-16T03-40-55Z/`**, `[floor] exit=0`, **no `ARM.txt`**.
550.3 s sits inside the 540.4–561.5 s same-code band the sibling SCORE recorded for this box.
No stray `release/wat` or `nextest` process before or after (checked both ways — a stray contends
for the box and spoils timing).

⭑ **This is the SECOND floor of the session, and the first one is why.** An earlier green —
`Summary [ 553.180s] 5251 tests run: 5251 passed (9 slow), 22 skipped`, `.floor/2026-09-16T03-27-38Z/`,
also no `ARM.txt` — weighed an **earlier version of the driven fixture**. `wat-scripts/` is inside
the `every_wat_scripts_file_loads` gate (the 553 s test; it *is* substantially the whole floor), so
growing the probe after that run left the green attached to a tree that no longer existed. Re-weighed.
The floor of record is the SECOND run, quoted above; `wat/service.wat` is byte-identical between the
two (md5 `06e19a4f5cee3cb668f1cadefbfa5c3f` both times — only the probe grew), and both runs are
reported so neither is hidden. The only thing that has changed since the second run started is **this
document**, which no gate reads.

Clippy `--release --workspace --all-targets`: **0** warnings, 0 errors.
`cargo nextest run --release --no-run`: clean. `cargo build --release`: clean.

## The ten rows

| # | what | result |
|---|---|---|
| 1 | All 9 Status sites name six variants; zero wildcards | ✅ **9 → 0** wildcarded. Whole-file: **14 → 5** wildcarded match forms |
| 2 | A second `Faulted` no longer raises | ✅ four second-recv sites return `GaveUp`; arm quoted below |
| 3 | Every remaining raise names the variant that arrived | ✅ 9 matches × 6 arms, **0** duplicated messages within any match — and one was **driven live** |
| 4 | Success path byte-identical | ✅ the only removed lines are the nine `(_ …)` arms (37 lines) + one trailing space |
| 5 | The three per-surface sites untouched and MARKED | ✅ marked — ⚠ **but there are FIVE wildcards left, not three**; see the correction |
| 6 | Floor | ✅ green, above |
| 7 | Happy path | ✅ `distinct=8000;dup=0` and `timeout=yes;discarded=yes;redial=Connected;retry-on=fresh` |
| 8 | clippy + tests compile | ✅ both clean |
| 9 | A DRIVEN second fault | ✅ **DRIVEN, all four sites, plus a negative control on the pre-stone binary** |
| 10 | Blast radius | ✅ `M wat/service.wat` + one new scratch probe. **No `src/`.** |

## Row 1 — counted by an instrument, and here is what that instrument can see

Neither `grep` nor eyeballing can do this count: the arm heads are unquoted symbols inside
quasiquoted macro bodies. The counter is a **balanced-paren reader** over the comment-stripped
source (`;` outside a string to end-of-line, string escapes tracked) that finds every
`(:wat::core::match …)` form, takes its arms **at depth 1**, and calls an arm wildcarded when its
head token is exactly `_`. It attributes nothing to an enclosing form. Whole-file, both revisions:

```
HEAD (0f91f036a)   match forms = 78   with a depth-1 `_` ARM = 14
working tree       match forms = 78   with a depth-1 `_` ARM =  5
```

The 9 that went away are, by line number **at HEAD**, exactly the DESIGN's table —
`1657 · 3145 · 3166 · 3235 · 3252 · 3323 · 3329 · 3381 · 3387` — reproduced independently, which
is the only reason to trust either number. Restricted to matches with a `~status-*-kw` arm head:

```
before  Status-naming match forms 9   wildcarded 9
after   Status-naming match forms 9   wildcarded 0
```

Nine before, nine after: no site was merged, split or lost. Each of the nine now has **exactly six
arms**, one per `Status` variant.

**What it cannot see:** it knows `~status-*-kw` only as a *spelling*. It does not resolve those
symbols to the `Status` defenum, so it would miss a Status match written with differently-named
binders, and it cannot tell a genuinely exhaustive parameterized match from an accidental one — the
same hole the DESIGN flagged in the census that found these. What makes the count load-bearing is
not the reader: it is **the checker**. Nine matches over a six-variant enum with no wildcard are
now exhaustiveness-checked, and every service in the corpus is generated by this macro, so a wrong
variant name is a broad red. The floor is the proof; the reader only says where to look.

## Row 2 — the behaviour change, quoted verbatim (`stop`, second recv)

```wat
                                        ;; ⭐ excursus 001, every-status-arm-is-named — THE
                                        ;; behaviour change. This is the SECOND recv, the one
                                        ;; D1-a's one-level drain added. A second queued
                                        ;; Status::Faulted used to fall into a `_` that raised
                                        ;; "expected Status::Stopped" — a LIE (a Faulted did
                                        ;; arrive) and a crash on a RECOVERABLE error. It now
                                        ;; returns a faced GaveUp naming what actually happened.
                                        ;; waited-ms comes from the method's own t0 — no second clock.
                                        ((~status-faulted-kw _cause)
                                          (:wat::service::StopOutcome::GaveUp
                                            (:wat::i64::/ (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) ~stop-t0-sym) 1000000)
                                            "a second Status::Faulted arrived — the fault stream outran the one-level drain"))
```

`~hib-t0-sym`, `~grant-t0-sym`, `~revoke-t0-sym` are the three siblings; grant/revoke build a
`GateOutcome::GaveUp` instead. **No second clock was minted** — each reads the method's own `t0`,
the same one `owner-recv-loop` measures against.

## ⭐ Row 9 — DRIVEN, on all four sites, with a negative control

`wat-scripts/scratch-pad/probe-two-faults-before-stop.wat`. A `defservice` whose `go` handler
raises when `n = 0`: the serve loop's `Outcome::Faulted` arm sends one `Status::Faulted` up the
lineage **and keeps serving**, so two calls queue two faults ahead of the owner's next ack. The
raise severs the calling conn (measured — the client sees `Lost`), so each boom dials its own.
A normal request runs first as the non-vacuity control. Post-stone, on the current binary:

```
"control=Message"
"stop-boom1=Lost"
"stop-boom2=Lost"
"stop     =GaveUp waited-ms=0 last=a second Status::Faulted arrived — the fault stream outran the one-level drain"
"hib-boom1=Lost"
"hib-boom2=Lost"
"hibernate=GaveUp waited-ms=0 last=a second Status::Faulted arrived — the fault stream outran the one-level drain"
"grant-boom1=Lost"
"grant-boom2=Lost"
"grant    =GaveUp waited-ms=0 last=a second Status::Faulted arrived — the fault stream outran the one-level drain"
"revoke-boom1=Lost"
"revoke-boom2=Lost"
"revoke   =GaveUp waited-ms=0 last=a second Status::Faulted arrived — the fault stream outran the one-level drain"
```

**The negative control is the half that makes this a proof.** The same fixture, unchanged, against
a binary rebuilt from **HEAD's `wat/service.wat`** (restored with `git checkout --`, rebuilt,
run, restored from a byte-identical copy — md5 verified both ways):

```
"control=Message"
"boom1=Lost"
"boom2=Lost"
#wat.kernel/AssertionFailure {:thread "main" :message "defservice stop: expected Status::Stopped" :location #wat.kernel/Location {:file "wat-scripts/scratch-pad/probe-two-faults-before-stop.wat" :line 103 :col 8} :actual nil :expected nil :frames [#wat.kernel/Frame {:file "wat-scripts/scratch-pad/probe-two-faults-before-stop.wat" :line 103 :symbol ":tf::svc/stop"} #wat.kernel/Frame {:file "src/freeze.rs" :line 1521 :symbol ":user::main"}] :upstream-chain nil}
```

⛔ The owner **died**, on a recoverable error, with the message the DESIGN called a lie — and the
word `Faulted` appears nowhere in it. That is the bug, reproduced on demand, then removed. (Quoted
whole, not windowed. `:line 103` is the *stop*-only draft of the fixture; the file has since grown
the hibernate/grant/revoke sites, so that line number no longer points at the `/stop` call — the
control was run before the probe was extended and has not been re-run against the old binary.)

`waited-ms=0` is honest: both faults were already queued, so the second recv returned without
waiting. The GaveUp does not claim a timeout — `last` says what ended it.

## ⭐ THE SURPRISE: the cleanup stop proved row 3 by accident

The first draft of the fixture called `(stop-faced (:tf::svc/stop h3))` after grant's GaveUp, to
avoid leaking a child. **It died** — and what it printed is the stone working:

```
"defservice stop: got Status::PeersAllowed (expected Stopped)"
```

`wat/service.wat`'s own grant comment **predicted this, in prose, before the stone existed**: *"a
surplus PeersAllowed ack left on the lineage peer is read by a later stop as Message(other) →
'expected Status::Stopped'."* Pre-stone that prediction was **unverifiable from the message**,
because the message named `Stopped` and said nothing about `PeersAllowed`. It now names the variant
that actually arrived, so the prediction and the observation finally agree — a live instance of row
3, on a path I was not trying to test. The raise itself is correct (rule 4: a genuine protocol
violation still raises) and the caller cannot face it (the raise happens *inside* the generated
`/stop`, before any `StopOutcome` exists, so `stop-faced` cannot catch it either), so the probe
simply does not ask; the comment in the fixture records why. That leaves grant's and revoke's
services without a stop — measured, not assumed: `ps -eo etimes,args | grep release/wat` after the
run shows **nothing** left behind.

## ⚠ THE CORRECTION: "the three per-surface sites" are three, but FIVE wildcards remain

EXPECTATIONS row 5 and DESIGN trap-door 5 say the three per-surface sites *"must still be exactly
three."* They are — `:2759` (`~reply-variant-kw`), `:2940` (`~accepted-ctor-kw`), `:3040`
(`~success-ctor-kw`) at HEAD numbering — and each now carries a comment saying the wildcard is
DELIBERATE and why: those variant sets are **per-surface and generated**, so "name every variant"
has no fixed meaning, and the `_` IS the desync detector.

**But the file holds FIVE wildcarded match forms after the change, not three.** `:2940` and `:3040`
are each the INNER match of a pair; the **enclosing** matches at HEAD `:2938` and `:3038` have their
own `_` arm, and they are over **`RecvOutcome`** —

```
   line 2978  other arm heads: ['(:wat::kernel::RecvOutcome::Message ~resp-sym)']
   line 3089  other arm heads: ['(:wat::kernel::RecvOutcome::Message ~resp-sym)']
```

⭑ `RecvOutcome` is a **FIXED** enum — `src/types.rs:1868` registers it as a builtin with exactly six
variants, `Message` · `Closed` · `Stopped` · `TimedOut` · `Lost` · `Malformed` (read, not recalled) —
so unlike the Reply sites these two **can** be named; the per-surface argument that
justifies the other three **does not apply to them**. They are the same blindness class, one enum
over, and the DESIGN's scope section did not name them, so they stand: **found, not fixed,
reported.** They are also NOT marked deliberate, deliberately — marking them would assert a reason
that isn't there. Note this is the mirror of the failure the sibling SCORE recorded: there, a
detector wrongly attributed a nested match to its enclosing form; here the enclosing form has a
defect of its own, and only a depth-aware reader separates the two.

## Row 4 — what the diff removed, exhaustively

`282 insertions(+), 37 deletions(-)`, one file. Every removed line, without exception: the **nine
`(_ …)` wildcard arms** (4 lines each = 36) **plus one** —

```
-                                  (:wat::core::match lu $
+                                  (:wat::core::match lu$
```

— the `extract-addr` match head, re-added **with its trailing space dropped**. That is the single
incidental change in the stone: whitespace, on a match head, not inside any arm. Stated rather than
glossed. **No success arm and no first-recv `Faulted` drain has a changed byte**, which is what
rules 1 and 2 required; `git diff -U1` shows those bodies only as context.

## Row 3 — and the limit of how it was checked

A second pass over the nine matches collected, per arm, the first string literal in that arm's own
body and looked for repeats within one match: **0 duplicated messages in any of the nine**. Sample,
`stop`'s first recv:

```
(~status-started-kw addr)        "defservice stop: got Status::Started (expected Stopped)"
(~status-hibernated-kw snapshot) "defservice stop: got Status::Hibernated (expected Stopped)"
~status-peers-allowed-kw         "defservice stop: got Status::PeersAllowed (expected Stopped)"
~status-peers-denied-kw          "defservice stop: got Status::PeersDenied (expected Stopped)"
```

⚠ **What that instrument actually reads** is "the first string in the arm", which for the *success*
and *drain* arms is not a message at all but `owner-recv-loop`'s `last` label (`"close"`, `"recv"`).
Those happen to differ within every match, so no duplicate was masked — but the column is not a
message column, and reading it as one would be wrong.

## What this stone does NOT do — unchanged from the DESIGN

- **No bound on the fault drain**, and ⛔ **I first wrote this bullet wrong — the correction is the
  point.** The draft said *"three booms would still kill the owner."* **False.** Read the six arms
  quoted above: the second recv returns `GaveUp` on the **first `Faulted` it sees**, so a third,
  fourth or thousandth queued fault changes nothing — recv is FIFO, the first recv takes fault #1,
  the second takes #2 and gives up, and the rest are never read. **Post-stone, no number of queued
  faults kills the owner.** What the stone does NOT fix is that the **ack is lost**: whenever ≥2
  faults precede it, `stop`/`hibernate`/`grant`/`revoke` return `GaveUp` and the owner cannot learn
  whether the operation actually took effect — the service may well have stopped, the gate may well
  have been applied. That residue belongs to the unsolicited-frame class (BREADCRUMB OPEN item 2),
  whose real fix separates notifications from the request/reply channel.
  ⚠ **The 3-fault case is NOT DRIVEN.** It is read off the arm set, not measured; driving it means
  another probe edit and therefore another floor run, and the DESIGN specified the 2-fault case,
  which *is* driven on all four sites. Saying so rather than quietly implying a third measurement.
  (The one way the owner still dies on a queued Status is a non-`Faulted` surplus — the
  `PeersAllowed` case above — and that raise is rule 4 working as specified.)
- **The `Faulted` cause is still dropped** at every arm that names it (`_cause`, unread). Carrying it
  into the raise message or the `GaveUp` `last` needs a **runtime** `:wat::string::interpolate` inside
  a quasiquoted body. ⚠ I did not find one in `service.wat` and I did not prove there is none: my
  check was a grep for lines carrying both `interpolate` and a `~` unquote (**0 hits** of 109
  `interpolate` occurrences, all of which appear to be macro-time), which is a weak instrument — it
  cannot see a call whose unquote sits on another line. Rather than ship a corpus-wide generated-code
  change on a weak reading, the cause stays dropped and this is named as a separate, checkable stone.
  `:wat::i64::/` by contrast IS proven in a generated body (`service.wat:2907`, pre-existing), which
  is why `waited-ms` was safe to compute there.
- **No `src/`.** The checker lint that makes this class impossible to reintroduce is item **3**.

---

## ⭑ REGRADED BY THE ORCHESTRATOR ON HIS OWN RUNS, 2026-09-15 — all rows hold

Every load-bearing row was re-taken independently rather than accepted:

```
floor    Summary [ 559.154s] 5251 tests run: 5251 passed (9 slow), 22 skipped
         .floor/2026-09-16T03-56-22Z/ · exit=0 · NO ARM.txt · inside the 540.4–561.5 s band (near its top)
clippy   0 · nextest --no-run clean
```

| row | the orchestrator's own result |
|---|---|
| 1 | ✅ my reader (comment-stripped, arms at depth 1) on `git show 0f91f036a:wat/service.wat` vs the tree: **wildcards 14 → 5**, **Status-wildcarded 9 → 0**, and **Status-naming matches still 9** — none lost, none invented. Remaining 5 at `:2785 · 2978 · 2980 · 3089 · 3091`. |
| 2 | ✅ **driven on all four**: `stop` · `hibernate` · `grant` · `revoke` each print `GaveUp waited-ms=0 last=a second Status::Faulted arrived — the fault stream outran the one-level drain`. |
| — | ⭐ **NEGATIVE CONTROL RE-TAKEN.** `git checkout 0f91f036a -- wat/service.wat`, rebuilt, **same probe**: exit **2**, `defservice stop: expected Status::Stopped`, `Faulted` nowhere in the message. Tree restored, md5 verified identical, rebuilt. The pre-stone lie is the orchestrator's measurement too, not a report he accepted. |
| 4 | ✅ `git diff --stat` = one file, **282 insertions / 37 deletions**, no `src/`. |
| 7 | ✅ `distinct=8000;dup=0` and `timeout=yes;discarded=yes;redial=Connected;retry-on=fresh`, 24 s. |

### The executor's surprise #2 is CONFIRMED, and deliberately FILED rather than folded in

`:2978` and `:3089` match `~r-sym` with arms `(:wat::kernel::RecvOutcome::Message ~resp-sym)` and `_`
— **`RecvOutcome` is a fixed builtin enum**, so the per-surface justification that covers `:2785`,
`:2980` and `:3091` does **not** cover these two. Same blindness class, one enum over.

⛔ **Not folded into this stone, and the reason is not scope-piety:** naming their five missing arms means
deciding what `Closed` / `Lost` / `Stopped` / `TimedOut` / `Malformed` should DO inside the generated
paging path — five behaviour decisions × two sites, in quasiquoted code. That is a stone, not a mechanical
port, and bundling it would have made this floor prove two things at once.

### Three things the executor did that are worth copying

1. **It refused to quote a green floor attached to a tree that no longer existed** — its first floor was
   green, then it changed the fixture; `wat-scripts/` is inside the load gate, so it re-weighed instead of
   reusing the number.
2. **It caught itself writing a placeholder floor number** before the run returned, replaced it with an
   explicit UNFILLED marker, and reported the near-miss.
3. **It corrected its own draft in public**: it had written *"three booms would still kill the owner"* —
   false, because the second recv returns `GaveUp` on the FIRST `Faulted` it sees, so no number of queued
   faults kills the owner. The real residue is the lost ack, and the 3-fault case is marked NOT DRIVEN
   rather than claimed.
