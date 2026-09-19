# SCORE — composed-path coverage for `every_wat_bad_fixture_actually_fails.rs`

Executing strike per `DESIGN.md`. Written as-I-go per house rule.

## Re-derivation, before any edit

Read `DESIGN.md` in full, then the gate file
(`tests/lint/every_wat_bad_fixture_actually_fails.rs`) in full including its ten existing unit
tests, then the underlying cast report DESIGN itself summarizes
(`docs/arc/2026/06/278-rules-engine/vigilia-2026-09-07-rete/reports-target4/peragrare.md`) and
the FINDINGS.md `4P1` row it replies to.

**Confirm DESIGN's central claim**: six of the seven exemption states DO have a per-component
unit-test proof, mapped correctly:

| state | driven by (component-level) |
|---|---|
| `absent` | `prose_naming_the_words_is_not_a_rune` (`declaration_on` returns `None`) |
| `bad-category` | `an_invented_category_is_read_not_skipped` (`declaration_on` reads an unknown category rather than silently skipping it) |
| `no-owner-field` | `a_reason_with_no_owner_field_names_nobody` (`owner_in` returns `None`) |
| `owner-absent` | `a_test_that_is_not_there_is_absent` (`owner_state` returns `Absent`) |
| `owner-live` | `a_running_test_reads_as_live` + `an_ignore_on_a_different_test_does_not_vouch_for_this_one` (`owner_state_in` returns `Live`, including the upward-scan boundary case FINDINGS names as the sharpest risk) |
| `owner-ignored` | `an_ignored_test_reads_as_banked` (`owner_state_in` returns `Ignored`), plus 3 real corpus fixtures driving it through the FULL composed path |

⚠ **`bad-category`'s proof is weaker than the table implies.** `an_invented_category_is_read_not_
skipped` only proves `declaration_on` extracts an unknown category string faithfully — no unit
test exercises the actual gating line, `!DECLARED_CATEGORIES.contains(&cat)`, at any level before
this strike. Noted, not escalated — it is still a real (if partial) component-level proof, and
`DECLARED_CATEGORIES.contains` is a one-line stdlib call with essentially no room for a distinct
"wiring" bug independent of the value `declaration_on` hands it.

## ⛔ A correction to DESIGN itself, found while re-deriving it

**DESIGN's own state table is short one state.** It lists 6 rows (`absent`, `bad-category`,
`no-owner-field`, `owner-absent`, `owner-live`, `owner-ignored`) and never mentions the 7th at
all — not in the table, not in prose, not among "out of scope."

The 7th state is **`short-reason`** — the `reason.chars().count() < MIN_REASON_CHARS` branch at
gate line 268. It is named explicitly, as one of the 7 axis-2 values, in the underlying cast
report DESIGN is built on:

> `peragrare.md`: *"AXIS2 declaration state (7: absent | bad-category | **short-reason** |
> no-owner-field | owner-absent | owner-live | owner-ignored)"*
>
> *"`(clean, short-reason)` — unvisited (0 of 268 members) — hides: a regression in the
> `MIN_REASON_CHARS` enforcement (line 268, off-by-one or comparison flipped) is never caught,
> because all 3 real reasons are far longer than the 24-char floor and none sits near the
> boundary. **Unchecked**."*

Checked against the ten reader tests at the bottom of the gate file: **none of them exercises
`MIN_REASON_CHARS` at any boundary, or at all.** There is no unit test analogous to
`an_invented_category_is_read_not_skipped` for reason length. So `short-reason` is not merely
"proven per-component but not composed" like the other six — **it had ZERO test coverage of any
kind before this strike**, per-component or composed. This is the state most worth driving, and
DESIGN's silence on it means the strike task's own framing ("the gate's seven exemption states
each have a per-component proof") was true for six of seven, not all seven.

The new composed test (below) covers `short-reason` along with the other six, which both answers
DESIGN's stated goal and closes this gap DESIGN itself missed.

## The refactor

**Not larger than it looked — completed as pinned.** `check_shard`'s single 90-line body was split
into three functions in `tests/lint/every_wat_bad_fixture_actually_fails.rs`:

- `verdict_for(rel, started_clean, content, sources) -> Option<String>` — the per-fixture
  branching, PURE (no filesystem, no `startup_from_file`), byte-identical failure messages to the
  original inline code.
- `collect_failures(fixtures: &[Fixture], sources: &[String]) -> Vec<String>` — the loop-and-push
  wiring itself (the `continue`-equivalent short-circuit, and whether a verdict reaches the
  output). Also pure.
- `check_shard(shard: usize) -> Vec<String>` — now a thin disk-walking shell: builds the real
  corpus's `Vec<Fixture>` (path, `startup_from_file(rel).is_ok()`, source text) and hands it to
  `collect_failures`. **No `assert!` inside it anymore for the fixture verdicts** — floor/vacuity
  checks (corpus-floor, non-empty shard, sources-not-empty) stayed as `assert!`s since they guard
  the WALK's own sanity, not a fixture's verdict.

`Fixture = (String, bool, String)` is the seam type: `check_shard` builds one list of these from
disk; the new composed test (below) builds a different list of these by hand, in memory, calling
the identical `collect_failures`. **No `.wat.bad` was ever written to any tree.**

The 16 `#[test] fn $name()` shard wrappers now do `let failures = check_shard($idx); assert!(failures.is_empty(), …)` — the big instructional panic message moved from inside `check_shard` to
the macro, verbatim except dropping the "N of the offenders declared something" aside (it read
`declared`, which the new split no longer threads through — purely informational, not load-bearing
on any assertion).

