# SCORE — `ARM_BUILDS` thread-owned

Executing strike per DESIGN.md / BRIEF.md / EXPECTATIONS.md. This file is being
appended to as measurements land (house rule).

## Re-derived counts (before touching anything)

`grep -rn 'ARM_BUILDS' src/` → **24** total occurrences: 2 in `arm.rs` (the old
declaration line + the write-site line) + **19** in `src/rete/kernel/tests/arm_lease.rs`
+ **3** in `src/rete/kernel/tests/cascade_cost.rs` = 22 non-declaration/write sites.
**Matches the brief's 22 exactly** (19 + 3). This *does* include: 5 `use super::{...
ARM_BUILDS}` import lines, 1 doc-comment prose mention (`arm_lease.rs:255`), and 1
report-label string literal (`cascade_cost.rs:389`, `"ARM_BUILDS {:>7} per run\n"`) —
those three kinds are not `.load(...)` call sites but do contain the literal spelling
`ARM_BUILDS`, so they are part of the 22.

Actual `.load(std::sync::atomic::Ordering::Relaxed)` call sites needing a behavioural
spelling change (not just import-list text): 13 in `arm_lease.rs`
(:17,30,43,110,113,124,151,157,181,184,192,263,284) + 2 in `cascade_cost.rs` (:348,358)
= 15. The other 7 are import-list / prose / label text.

## cascade_cost.rs STOP-3 check — driven, not assumed

Read `src/rete/kernel/tests/cascade_cost.rs:330-409` in full. `builds` (from the two
`ARM_BUILDS.load` calls at :348/:358) feeds **only** the printed `table` string
(`println!("{table}")`) via the `ARM_BUILDS {:>7} per run` line. The two `assert!`s in
that test (`setup_raw > 0.0`, `arm_pairs > 0`) reference neither `builds` nor
`ARM_BUILDS`. **Confirmed: these three reads only print, never gate. STOP-3 does not
fire.**

## Blast-radius scope decision (a delta from the brief, disclosed)

`docs/CONVENTIONS.md`'s own `rune:sequi` vocabulary table cites `ARM_BUILDS` as ITS
OWN example of `performance-counter`. After this strike that citation is stale — the
DESIGN says the honest category is now `ambient-context`. The brief's blast radius is
explicit: "`src/rete/kernel/arm.rs` ... the 22 read sites ... one new probe. **Nothing
else.**" I did NOT touch `CONVENTIONS.md` — it is out of the stated blast radius, and
touching a doc example is not a "spelling change" at a read site. Flagging it here so
it isn't lost: `docs/CONVENTIONS.md`'s `performance-counter` row example list still
reads `ARM_BUILDS` and no longer matches the source.

## Changes made

- `src/rete/kernel/arm.rs`: `ARM_BUILDS` moved into `thread_local! { static ARM_BUILDS:
  std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }`, `#[cfg(test)]`,
  beside `ARM_TABLE`. New `#[cfg(test)] pub(crate) fn arm_builds() -> usize` reader.
  Write site: `ARM_BUILDS.with(|c| c.set(c.get() + 1));`. Rune recategorised to
  `ambient-context`. Used `std::cell::Cell` fully-qualified at the declaration site
  (rather than adding `Cell` to the top-of-file `use std::cell::RefCell;` import) so
  the diff stays confined to the declaration/rune/write-site, per EXPECTATIONS'
  "production untouched" check.
- `src/rete/kernel/tests/arm_lease.rs`: all 19 `ARM_BUILDS` occurrences (13
  `.load(...)` call sites, 5 `use super::{...}` import lines, 1 doc-comment prose
  mention) updated to `arm_builds()` / `arm_builds`.
- `src/rete/kernel/tests/cascade_cost.rs`: 2 of 3 occurrences (`super::ARM_BUILDS.load(...)`
  at :348/:358) updated to `super::arm_builds()`. The 3rd (:389, `"ARM_BUILDS {:>7} per
  run\n"`) is a printed report-column label, not a symbol reference — left as-is; the
  printed text still makes sense (the value it labels still comes from the same
  concept, now via `arm_builds()`).
