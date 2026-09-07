# EXPECTATIONS — STONE 251.9: the catch-all that ate the declaration

Written BEFORE the strike. Every row's bar is derived from the KEYWORD-HEAD CONTROL that runs beside
it, never from what I expect the output to be.

⚠ **THE DEFECT'S SIGNATURE IS A PASS.** A symbol-headed declaration today exits 0 and declares
nothing. So no row may assert "the symbol form succeeds" — rows 1–3 assert the two spellings AGREE,
whatever the keyword spelling answers. That is why the committed probe runs both fixtures of a pair
and asserts them byte-identical below line 1 before comparing exit codes.

| # | what | command | expected |
|---|---|---|---|
| 1 | a malformed variant payload is refused under BOTH head spellings | `cargo nextest run --release -E 'test(probe_arc251_stone9)'` | `malformed_variant_is_refused_under_both_head_spellings` PASSES (RED at HEAD: kw=3, sym=0) |
| 2 | a missing MANDATORY purity marker is refused under both | same | `missing_mandatory_purity_marker_…` PASSES (RED at HEAD: kw=3, sym=0) |
| 3 | a symbol-headed declaration actually DECLARES | same | `a_symbol_headed_declaration_actually_declares` PASSES (RED at HEAD: the reference to `:probe::Color` is unresolved) |
| 4 | the probe's `#[ignore]`s are GONE | `grep -c '#\[ignore' tests/resolve/probe_arc251_stone9_symbol_head_declaration.rs` | `0` — the stone un-ignores its own rows |
| 5 | ⛔ the catch-alls are actually gone | `grep -cE '_ => (return )?(false\|None)' src/declare/parse.rs` | **strictly less than the pre-strike count**, and every remaining one is a SHAPE guard (`WatAST::List(...) => items`), never a head reader. A site that kept its `_` kept its silent path |
| 6 | the compiler's population, recorded | the SCORE | the real site list from removing the `_` arms, with **any site the BRIEF's room map missed named explicitly** as a finding |
| 7 | no keyword-headed behaviour moved | `./scripts/floor.sh` unpiped, Summary line | `5206+ passed, 0 failed` — every legacy program renders and resolves byte-identically (STOP-5) |
| 8 | clippy | `cargo clippy --release --all-targets --workspace` | 0 errors |
| 9 | names were NOT touched | `git diff --stat` | zero changes in `src/function/parse.rs`; no NAME reader in `parse.rs`/`register.rs` converted (STOP-1) |
| 10 | markers were NOT touched | `git diff` | `:guard`/`:ensure` (`function/parse.rs:417·426`) and the metadata-map key reader (`parse.rs:322`) unchanged (STOP-2) |

## RUNTIME PREDICTION

**20–35 min.** One door plus ~23 mechanical conversions the compiler enumerates. The risk is not
volume; it is a room that turns out to need the head for something other than a comparison (STOP-3),
which converts the stone into a report rather than a strike.

## TRAP DOORS — named before the strike

- **A green floor proves almost nothing here.** The corpus has ~18 symbol heads and none is a
  declaration, so the floor was green WITH the defect and will be green after. Rows 1–3 are the only
  rows that can see the change; row 7 only proves nothing REGRESSED.
- **`grep -c` on row 5 is a text count, not a census.** It is a ratchet on a number that must fall,
  paired with row 6's requirement to name the sites. Neither alone is sufficient — the count cannot
  tell "one converted, one added" from "nothing happened".
- **The room map is mine and this campaign got four censuses wrong.** Row 6 exists so a missed site
  is recorded as a finding rather than silently absorbed. Finding sites the map missed is the
  EXPECTED outcome, not a failure of the strike.
- **`ns_to_wat_path` is a resolution primitive with six non-codec callers** (`normalize.rs:413`,
  `types.rs:5060/5177`, `macros/expand.rs:587`, `declare/parse.rs:378/679`). This stone must not
  edit it — only call it. An edit there is a change to symbol resolution.
- **The two head lists are a trap that already sprang once.** `DECLARATION_HEADS` and
  `RUNTIME_DECLARATION_HEADS` answer different questions and their shared doc records the shipped
  bug from conflating them. A door that makes them "easy to unify" is the same defect wearing a
  refactor's clothes.
