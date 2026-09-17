# SCORE — `.wat.bad` is enforced now, and my own contract decision was the thing that was wrong

> **Written after the orchestrator's own re-run.** Rows marked *re-driven* were run on this machine
> at HEAD `29b207e6e` + the strike. The rider's figures are noted where they differ; none was taken
> on trust.

## The scorecard, graded

| # | required | result |
|---|---|---|
| 1 | ★ every `.wat.bad` fails at startup | ✅ `every_wat_bad_fixture_actually_fails`, **sharded ×16, 16.1 s**, population **281** |
| 2 | ★ the 16 resolved | ✅ **13 renamed**, **3 runed** — see the correction below. `.wat.bad` 281 → **268** |
| 3 | ★ discovered, not listed | ✅ walks `tests/`+`wat-scripts/`+`docs/`; `CORPUS_FLOOR = 200` |
| 4 | REDs on a regression | ✅ rename a passing fixture back → RED naming it, 25/26 |
| 5 | a renamed fixture stays load-bearing | ✅ break it → its own test REDs, span naming the **new** `.wat` path |
| 6 | ★ cannot pass vacuously | ✅ **re-driven by the orchestrator** — see below |
| 7 | the rowed mechanism honestly scoped | ✅ the 2 `arc170_slice_1e_*` left alone, and said so |
| 8 | zero mains added | ✅ `git diff HEAD -- '*.wat*' \| grep '^+.*:user::main'` → **empty** |
| 9 | floor / lints / clippy | ✅ **`5402 tests run: 5402 passed (2 slow), 21 skipped`** (439.5 s), **0 FAIL rows**, lints **254** (+26 = the shards), clippy rc=0. **Skipped unchanged at 21** — nothing was left un-ignored |
| 10 | `src/` | ✅ zero diff, index AND worktree |

## ⛔⛔ THE FINDING IS MY OWN CONTRACT DECISION, AND IT WOULD HAVE DESTROYED EVIDENCE

The DESIGN pinned: *"a fixture that legitimately starts up clean does not get a rune — it gets
renamed; the extension is the claim,"* on the stated premise that *"the 16 divide into two honest
kinds, and neither is a bug in the test."*

**Three of the 16 are a third kind, and they are exactly the STOP-1 the brief told the rider to stop
on — while the brief listed all three by name in its own 16 and asserted no STOP applied.**

| file | its test asserts |
|---|---|
| `probe_diag_typealias_leniency_check` | `Ok(_) => panic!("LENIENT: …")` |
| `probe_undefined_builtin_resolves_wrong_leaf` | `assert!(result.is_err(), …)` |
| `probe_undefined_builtin_resolves_bogus` | `assert!(result.is_err(), …)` |

All three are `#[ignore]`d — *"RED-at-HEAD: checker rejection of undefined builtins (arc-255
builtin-registry) not yet built; unlock when we circle back to arc 255"*. **`.wat.bad` here is an
ASPIRATION, not a lie**: the file *should* be rejected, the substrate is lenient today, and a banked
ignored test records the gap. **Renaming them would have erased three tracked known-gap markers** —
and my prescription had no way to express that, because I wrote it from a taxonomy I had measured but
not finished reading.

### The rider's counter-proposal is better than what I pinned, and it said so rather than complying

`;; rune:lint(bad-is-banked) — … banked-by: <test fn>`, where **the gate verifies the named test
exists under `tests/` and is still `#[ignore]`d**. Two properties my design could not have had:

- the exemption is **checked, not declared** — a rune naming a test that does not exist REDs;
- it is **self-clearing** — when arc 255 lands and the test is un-ignored, the gate REDs and forces
  the file to be dealt with. It cannot rot into a permanent excuse.

The rider invented that arm, so it mutation-proved it: un-ignoring a banked owner REDs with *"names
'wrong_operator_leaf_is_a_check_error', which is NO LONGER #[ignore]d — the gap this fixture banked
has CLOSED."* **Accepted over the DESIGN.**

## Row 6, re-driven by the orchestrator

Roots repointed at a directory that does not exist:

```
FAIL ( 1/26) every_wat_bad_fixture_actually_fails_shard_07
  the .wat.bad walk found only 0 file(s) under ["nonexistent-dir-for-mutation-proof"] — under the
  floor of 200, so it is not reaching the corpus it claims to guard and a green verdict below would
  mean nothing
```

All 16 shards RED in 0.039 s. Gate restored, **md5-verified against HEAD**. This is the arm that
matters most: a discovered gate that finds nothing and reports success is the defect this arc has now
found in three separate places.

## The 13 renamed, by kind

- **Kind 1 — premise retired, test correctly asserts `is_ok()` (7):** the `arc237_8a/8b/8c/8d`
  family, all *"arc 300 C4/C5 retired 237.8a's reject"*.
- **Kind 2 — starts up, then INVOKES; error at eval (4):** `probe_arc241_stone5_c05`,
  `probe_diagnostic_non_keyword`, `probe_diagnostic_non_vector`,
  `probe_arc234_stone4_hash_destructure`. **Exactly the builder's pattern.**
- **Kind 3, which the brief does not name (2):** `stone10_remedy_c04`/`_c08` assert
  `assert_eq!(msg, "<startup succeeded — no error to display>")` through a `display_err` sentinel.
  Their own header calls C04 *"passes trivially at HEAD"* — the "did you mean" path they claim to
  probe is never reached. Renaming is right; **the triviality is pre-existing and affirmatively left
  out of scope**, not fixed silently.

## Honest deltas — four more corrections to my artifacts

- **A rename collision the brief did not anticipate.** `tests/wat_lang/probe_arc234_stone4_hash_destructure.wat`
  **already existed** as the `startup_beside(file!())` fixture for probes 3/4/6. A blind `git mv` to
  `.wat` would have overwritten it and destroyed three passing tests. Renamed to
  `…_unknown_field.wat` instead. **"Rename the 16 to `.wat`" is not always available.**
- **Blast radius understated.** Beyond the fixtures, their `.rs` referrers and the gate, the rename
  also touched an **`.edn` golden carrying the fixture path AND its line numbers** (load-bearing — it
  went red on a one-line header shift), a sibling `.wat`'s comments, and a docs `.md` gated by
  `no_stale_path_in_doc`.
- **I pointed at the wrong precedent.** The BRIEF sends the rider to
  `every_walking_gate_declares_non_vacuity.rs` for house style. The near-exact sibling is
  **`tests/lint/docs_wat_loads_or_declares_why_not.rs`** — the same walk-and-startup question in the
  opposite direction, with the rune-or-load contract already built and `rune:lint(red-by-design)`
  already minted. Never mentioned.
- **`diagnostic_output_is_deterministic.rs` should NOT share the discovery**, and the reason is this
  strike's own history: it drives the **binary**, which is the instrument that got draft 1 withdrawn.
  Coupling them would put the wrong driver back inside this gate. Its corpus went 278 → 265, above
  its own `> 200` floor; verified green.

## What the rider did that is worth copying

- **It hit nextest's 30 s kill** with one 281-startup test and **split instead of overriding**,
  because `.config/nextest.toml` answers that case by name: *"⛔ IF THIS EVER NEEDS RAISING, SPLIT
  INSTEAD."* No config change.
- **Its mutation 2 did not mutate on the first attempt** — a return-type change that `apply` leaves
  unconstrained, so the file still type-checked and the test still passed. **It discarded that
  reading instead of banking it**, which is the exact false negative that invalidated a C16 proof
  this week.
- **Its own md5 check caught it stripping one byte too many** on a restore (a trailing newline),
  diagnosed with `xxd`.

## Pre-existing false prose the rename exposed

Three fixture headers said *"startup MUST fail"* while their own tests asserted startup succeeds —
`probe_arc241_stone5_c05`, `probe_diagnostic_non_vector`, `probe_arc234_stone4_hash_destructure`.
**The filename lie was mirrored in the comments.** Two `8c`/`8d` headers also said *"Name/path kept
unchanged"*, now false. All corrected.

## What this strike does NOT close

C18's rowed mechanism (`assert!(!ok)` unfalsifiable via a nil main) is **2 files, both legitimate**,
and is left alone. The row's alarm was right; its mechanism was not, and the defect underneath it was
larger. **C18 closes on the enforced extension, not on the idiom it named.**