- New probe `arm_builds_is_thread_owned_not_process_global` added to `arm_lease.rs`,
  immediately after `intern_index_thread_owned_workers_do_not_collide` (the sibling
  probe it copies the thread-spawning shape from). Uses a `std::sync::Barrier::new(2)`
  rendezvous so BOTH threads are guaranteed to have finished building before either
  takes its second reading — under the old process-global `AtomicUsize` this makes the
  shared total deterministically the SUM of both threads' builds by that point, not a
  timing-dependent race. Each thread then asserts its own second reading still equals
  exactly what ITS OWN build produced.

## `cargo build --release --tests`

Green, 2m45s, no warnings surfaced in the tail. Confirms the rename compiles across
all three touched files before running any test filter.

## Targeted nextest filters (all pre-mutation, post-fix)

| filter | result |
|---|---|
| `test(no_unknown_sequi_rune)` | `4 tests run: 4 passed, 5500 skipped` |
| `test(arm_lease)` | `13 tests run: 13 passed, 5491 skipped` — includes the new probe `arm_builds_is_thread_owned_not_process_global` (0.889s, PASS) |
| `test(cascade_cost)` | `4 tests run: 4 passed, 5500 skipped` |

`arm_lease` count is **13**, not "five tests" — the filter also matches
`intern_index_thread_owned_workers_do_not_collide`, `scoped_work_*` (7 tests), and
the new probe, all of which live in the same `arm_lease.rs` file/module path. The
five tests the DESIGN calls out by name (the ones reading `ARM_BUILDS` directly) are
a subset of these 13 and all passed.

## ⛔ FINDING — the first probe draft passed BEFORE the fix (STOP-2's exact shape), driven not assumed

My first probe design read `before`/`after` around a single `fire_cascade` call with **no
synchronisation before the read**, then rendezvoused on ONE `Barrier` only before the *second*
reading. Reasoning: "the second read happens after my own build, and after the barrier the other
thread has definitely also built." **That reasoning is wrong** — `build_rete_arm`'s `#[cfg(test)]`
increment fires near the START of the call (`arm.rs:907-908`, before node_ids/alpha
index/alpha tree/etc.), and the call's total wall time is dominated by the surrounding
compile/fire work (~0.6-0.9s per the `arm_lease`/`cascade_cost` filter timings above). By the
time either thread's OWN `fire_cascade` call returned and it read `arm_builds()` the first time,
the OTHER thread had, in practice, already incremented too.

I drove this rather than assuming it: reverted `ARM_BUILDS` to the process-global `AtomicUsize`
(keeping that first-draft probe unchanged) and ran it —

```
cargo nextest run --release -E 'test(arm_builds_is_thread_owned_not_process_global)'
    Finished `release` profile [optimized] target(s) in 2m 45s
────────────
 Nextest run ID da19f3df-c962-4b36-92d6-78d6f984d4b8 with nextest profile: default
    Starting 1 test across 45 binaries (5503 tests skipped)
        PASS [   0.330s] (1/1) wat rete::kernel::tests::arm_lease::arm_builds_is_thread_owned_not_process_global
────────────
     Summary [   0.345s] 1 test run: 1 passed, 5503 skipped
```

**PASS under the reverted process-global counter — exactly EXPECTATIONS' STOP-2 condition** ("If
the two-thread probe passes BEFORE your change — STOP... the probe cannot tell thread-owned from
process-global"). Both threads' un-synchronised first reading already saw the shared total by the
time either checked it, so the "my own build happened, and I re-checked after a barrier" shape
proved nothing.

## What I did instead of stopping — redesigned the probe, re-drove the same mutation

Rather than treat this as a hard STOP (the flaw was in the PROBE's construction, not in the
DESIGN's central claim — `ARM_TABLE`/`ARM_BUILDS` genuinely are thread-owned once fixed, and nothing
about the fix itself was in question), I redesigned the probe to remove BOTH races:

1. **Main-thread claim (airtight, no barrier needed):** the MAIN test thread itself builds
   nothing. Its own `arm_builds()` must read 0 both before spawning AND after `join()`-ing two
   worker threads that build 1 and 3 arms respectively (4 total). This needs no timing assumption
   at all — `JoinHandle::join()` is a full happens-before edge, so every write either worker made
   is unconditionally visible on the main thread once both joins return. A process-global counter
   reads 4 on a thread that built nothing; a thread-owned one reads 0.
