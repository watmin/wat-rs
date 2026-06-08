# NOTE — arc 214 wrap-up: the ignore drawdown ledger

**Filed 2026-06-07 (builder: "remove all the ignores we can as we come to an
end").** With the v5 deadlock DEAD (Stone 6.4), the arc-170 ignore backlog —
much of it scar tissue around the deadlock + the arc-242 nil-syntax migration
— can finally draw down. This ledger classifies every LIVE `#[ignore]`
attribute (statement-position; doc-comment mentions excluded) so the closeout
knows what dies now, what's legitimately banked, and what dies with the
envelope retirement.

## Class A — nil-fixture (KILLABLE NOW; the 6.5 sweep)

11 process/subprocess tests `#[ignore]`'d solely on the arc-242
value-position-nil syntax ("...this still fails: ...':wat::core::nil' in value
position..."). The fd-leak in the reason is long dead; the nil syntax is the
only blocker — the identical fix that freed Stones 8.2/8.3 (value-position
`:wat::core::nil` → bare `nil`). Sites:
- tests/wat_run_sandboxed.rs ×3
- tests/comms/lifeline_orphan_clean_via_fork_program.rs ×1
- tests/probe_declaration_form_lift.rs ×3
- tests/wat_arc170_program_contracts.rs ×4 (the nil-class ones only; NOT the
  two walker-WIP ignores in the same file)
**Disposition: swept this wrap (the nil migration + un-ignore, each verified
enveloped).**

## Class B — process-tier envelope-routing (NOT failures; dies with the soak)

5 tests `#[ignore]`'d "process-tier probe: run via integration-run.sh /
setsid timeout" — these PASS; the ignore keeps them out of the default
`cargo test` run-tier (which deadlocked on the old stack). Sites:
peer_process_round_trip, peer_verb_round_trip_process,
peer_select_prime_process, spawn_program_prime_process ×2.
**Disposition: HELD for the envelope-retirement, which the stability-100 soak
(#207) gates.** 100/100 clean raw-workspace rounds PROVES the default tier is
safe → these ignores come out as part of that stone (the same stone that
un-bans the raw tier in the discipline docs). Removing them before the soak
would put process tests in the unproven default tier — premature.

## Class C — banked future-arc disconfirming gates (RED-by-design; STAY)

- `probe_arc251_stone0_symbol_head` — RED until arc 251.1 (open).
- `probe_diag_typealias_leniency` — arc 255 banked gate (the type-keyword
  leniency the 8.2 dark-class surfacing found; open).
**Disposition: STAY ignored until their arc lands. Legitimate.**

## Class D — 249-era diagnostics (AUDIT, not blind-remove)

3 `#[ignore]`'d "249.x diagnostic — run with --ignored to read the gap"
(probe_arc249_threading_in_wat, probe_arc249_4_rehome_in_wat ×2). Arc 249 is
CLOSED. These are read-the-gap diagnostics, not gates — they may now pass
(un-ignore), be stale (delete), or document a permanent gap (keep + reword).
**Disposition: the parked "stale 249-era diagnostic probes audit" — its own
small judge-stone, NOT this wrap's blind sweep.**

## Class E — walker WIP (AUDIT)

2 in wat_arc170_program_contracts.rs: "ARC-170 WIP: BareLegacyMainSignature
walker no longer fires for a non-...". The walker-2 parked class — a behavior
question (should the walker fire?), not a syntax fix.
**Disposition: the parked walker-disconnect audit; own stone.**

## Closeout accounting (for the Slice-9 INSCRIPTION, FM-11)

The INSCRIPTION states the final ignore census affirmatively: Class A killed;
Class B retired-with-the-soak (named); Classes C/D/E affirmatively scoped to
their named arcs/audits. No ignore left unaccounted.
