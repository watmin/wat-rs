# STONES 296-L / 296-M, PHASE 3 — the two walls, and the four probes that pass for the wrong reason

DRAWN 2026-08-27 against `de49c56b1`.
**PRIOR ART:** `tests/lint/no_loose_string_assert.rs` is the shape both lints copy — scan, fail
listing every `file:line`, `rune:lint` exemption, drive-to-zero. `git log -1 4b49f3c5c` and
`git log -1 de49c56b1` carry the two stones this closes.

Two independent pieces of work. **They may be struck by two riders in parallel — the files are
disjoint.** Piece B does not depend on piece A.

---

# PIECE A — the four probes that pass for the wrong reason

Stone L's whole thesis is that a bare `is_err()` hides a test passing for an unrelated reason. It
found six. **Two are genuinely blocked** on arc 255's unbuilt registry (`--check` exits 0 outright)
and are already `#[ignore]`d — leave them. **Four are fixable, and fixing them is this piece.**

For each: the fixture must be made to REACH the code the test claims to exercise, then the assertion
migrated to name that error via `assert_startup_error!` / a `matches!` on the real kind.

```
tests/types/probe_arc293_4d_fix_silent_member_drop.rs:16
    CLAIMS   "members written outside the [...] vector must be a hard error"
    ACTUALLY #wat.type/MalformedDecl "expected `:nature :<kw>` after the surface name"
    WHY      the probe landed 2026-06-28 (35ba08636); arc 278 S4c made `:nature` MANDATORY on
             2026-07-07 (38f31069f). The arity gate now fires on arg[1] before the parser ever
             reaches the member-vector logic. Give the fixture a `:nature` clause so the
             member-vector case is reachable again.
    ⚠ IF THE DEFECT IS BACK: with `:nature` present, the members may once more be SILENTLY
      DROPPED. That is a REAL SUBSTRATE BUG resurfacing, not a test problem. STOP and report it —
      do not fix the substrate in this stone.

tests/macros/probe_arc249_macro_engine.rs:55   and   :94
    CLAIM    an F5 purity-gate rejection (:55) and a hygiene-bound refusal (:94)
    ACTUALLY #wat.macro/MalformedDefmacro "expected return-type keyword after `->`"
    WHY      a stale return-type spelling — `(:AST :- [:wat::holon::HolonAST])` — fails SIGNATURE
             PARSING before either gate runs. Working fixtures in the same directory spell it as a
             bare keyword (e.g. `:wat::WatAST`). Fix the spelling in each fixture so the form
             parses and the gate under test actually fires.

tests/rete/probe_arc278_seq1b_list_hofs.rs:196
    CLAIMS   a List/Vector preservation TypeMismatch
    ACTUALLY StartupError::Parse(Lex(AngleTypeHeadInName))
    WHY      the fixture uses the RETIRED angle-bracket spelling
             `:wat::core::Vector<wat::core::i64>` (arc 109 annihilated it), so it dies at LEX time.
             Respell to the surviving `(:wat::core::Vector :- [:wat::core::i64])` form.
```

**Method:** ground every claim by running the fixture (`./target/release/wat --check <f>`) BEFORE
and AFTER, and paste both. The point is not that the test goes green — it was already green. The
point is that it now fails for its own reason. **Prove it: perturb the repaired fixture so the
defect under test is absent, and show the test FAIL.**

**STOP-A1 — a repaired fixture reveals a live substrate defect.** Report it in full; do not fix it.
**STOP-A2 — a fixture cannot be made to reach its target without changing what the test means.**
Report it; the probe may need retiring instead, which is the builder's call.

---

# PIECE B — the two walls

Both land on an ALREADY-ZERO population (L's is zero once Piece A lands; M's is zero now), so
neither ever shows a red on main. Copy `tests/lint/no_loose_string_assert.rs` for structure: scan
`tests/`, collect offenders, fail with every `file:line` and a rubric naming the fix.

## B1 — `tests/lint/no_bare_is_err.rs`

Bans an `assert!` statement whose body mentions `.is_err()` and checks nothing about WHICH error.
The detector is already written and validated — port
`docs/arc/2026/06/296-diagnostics-fully-edn/PROBE-296-L-bare-is-err-census.py` to Rust: find each
paren-balanced `assert!( … )`, skip char and string literals while balancing (a `'('` desyncs a
naive counter — that defect was found IN that instrument), and flag it if the body contains
`.is_err()` and none of `matches!`, `assert_check_error_present`, `assert_startup_error`, `.kind`,
`unwrap_err`, `err_kind`, `StartupError::`.

⛔ **THE LINT FREEZES NAMES, NEVER A COUNT.** Two sites remain and cannot be fixed — both in
`tests/wat_lang/probe_undefined_builtin_resolves.rs`, both `#[ignore]`d, both waiting on arc 255's
registry (`--check` exits 0, so there is no error to name). Put those two in an explicit `const`
allowlist keyed by `file:line`-independent identity (file + test fn name), each with its blocker
written beside it. A count cannot tell "+1 new, −1 fixed" from "nothing happened", and its error
text cannot name the offender. `[[feedback_a_gate_freezes_names_never_a_count]]`

Exemption form: `// rune:lint(bare-is-err) — <reason>`, per-offense.

## B2 — `tests/lint/no_error_flattening_helper.rs`

Bans `fn … -> Result<_, String>` in `tests/` whose body `map_err`s a typed error into a `format!`
or `to_string`. Port `PROBE-296-M-flattening-helper-census.py`. Population is currently **0**, so
no allowlist is needed — and if you find yourself wanting one, that is the signal something is
wrong, not that the list should exist.

The rubric its failure prints must say what Stone M learned, because the obvious fix is wrong half
the time: **return the TRUE error type.** `StartupError` is the union only when a helper genuinely
chains several (`Parse`/`Macro`/`Type`/`Resolve`/`Check`/`Runtime`); a helper that chains only
`call_beside_value` should return `RuntimeError`, and wrapping it is gratuitous envelope. And when
the `Err` is never inspected by any caller, the honest shape is **no `Result` at all** — panic on
the broken precondition (see `tests/program/probe_arc170_edn_bridge_unspellable.rs`'s `crosses`).

## ⛔ Both walls must be PROVEN BY BREAKING THEIR DOOR

A gate that cannot fail is not a gate. For EACH lint: introduce one violation, show it RED naming
that exact `file:line`, remove it, show GREEN. Report all four outcomes verbatim. A lint that has
only ever been observed green has not been observed at all. `NISI FRANGAS, NIHIL PROBAS.`

---

## Your role (both pieces)

cwd `/home/john/work/holon/wat-rs`; `pwd` first. **Ending your turn ENDS you** — every command
FOREGROUND. **No sub-agents. No worktrees.** Do not commit, push, revert, `git checkout`, or run
`git stash` in any form.

**Do not run cargo** while a sibling rider is live — ground with `./target/release/wat --check`
(no build lock) and by reading source. The orchestrator compiles and weighs centrally.
**EXCEPTION for Piece B only:** proving a lint's door requires running it. Use exactly
`cargo nextest run --release -E 'test(<your lint fn>)'` — one scoped filter, nothing wider, and
never the floor.

## Report back with

Piece A: each fixture's `--check` output BEFORE and AFTER, the discriminant now asserted, and the
perturbation proof (fail + restore) for each. Every STOP-A finding in full.
Piece B: each lint's source, its rubric text, and the four broken-door outcomes verbatim. The exact
allowlist you froze and each entry's blocker.
Both: anything this brief got wrong. What you did NOT do, and why.