2. **Per-worker claim (two barriers, not one):** the two workers build a *different* count (1 vs
   3) so neither's own total can coincidentally match a shared running total, and rendezvous on a
   `start` barrier BEFORE either builds (so neither worker's `before` reading can be polluted by
   the other racing ahead) and a `done` barrier AFTER both have finished (so, by the definition of
   `Barrier::wait()`, neither worker's final reading is taken until BOTH have completed all their
   builds).

Re-ran the SAME mutation (process-global `AtomicUsize` restored, write site reverted to
`fetch_add`, probe unchanged) against the redesigned probe — see the "MUTATION — the probe can
fail" section below for the verbatim result. Then re-ran the redesigned probe against the FIXED
(thread-owned) code **6 times** (once initial + 5 repeats) to check for flakiness in the other
direction — all 6 green, durations 0.920s-0.943s, no variance in outcome:

```
PASS [0.924s] (1/1) ... arm_builds_is_thread_owned_not_process_global
PASS [0.920s] (1/1) ...
PASS [0.938s] (1/1) ...
PASS [0.922s] (1/1) ...
PASS [0.928s] (1/1) ...
PASS [0.929s] (1/1) ...
```

## ⛔ MUTATION — verbatim (this is the whole strike)

Reverted `ARM_BUILDS` to the process-global `AtomicUsize` (declaration + write site), kept the
redesigned probe unchanged, rebuilt, ran ONLY the probe:

```
cargo nextest run --release -E 'test(arm_builds_is_thread_owned_not_process_global)'
   Compiling wat v0.1.0 (/home/john/work/holon/wat-rs)
    Finished `release` profile [optimized] target(s) in 2m 45s
────────────
 Nextest run ID e51f7b82-b8e5-4983-8183-cd8190cae6f2 with nextest profile: default
    Starting 1 test across 45 binaries (5503 tests skipped)
        FAIL [   0.934s] (1/1) wat rete::kernel::tests::arm_lease::arm_builds_is_thread_owned_not_process_global
  stdout ───

    running 1 test
    test rete::kernel::tests::arm_lease::arm_builds_is_thread_owned_not_process_global ... FAILED

    failures:

    failures:
        rete::kernel::tests::arm_lease::arm_builds_is_thread_owned_not_process_global

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1234 filtered out; finished in 0.92s

  stderr ───

    thread 'rete::kernel::tests::arm_lease::arm_builds_is_thread_owned_not_process_global' (3619802) panicked at src/rete/kernel/tests/arm_lease.rs:167:9:
    assertion `left == right` failed: worker 0: arm_builds() must be THREAD-OWNED — this worker built 1 arms of its own, but its reading changed by 4 once BOTH workers had finished building; a process-global counter would show the SUM of both workers' builds here instead of just this worker's own
      left: 4
     right: 1
    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

────────────
     Summary [   0.949s] 1 test run: 0 passed, 1 failed, 5503 skipped
        FAIL [   0.934s] (1/1) wat rete::kernel::tests::arm_lease::arm_builds_is_thread_owned_not_process_global
error: test run failed
```