No `.config/nextest.toml` override references this test's internal function names (only
`test(every_wat_bad_diagnostic_is_byte_stable)` and `binary_id(wat::lint)`, both untouched — no
test function was renamed, and the new composed test runs in 0.01s, well inside the generic
`wat::lint` budget). Checked `grep -rn check_shard` across the repo: the only other hit is
`tests/lint/diagnostic_output_is_deterministic.rs`'s own unrelated, differently-shaped
`check_shard` in a different file — not touched.

## The new composed test

`tests/lint/every_wat_bad_fixture_actually_fails.rs`, `mod composed`,
`check_shard_composition_drives_every_exemption_state`. Drives `collect_failures` — the exact
function the real gate calls — over an in-memory `Vec<Fixture>` of 8 synthetic entries: one dirty
startup (the short-circuit) plus one fixture per exemption state (`absent`, `bad-category`,
`short-reason`, `no-owner-field`, `owner-absent`, `owner-live`, `owner-ignored`), with a synthetic
`sources` haystack shaped like a real `tests/*.rs` file (one still-`#[ignore]`d owner, one live
owner, separated by the same blank-line boundary `an_ignore_on_a_different_test_does_not_vouch_
for_this_one` proves at the component level). Asserts the exact SET of paths that came back as
failures — not just a count — so a state routed to the wrong branch is caught, not merely a wrong
tally.

### A real, non-flake RED found and fixed mid-strike

First floor run after the refactor went RED — **not a flake, a genuine hit from my own new code**,
captured before any re-run per doctrine:

```
Summary [ 454.467s] 5490 tests run: 5489 passed (2 slow), 1 failed, 19 skipped
    FAIL [   0.052s] ( 120/5490) wat::lint no_inlined_edn::tests_carry_no_inlined_edn
```

```
thread 'no_inlined_edn::tests_carry_no_inlined_edn' (2944704) panicked at /home/john/work/holon/wat-rs/tests/lint/no_inlined_edn.rs:803:5:

🔥🔥🔥 INLINED-EDN — 1 site(s) carry a string literal whose content opens with
`#`/`{`/`[`/`(` (EDN-esque, trimmed of leading whitespace).
...
Offenders:

tests/lint/every_wat_bad_fixture_actually_fails.rs:504
```

Cause: `synthetic_sources()`'s string literal began `"#[test]\n#[ignore = …"` — trimmed, it opens
with `#`, tripping the EDN-esque detector. The file's own existing synthetic sources
(`an_ignored_test_reads_as_banked`, `a_running_test_reads_as_live`,
`an_ignore_on_a_different_test_does_not_vouch_for_this_one`) all avoid this by prefixing a `//`
comment line. Fixed by following that same established convention:
`"// two synthetic gates\n#[test]\n#[ignore = …"`. Re-ran only the two affected test groups
(`no_inlined_edn` + `every_wat_bad_fixture_actually_fails` + `composed` + `reader`) to confirm the
fix — 50/50 green — before re-running the FULL floor from scratch (not a partial re-run standing
in for it).

### Mutation proof (verbatim)

Broke the exact routing FINDINGS names as sharpest — the gate's own advertised "self-clearing"
mechanism — by rerouting `Owner::Live` from a failure to `None` in `verdict_for`:

```rust
// before (correct):
Owner::Live => Some(format!(
    "  {rel}\n      rune:lint({cat}) names `{owner}`, which is NO LONGER #[ignore]d — \
     the gap this fixture banked has CLOSED. The exemption is stale: either the file \
     now fails (drop the rune) or it does not and the name is wrong (rename it .wat)"
)),

// mutated (broken):
Owner::Live => None,
```

