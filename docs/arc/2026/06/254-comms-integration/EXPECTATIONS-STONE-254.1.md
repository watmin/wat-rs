# EXPECTATIONS — Stone 254.1 (independent scorecard, written pre-strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | **load-bearing**: struct-with-`Sender`-field channel payload rejected | un-ignore the probe + `cargo test --release --test nursery channel_of_struct_with_opaque_field` | PASS (the gate now produces a portability rejection) |
| 2 | portable payload still accepted | `cargo test --release --test nursery portable_channel_payload_still_accepted` | green |
| 3 | parse-gate finding unaffected | `cargo test --release --test nursery bare_sender_payload_rejected` | green |
| 4 | lib baseline preserved | `cargo test --release --lib -p wat` | ~940/0/x (no NEW failures from the gate) |
| 5 | no over-rejection | `cargo test --release --test comms` + `--test nursery` | green; ANY red = a legitimately-portable struct/record payload wrongly rejected → fix `is_portable_type`, not the gate (STOP-2) |
| 6 | clippy clean on touched file | `cargo clippy --release` (check.rs) | no new warnings |

## Independent prediction

- **Runtime band:** 15–25 min (recursive `TypeExpr` classifier + `TypeEnv` Record/Struct/Enum lookups + the one gate site + cascade triage on existing channel tests).
- **Mode:** A (clean) likely; the classification set is fully enumerated in the BRIEF from the value-level encoder.

## Trap-door risks

- The `TypeDef` shape for reading struct field types / distinguishing Record/Struct/Enum may not be cleanly accessible → STOP-1 (report the shape).
- Existing tests may carry a struct/record channel payload that IS all-portable; if the predicate is too strict it reddens them → that's the predicate teaching its boundary; fix the predicate to accept all-portable composites (STOP-2), never loosen the gate.
- `reduce`/alias canonicalization must run before the match, or a typealias to a non-portable type slips through.

## Scoring note

Score against an INDEPENDENT re-run (orchestrator), not sonnet's say-so. Row 1 is
the un-ignored probe — verify it actually rejects (read the diagnostic), and that
rows 2/3 (portable still accepted) did not regress.