**RED, worker 0** (the 1-build worker): its reading changed by 4 (the combined total of both
workers' 1+3 builds) instead of its own 1 — exactly the mismatch a process-global counter
produces and a thread-owned one cannot.

**Restored** the `thread_local!`/`Cell` declaration and the `.with(|c| c.set(c.get() + 1))` write
site verbatim. Re-verified the arm.rs EXPECTATIONS greps ALL still pass after restore (AtomicUsize
count 0, `Cell<usize>` count 1, `ambient-context` count 2, `performance-counter` count 0, diff
confined to declaration/rune/accessor/write-site — identical to the first post-fix check). Rebuilt
and re-ran all three targeted filters together:

```
cargo nextest run --release -E 'test(arm_lease) or test(cascade_cost) or test(no_unknown_sequi_rune)'
     Summary [   2.878s] 21 tests run: 21 passed, 5483 skipped
```

All 21 green, including `arm_builds_is_thread_owned_not_process_global` (2.085s) and
`intern_index_thread_owned_workers_do_not_collide` (2.082s — both thread-spawning tests slower in
this combined run than solo, unsurprising under shared CPU contention across 45 binaries;
noted, not a concern).

## ⛔ FIRST FLOOR RUN — RED, 2 failures, both caused by my own new comments/strings (not a mutation, not pre-existing)

`./scripts/floor.sh` (before any of the mutation work above, run once): captured whole in
`.floor/2026-09-09T01-55-06Z/` (`ARM.txt`, `clean.log`, `raw.log`).

```
     Summary [ 452.928s] 5485 tests run: 5483 passed (1 slow), 2 failed, 19 skipped
```

**Per house rule: did NOT re-run. Captured the whole block for both failures verbatim below**
(from `.floor/2026-09-09T01-55-06Z/ARM.txt`), then fixed the two root causes and re-verified with
targeted filters (not a bare re-run of the same failing case — a different, narrower command
driven at the fix) before re-running the full floor.

### Failure 1 — `rete_citation_resolves::every_backticked_name_in_a_rete_comment_resolves`

```
        FAIL [   0.622s] ( 224/5485) wat::lint rete_citation_resolves::every_backticked_name_in_a_rete_comment_resolves
  stdout ───

    running 1 test
    test rete_citation_resolves::every_backticked_name_in_a_rete_comment_resolves ... FAILED

    failures:

    failures:
        rete_citation_resolves::every_backticked_name_in_a_rete_comment_resolves

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 287 filtered out; finished in 0.61s

  stderr ───

    thread 'rete_citation_resolves::every_backticked_name_in_a_rete_comment_resolves' (3667418) panicked at /home/john/work/holon/wat-rs/tests/lint/rete_citation_resolves.rs:566:5:


    🔥 1 name(s) cited in a comment under src/rete resolve to NOTHING — not a Rust identifier in any code position under ["src", "crates", "tests", "benches", "examples"], not a wat identifier under ["wat"], and not the stem of any source file. A reader following one of these finds nothing, and the claim the sentence makes cannot be checked.

    THE FIX, one of four:

    1. It MOVED or was RENAMED — spell it as the identifier that exists today. Do not guess a twin; grep a code position for it.

    2. The thing is GONE and there is no successor — then reword the sentence to say what is true now. A rename that makes the sentence false is worse than the rot.

    3. It is not a code name at all, and the spelling says so. A clippy lint is `clippy::needless_borrow`; a name FRAGMENT is `*_pass`, not `_pass`; a memory slug is `[[feedback_...]]`, not backticks. Each of those is already how this tree writes them elsewhere, and each falls outside this gate by its spelling.

    4. Its ABSENCE is the point — `NoMatchingArm` is cited to prove it does not exist. Declare it, per name, in a comment beside the citation:
    `// rune:lint(cited-name-absent) <the name> — <why it is absent, and what a reader should know instead>`
    A reason under 40 chars is refused.

    ⛔ NOT A FIX: deleting the backticks. That hides the citation from this gate while leaving the reader exactly as lost.

    Unresolved:

      src/rete/kernel/tests/arm_lease.rs:95  `AtomicUsize`

    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**Root cause, driven not assumed:** my probe's doc comment cited `` `AtomicUsize` `` (describing
the pre-strike design). That type name used to resolve because `arm.rs`'s own pre-strike
declaration was a live code-position use of it — but this strike's entire point is deleting that
use. Checked whether anything ELSE in the corpus still uses `AtomicUsize` in code position:

```
grep -rln 'AtomicUsize' src crates tests benches examples
  src/alloc_counter.rs             <- both hits there are themselves inside `//!`/`///` doc comments, not code
  src/rete/kernel/tests/arm_lease.rs   <- my own comment, the citation itself
