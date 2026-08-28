# STONE 296-L — a bare `is_err()` asserts nothing

DRAWN 2026-08-26 against `f3fcbfafe`.
**PRIOR ART — read these two before anything else.** `tests/lint/no_loose_string_assert.rs` (the
lint shape, the `rune:lint` exemption form, the drive-to-zero campaign — now GREEN with 158 audited
exemptions) and `docs/arc/2026/06/278-rules-engine/BRIEF-check-error-membership-assert.md` (which
minted `assert_check_error_present!`, the macro this stone extends rather than reinvents).

## Why this is arc 296's, and not a new arc

296 minted the loose-assert doctrine and drove its lint to zero. **Measured: `BRIEF-296-loose-assert-purge.md`
contains ZERO occurrences of `is_err`.** The campaign attacked assertions that check a value
*loosely* and never scoped the sibling class — an assertion that checks **nothing at all about which
error**. Same disease, one level deeper, same arc. 296 has no `INSCRIPTION.md`; it is open.

## The defect, in its own words

```rust
assert!(
    result.is_err(),
    ":guard false on the only clause should raise NoMatchingClause; got Ok",
);
```

**The message names the expected error. The assertion checks none of it.** The prose knows
`NoMatchingClause`; the code accepts *any* `Err` — a retirement, a typo, a parse failure, a renamed
fixture, a missing file. This is not hypothetical: Stone C (arc 255) retired a name and **silently
disarmed eleven tests of exactly this shape with the floor green throughout**, and Stone F had one
more at risk that only survived because a human pasted a diagnostic and read it.

That hand-check is the tell. "Paste the diagnostic and confirm it names the right thing" has now
been written into two consecutive briefs — the **convention** rung, executed by someone remembering.
It held twice. It fails the first time nobody does.

## ⛔ THE ONE CONTRACT DECISION — the discriminant is the INNER error, NEVER the outer variant

`startup_from_file` returns `Result<FrozenWorld, StartupError>` (`src/freeze.rs:958`), and
`StartupError` has 11 variants. **Matching the outer variant is VACUOUS for exactly the failure this
stone exists to prevent**, because a retirement error and the defect it masked are *both*
`StartupError::Check`:

```
the DEFECT      #wat.check/CheckErrors [ #wat.check/EnsureFnInvalid  {…} ]
the RETIREMENT  #wat.check/CheckErrors [ #wat.check/MalformedForm    {"…is retired; use…"} ]
```

Both verified this session by running `--check` on the real fixtures. `assert!(matches!(e,
StartupError::Check(_)))` would have passed on both — a green test proving nothing, which is the
defect wearing a fix's clothes. **A migrated site must name the inner `CheckErrorKind`** (or, for
non-`Check` variants, the specific variant that carries the meaning). **STOP-2.**

## The population — 150, and the instrument is committed beside this brief

```
tests/types      43      tests/services    7      tests/kernel      4
tests/function   42      tests/rete        6      tests/collection  2
tests/wat_lang   12      tests/comms       4      tests/reflection  1
tests/resolve     8      tests/diagnostics 4      tests/process     1
tests/macros      8      tests/value       8
                                    150 sites across 84 files, 14 directories
```

⚠ **A first line-based grep said 70.** An `assert!` spans lines, so a line-based instrument cannot
see the statement. `PROBE-296-L-bare-is-err-census.py` (committed) is statement-scoped and
paren-balanced, and was **validated against five ground-truth controls before its number was
quoted** — single-line bare → BARE, wrapped multi-line bare → BARE, `matches!`-guarded → KINDED,
`is_ok()` → neither, `if x.is_err()` control-flow → neither. All five behaved as specified.
Re-run it; do not trust this table. `[[feedback_validate_a_search_pattern_before_trusting_its_count]]`

## ⛔ PHASE ORDER — the lint lands LAST, and that DEPARTS from 296's own precedent

`no_loose_string_assert` landed as a deliberate **expected-red** progress meter. **We cannot repeat
that**: main only takes a green tree. So:

```
PHASE 1   the macro.  `assert_startup_error!` beside its three siblings in src/lib.rs.
PHASE 2   migrate 150 -> 0.  Fanned by DIRECTORY; the census is the worklist.
PHASE 3   the wall.  tests/lint/no_bare_is_err.rs lands on an ALREADY-ZERO tree.
```

The lint therefore never shows a red on main; it lands as the ratchet that holds zero. Phase 3 on a
non-zero tree is **STOP-4**.

## The rooms — verified against `f3fcbfafe`

