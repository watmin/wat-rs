# EXPECTATIONS — the type registry holds the BUILTIN types

Written BEFORE the strike, against `ff0ca0b2c`. Ruled E-by-C.

## The scorecard

| # | what | my command | expected |
|---|---|---|---|
| 1★ | the DOOR tells the truth | a test calling `registrations(":wat::core::i64")` | contains `RegistryKind::Type` |
| 2 | container · opaque · rust-backed | same, three more names | contains `Type` |
| 3★★ | negative control | `registrations(":user::NoSuchType")` | **empty** |
| 4★ | membership without structure | `get(":wat::core::i64")` | **`None`** |
| 5 | the derived gate | the const-iterating test | every entry `contains`-true |
| 6 | `TypeDef`/`Nature` untouched | `git diff src/types.rs` | no new variant, no new `Nature` arm |
| 7 | THE DOOR untouched | `git diff src/value/symbol_table.rs` | **empty file diff** |
| 8 | the floor | `scripts/floor.sh` | **4859/4859**, 0 FAIL, 19 skipped — **unchanged** |
| 9 | clippy | `-D warnings` | 0 |

**Row 8's expectation is UNCHANGED, and that is the unusual part.** Every other stone this session
moved the count or the behaviour. This one adds membership answers that nothing reads yet, so a
green-and-identical floor is the correct outcome and any movement is a finding.

## The rows that can lie

**Rows 1, 2 and 5 are all positives, and a `contains` that returns `true` unconditionally passes
every one.** Row 3 is the only row that can distinguish a populated registry from a broken predicate.
If row 3 is not verbatim in the report, the other rows are unearned.

**Row 1 must go through `registrations`, not the new field.** A test that reads the store directly
proves the store works and says nothing about the door — and the door is the whole ruling. If the
rider tests `TypeEnv`'s new field instead, rows 1 and 2 are measuring the wrong subject.

**Row 4 is doing double duty.** It asserts the asymmetry (membership without structure) so it lives
in a test rather than a comment, and it is also the guard against someone later "fixing" `get` to
fabricate a `TypeDef`. A rider who finds row 4 uncomfortable and makes `get` return something is
building option A by accident.

**Row 7 is the narrow-waist check made mechanical.** The ruling is that nothing above `TypeEnv`
learns anything. If `symbol_table.rs` has a diff, the waist moved and the stone lost its own
argument.

## Independent prediction

**Runtime: 30-50 minutes.** The mechanism is small — one field, one `||`, one loop over two consts,
one explicit table. The cost is in verification, not construction: group 3 is ~24 names that each
need a corpus citation before registering, and the brief forbids transcribing them.

**Trap-doors named in advance:**
- **Transcribing the evidence list.** The likeliest failure and the one I set up by providing the
  list at all. It is measured evidence from a rider's convergence, not a census I ran — and this file
  has made my counts wrong four times. A report without per-name citations means row-by-row
  verification did not happen.
- **The colon convention.** `BARE_CONTAINER_HEADS` carries no leading colon while the registry is
  colon-prefixed. A rider that iterates it naively registers `wat::core::Vector` and row 2 fails on
  the container while passing on the primitive — a half-green that reads as a typo, not a defect.
- **Registering a name that is not a type.** `:wat::core::Value` and `:wat::core::Never` are
  escape-hatch sentinels from the check layer's own scheme registry. If they are not genuinely usable
  in a type position, registering them makes the future wall accept something it should reject.
  STOP-2 covers it; I expect this to be the one that fires, if any does.
- **The floor moving.** If STOP-1 fires, something depended on `contains` being false for a builtin
  name. That is a substrate finding worth more than this stone.

## Mode

- **Mode A** — all nine rows, row 3 verbatim, every group-3 name carrying a corpus citation, floor
  unchanged.
- **Mode B** — ships, but group-3 names transcribed without citation, or row 1 tested against the
  store rather than the door.
- **Mode C** — a STOP fires. Ship nothing; the report is the deliverable. STOP-1 firing is a GOOD
  outcome: it means a consumer was relying on the registry's silence.