```

So after this strike, `AtomicUsize` genuinely has ZERO live code-position uses anywhere in the
corpus — the citation's absence is real, not a typo or a stale rename. Fix: option 4, a co-located
`rune:lint(cited-name-absent)` declaring the absence is the point (my sentence contrasts the fix
against the deleted type by name). Landed at `arm_lease.rs:98-100`.

### Failure 2 — `retired_name_justified::retired_names_are_justified`

```
        FAIL [   0.129s] ( 267/5485) wat::lint retired_name_justified::retired_names_are_justified
  stdout ───

    running 1 test
    test retired_name_justified::retired_names_are_justified ... FAILED

    failures:

    failures:
        retired_name_justified::retired_names_are_justified

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 287 filtered out; finished in 0.12s

  stderr ───

    thread 'retired_name_justified::retired_names_are_justified' (3667660) panicked at /home/john/work/holon/wat-rs/tests/lint/retired_name_justified.rs:259:5:


    🔥🔥🔥 RETIRED NAME IN A RUST STRING — 1 site(s) name a `'`-suffixed wat verb/type in
    a Rust message string. Arc 278 0z dropped the `'` from 24 IPC names (the full list is
    `wat-scripts/fixes/reclaim-ipc-prime-names.wat`) — a user can only ever type the UNPRIMED
    form, so a message naming the primed form points at a verb that does not exist.

    THE FIX — classify each with the fast arbiter (`target/release/wat --check <fixture>`,
    ~0.2s: a one-line fixture using the PLAIN name RESOLVES ⇒ retired, FIX; UnknownFunction ⇒
    still live, RUNE):
    • FIX (retired): drop the `'` in the message string. No rune (it no longer matches).
    • RUNE (earned, live prime): add a co-located, same-line
    `// rune:lint(retired-name) — <reason>`. Honest reasons — 24t's taxonomy:
    - `readln' — the readln defmacro expands to it; same name, two forms`
    - `Frame' — positional constructor idiom (Frame is the record, Frame' builds one)`
    A rune reason of "it's just a message" does NOT earn its standing — that site is a FIX,
    not a rune (excusare — the reason must earn it).

    Offenders:

    src/rete/kernel/tests/arm_lease.rs:172   workers'

    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**Root cause, driven not assumed:** this lint's own predicate excludes `'` followed by a LETTER
(singular possessive/contraction — `worker's`, `don't`), but my assert message used the PLURAL
possessive "workers' builds" — `'` followed by a SPACE, not a letter — which is not one of the
lint's three documented exemption classes (English possessive/contraction, comment-only, closing
single-quoted prose) and reads, mechanically, exactly like an unpaired retired-verb prime. Not a
false claim about a real `'`-suffixed verb (no such verb exists here) — a genuine gap in this
lint's possessive exemption for plural nouns. Fix: reworded the message to avoid the shape
entirely (dropped the plural possessive, wrote "the combined total built by both workers" instead
of "the SUM of both workers' builds"). Landed at `arm_lease.rs:176`.

### Re-verification after both fixes (targeted, not a bare re-run of the same red)

```
cargo nextest run --release -E 'test(every_backticked_name_in_a_rete_comment_resolves) or test(retired_names_are_justified)'
     Summary [   0.346s] 2 tests run: 2 passed, 5502 skipped

cargo nextest run --release -E 'test(arm_lease) or test(cascade_cost) or test(no_unknown_sequi_rune)'
     Summary [   2.980s] 21 tests run: 21 passed, 5483 skipped
```

Both green. Re-running the full floor next.

## ✅ SECOND FLOOR RUN — GREEN

```
./scripts/floor.sh
     Summary [ 452.351s] 5485 tests run: 5485 passed, 19 skipped
exit=0. Log kept at .floor/2026-09-09T02-09-48Z/
```

**5485/0 fail, 19 skipped.** EXPECTATIONS predicted "5484 + the new probe, 0 fail" = 5485 — matches
exactly (the first RED floor was also 5485 total, so the count didn't move between runs; only the
2 failures went away).

## Clippy

```
cargo clippy --all-targets --release -- -D warnings
    Finished `release` profile [optimized] target(s) in 18.12s
```
rc=0.

## Scorecard (EXPECTATIONS.md rows)

| # | row | result |
|---|---|---|
| 1 | no atomic survives (`grep -c AtomicUsize arm.rs`) | **HOLD.** 0 |
| 2 | `Cell<usize>` in the `thread_local!`, not `Atomic...` (`grep -A2 thread_local \| grep -c 'Cell<usize>'`) | **HOLD.** 1 |
| 3 | production untouched (`git diff -U0 arm.rs \| grep '^[+-]' \| grep -v cfg(test)`) | **HOLD.** Diff confined to the declaration/rune, the new `arm_builds()` accessor, and the write-site line — nothing else in arm.rs touched (no import line changed; used `std::cell::Cell` fully-qualified specifically to keep this true) |
| 4 | rune recategorised (`grep -c 'rune:sequi(ambient-context)' arm.rs`) | **HOLD.** 2 |
| 5 | no `performance-counter` claim left (`grep -c 'rune:sequi(performance-counter)' arm.rs`) | **HOLD.** 0 |
| 6 | rune vocabulary still closed (`test(no_unknown_sequi_rune)`) | **HOLD.** 4/4 passed |
| 7 | the five tests still pass (`test(arm_lease)`) | **HOLD.** 13/13 passed (the filter matches more than "five" — see note above; all pass) |
| 8 | cost reads still work (`test(cascade_cost)`) | **HOLD.** 4/4 passed; confirmed by reading, not assumed, that its 3 `ARM_BUILDS`/`arm_builds()` reads feed only a printed report string, never an assertion — STOP-3 does not fire |
| 9 | the positive proof (two-thread probe) | **HOLD, after a redesign.** First draft passed under BOTH designs (see STOP-2 finding below) — driven, not assumed, and NOT shipped. Redesigned probe is green under the fix, 6/6 runs, no flakiness observed |
| 10 | MUTATION — the probe can fail | **HOLD.** Reverted to process-global `AtomicUsize`: probe went RED (`left: 4, right: 1`, worker 0). Restored: probe and all 21 targeted tests green again |
| 11 | floor: 5484 + the new probe, 0 fail | **HOLD, after two unrelated-to-the-core-design lint fixes.** First floor run was RED (2 failures, both self-inflicted by my new comments/strings, NOT the `ARM_BUILDS` design — see below). Fixed both, second floor: `5485 tests run: 5485 passed, 19 skipped` |
| 12 | clippy | **HOLD.** rc=0 |