Ran only the new composed test:

```
$ cargo nextest run --release --test lint -E 'test(every_wat_bad_fixture_actually_fails::composed)'
    FAIL [   0.010s] (1/1) wat::lint every_wat_bad_fixture_actually_fails::composed::check_shard_composition_drives_every_exemption_state
  stderr ───
    thread '...check_shard_composition_drives_every_exemption_state' (2896426) panicked at
    /home/john/work/holon/wat-rs/tests/lint/every_wat_bad_fixture_actually_fails.rs:578:9:
    assertion `left == right` failed: composed wiring routed the wrong fixtures to failure — full failures: [
        "  absent.wat.bad\n      starts up CLEAN (startup_from_file returned Ok) but is named `.wat.bad`, and declares nothing",
        "  bad-category.wat.bad\n      rune:lint(nonsense) is not one of [\"bad-is-banked\"] — a second category needs its own discriminating question against that one, added to this gate deliberately",
        "  short-reason.wat.bad\n      rune:lint(bad-is-banked) carries no reason (9 chars). It must say what the substrate SHOULD do with this file instead of accepting it",
        "  no-owner-field.wat.bad\n      rune:lint(bad-is-banked) names no owner. Append `banked-by: <test fn name>` — the ignored test that banks this gap is what makes the exemption checkable and self-clearing",
        "  owner-absent.wat.bad\n      rune:lint(bad-is-banked) names `no_such_fn_anywhere` as its owner, but no `fn no_such_fn_anywhere` exists under tests/. The exemption points at nothing",
    ]
      left: ["absent.wat.bad", "bad-category.wat.bad", "short-reason.wat.bad", "no-owner-field.wat.bad", "owner-absent.wat.bad"]
     right: ["absent.wat.bad", "bad-category.wat.bad", "short-reason.wat.bad", "no-owner-field.wat.bad", "owner-absent.wat.bad", "owner-live.wat.bad"]
```

`owner-live.wat.bad` is missing from `left` (actual) — the mutated wiring silently absorbed the
stale exemption instead of flagging it, exactly the mechanism FINDINGS' `4P1` row named as the
highest-value gap. **RED confirmed, exact arm named** (the `Owner::Live` match arm in
`verdict_for`). Restored the arm verbatim; re-ran the same scoped selection
(`every_wat_bad_fixture_actually_fails` + `composed` + `reader`, 32/32) — green. `cargo clippy
--release --test lint` — clean, no new warnings.

## Floor

Ran `./scripts/floor.sh` in the foreground (no backgrounding), twice: once that surfaced the real
`no_inlined_edn` red above (captured and fixed, not re-run blind), and once clean after the fix.
Final `Summary` line, read from `.floor/latest/clean.log`:

```
Summary [ 454.461s] 5490 tests run: 5490 passed (2 slow), 19 skipped
```

5490, not the pinned 5489 — accounted for exactly by the one new test
(`check_shard_composition_drives_every_exemption_state`); no other count moved.

## What I did NOT do

- Did **not** add fixtures for any of the six previously-unvisited states to the real corpus —
  DESIGN rules this out explicitly (would red the gate).
- Did **not** re-litigate `peragrare`'s grid or corpus measurement, which DESIGN's refutation of
  `4P1` leaves standing and I re-confirmed by reading `peragrare.md` directly.
- Did **not** touch `3P1`'s axis-pair coverage (DESIGN: "same class, different instrument, its own
  row").
- Did **not** write a temporary on-disk `.wat.bad` corpus at any point — the PIN held throughout;
  the refactor made it unnecessary rather than merely avoided.
- Did **not** add a second mutation-proof beyond the `owner-live` one above. One clean, verbatim,
  restored mutation proof was asked for and delivered; I judged a second (e.g. breaking the
  dirty-startup short-circuit, or the `failures.push` in `collect_failures`) as available but not
  required — the composed test's dirty-startup case (`verdict_for(rel, false, garbage, sources)`
  must yield `None`) is already present in the test body and would catch a swallowed-continue
  regression, just not separately mutation-proven in this pass.
- Did **not** change `DECLARED_CATEGORIES`, `MIN_REASON_CHARS`, `OWNER_FIELD`, or any of the
  10 pre-existing `reader` unit tests — all run unmodified and green (`22/22` reader+composed+
  shard-adjacent selection above, `10/10` reader tests specifically).