```
src/lib.rs:353                        assert_check_error_present!  — the sibling to COPY (22 live callers)
src/lib.rs:241, :305                  assert_edn_eq! / assert_edn_matches_file! — the neighbours
src/freeze.rs:958                     startup_from_file -> Result<FrozenWorld, StartupError>
src/freeze.rs  `pub enum StartupError` 11 variants; Check(CheckErrors) is the one that matters
src/check/error.rs:87                 `pub enum CheckErrorKind` — the INNER discriminant
tests/lint/no_loose_string_assert.rs  the lint to copy: scan, fail-listing every file:line, rune:lint exemption
tests/function/probe_arc237_stone3_guard_ensure.rs:154   8 sites — densest file, and Stone F's near-miss
```

**What the migration reads from:** each site's own assertion message already names the error it
expects. That message is the transcription source — **and the finding source.** Where the message
names an error the code does not actually produce, that is the prize of the whole stone.

## Phase 2 is a FAN-OUT, and the riders do TEXT ONLY

Directories are disjoint, so one rider per directory (`tests/types` and `tests/function` each merit
their own; the remaining twelve fit one tail rider). **No rider runs the floor or clippy** — one
`target/` lock, N builds, and a per-rider gate over a workspace it does not control is a gate that
lies. The orchestrator weighs centrally once the tree is quiescent.

## Your role

cwd `/home/john/work/holon/wat-rs`; `pwd` first. **Ending your turn ENDS you** — every command
FOREGROUND. **No sub-agents. No git worktrees.** Do not commit, push, revert, or `git checkout`.
⛔ **Do not run `git stash` in any form.** For before/after, copy to `/tmp` and diff there.

You may run `cargo build --release`, `./target/release/wat --check <f>`, targeted
`cargo nextest run --release -E '<filter>'`, and the census probe. Not the floor, not clippy.
⚠ `cargo test` is not a diagnostic here — `src/host/test_runner.rs:48-55` documents why.

## STOP triggers — each REJECTS. Ship nothing further on that thread; report the gap.

1. **STOP-1 — a site's message names an error the code does not produce.** That is a **FINDING**.
   Surface it. **Do NOT rewrite the message to match observed reality** — that converts a real
   discovery into a silent capitulation, and the message was the only record of the original intent.
2. **STOP-2 — you would assert only the outer `StartupError` variant.** Vacuous by construction.
3. **STOP-3 — a site whose error genuinely carries no stable discriminant.** Take the
   `// rune:lint(bare-is-err) — <reason>` exemption with a real reason. Never a fudge, never a
   `contains` (that trips the loose-assert lint, which is green and must stay green).
4. **STOP-4 — the lint would land on a non-zero tree.**
5. **STOP-5 — you would change `startup_from_file`'s signature.** 429 occurrences in `tests/**.rs`,
   of which the census finds 150 in a bare-`is_err` assertion — so the large majority are POSITIVE
   uses this stone must not disturb. Out of scope, affirmatively: the blast radius exceeds the
   defect by an order of magnitude, and the check rung is where this material runs out.

## Acceptance — every row measures a MECHANISM

```bash
# 1. the census reaches ZERO — same instrument that produced 150.
python3 docs/arc/2026/06/296-diagnostics-fully-edn/PROBE-296-L-bare-is-err-census.py    # BARE: 0

# 2. ★ NON-VACUITY OF THE MIGRATION — the point of the whole stone.
#    Pick THREE migrated sites. Perturb each fixture so it raises a DIFFERENT error
#    (e.g. rename a verb in the .wat.bad so it raises a resolve error instead of the declared one).
#    Each test must now FAIL. Restore; each must pass. Report all six outcomes.
#    Before this stone every one of those perturbations would still have passed.

# 3. ★ the lint can fail — break its door.
#    Add one bare `assert!(x.is_err())` to any test; the lint goes RED naming that file:line.
#    Remove it; GREEN. Report both.
cargo nextest run --release -E 'test(no_bare_is_err)'

# 4. the loose-assert lint stays green — no migration reached for `contains` as an escape.
cargo nextest run --release -E 'test(tests_carry_no_loose_string_assert)'

# 5. every exemption carries a reason; list them for the orchestrator to audit (excusare).
grep -rn 'rune:lint(bare-is-err)' tests/ --include=*.rs

# 6. build clean.
cargo build --release && cargo build --release --all-targets
```

## Report back with

- Each row's actual output, naming the command that produced each number.
- **Row 2's six outcomes in full** — three perturbations, three restorations.
- **Row 3's both outcomes** — the lint RED with a bare assert present, GREEN without.
- **Every STOP-1 finding**: the site, the message's claim, and the error actually produced.
- Every exemption taken, with its reason, so it can be audited rather than trusted.
- Anything this brief got wrong. What you did NOT do, and why.