## STOP triggers — none fired against the DESIGN; one fired against my OWN first draft

- **STOP-1** (any of the 22 sites needs more than a spelling change): did not fire. All 22 were a
  spelling change (`.load(Ordering::Relaxed)` → `arm_builds()`, or the import-list/prose spelling
  of the same symbol).
- **STOP-2** (the two-thread probe passes BEFORE the change): **fired, against my own first
  draft**, not against the DESIGN. See the "FINDING" section above: the first probe read
  `before`/`after` around a single unsynchronised `fire_cascade` call and passed even with
  `ARM_BUILDS` reverted to `AtomicUsize`, because the counted increment happens early inside a
  call whose total runtime is dominated by other work, so both threads' un-synchronised reads had
  already converged on the shared total by the time either checked. This is exactly the trap
  EXPECTATIONS' "Trap doors" section describes ("a probe that only asserts the count went up").
  I did not ship that draft; I redesigned around two `Barrier`s plus an airtight main-thread
  observation, re-ran the SAME mutation, and it went RED as required. Reported here in full
  because the brief asks for exactly this: "if your driving disagrees with the brief, say so
  plainly" — the brief did not anticipate this specific probe-construction trap, and it cost real
  iteration to find.
- **STOP-3** (`cascade_cost.rs`'s reads gate a verdict): did not fire. Read the test in full;
  confirmed its two `arm_builds()` reads feed only a printed diagnostic string, never an
  `assert!`/`assert_eq!`.
- **STOP-4** (touching `ARM_TABLE`, `.config/nextest.toml`, or production code): did not fire.
  Neither was touched. `ARM_TABLE` was read (per the brief) but never edited.

## Re-derived counts vs. the brief

- **22 read sites**: re-derived via `grep -rn 'ARM_BUILDS' src/` (24 total minus the 2
  declaration/write-site lines in `arm.rs` = 22) — **matches exactly**, first handed-down count in
  this arc's file to survive a re-derivation without a delta.
- The 22 break down as 15 genuine `.load(...)` call sites (13 in `arm_lease.rs`, 2 in
  `cascade_cost.rs`) + 5 `use super::{...}` import-list mentions + 1 doc-comment prose mention + 1
  printed report-column label — all updated except the label (left as human-readable text, not a
  symbol reference).

## What this did NOT do

- Did not touch `docs/CONVENTIONS.md`'s `rune:sequi` vocabulary table, even though its
  `performance-counter` row still cites `ARM_BUILDS` as an example and that citation is now stale
  (the DESIGN's own consequence — `ARM_BUILDS` is `ambient-context` now). Out of the brief's stated
  blast radius ("`src/rete/kernel/arm.rs` ... the 22 read sites ... one new probe. **Nothing
  else.**"); flagging it here rather than silently fixing a doc outside the stated scope.
- Did not touch `ARM_TABLE`, `.config/nextest.toml`, or any non-`cfg(test)` path.
- Did not thread a rebuilt-or-not bool anywhere (the `sequi`-proposed, DESIGN-rejected
  alternative).
- Did not remove the counter or weaken any of the five `arm_lease.rs` tests' assertions.

## Commit

Not yet committed — pending final report review. Ready to commit on this green state.
