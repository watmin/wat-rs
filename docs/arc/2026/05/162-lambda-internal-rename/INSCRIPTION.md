# Arc 162 — INSCRIPTION

**Inscribed 2026-05-07 by orchestrator.** All slices shipped.

## What shipped

Arc 155 retired the user-facing `:wat::core::lambda` keyword in
favor of `:wat::core::fn`. The Rust-level identifier sweep was
deliberately scoped out of arc 155, leaving ~353 `lambda`/`Lambda`
references in source. Arc 162 closes that gap.

User direction 2026-05-07: *"let's clean up the lambda refs - i
wasn't happy seeing left overs in the source... we need to make
sure we don't leave confusion when we do these clean ups."*

## Slices

| Slice | Commit | What landed |
|---|---|---|
| 1 (orchestrator manual) | `a91e940`, `f295742` | Bucket A live-identifier rename: `Value::wat__core__lambda` → `Value::wat__core__fn`; `WatLambdaSigmaFn` → `WatFnSigmaFn` (public type); `parse_lambda_signature*` (×2) → `parse_fn_signature*`; walker helpers; type-name strings; debug `<lambda@span>` → `<fn@span>`; callee labels; `tests/wat_spawn_lambda.rs` → `tests/wat_spawn_fn.rs`; 3 test-fn renames; first wave of Bucket B comment text |
| 2 (sonnet edits-only) | `b489fb2` | Bucket B sweep across 34 files (~158 sites) + 2 Bucket A residuals slice 1's audit grep missed (`name_from_keyword_or_lambda`, `require_lambda`, 5× `format!("<lambda@{}>")`) |
| 3 | (this commit) | Closure paperwork |

## Substrate impact

| Audit grep | Pre-arc-162 | Post-slice-1 | Post-slice-2 |
|---|---|---|---|
| Total `lambda`/`Lambda` | 353 | 250 | 92 |
| Bucket A live identifiers | ~46 | 0 | 0 |
| `BareLegacyLambda` (Bucket D, preserved) | 28 | 28 | 28 |
| Workspace pass/fail | 2041/0 | 2041/0 | 2041/0 |

The 92 residual sites are all Bucket C (arc 155 retirement comments
referencing the retired keyword by name as historical context) or
Bucket D (`BareLegacyLambda` variant + Display arms preserved per
arc 113 orphaned-scaffolding precedent).

## Settled design

### The Bucket A/B/C/D classification framework

Arc 162's BRIEF-SLICE-1 codified four buckets that every `lambda`/
`Lambda` site falls into. This framework is now the canonical
orientation device for any future surface-retirement-leftover sweep
(see arc 163's queued audit doc):

- **A — RENAME**: live identifiers using legacy name as concept
  (Rust types, fns, vars, public APIs, debug strings, type-name
  strings, test fn names, file names)
- **B — UPDATE**: comment text using legacy name in present tense
  as if the concept is current
- **C — KEEP**: comments recording the retirement (historical
  context — "arc N retired X", "(formerly Y)"). Per
  `feedback_inscription_immutable.md`, historical record stays.
- **D — KEEP**: orphaned scaffolding per arc 113 precedent —
  variants + Display arms naming the legacy form they reject

### Why the work split as it did

Slice 1 was orchestrator manual because the first sonnet spawn
returned in 39s with a false "Bash permission denied" claim. I
took the claim at face value (FM 7 failure pattern) and ran the
rename myself with full Bash access. Slice 2 caught the discipline
failure: a 30-second verification probe (a tiny sonnet whose only
job was `which cargo && cargo --version`) confirmed Bash works
fine for sub-agents in this environment. The two prior denials
were sonnet hallucinations triggered by brief language mentioning
Bash skepticism.

Slice 2 then ran sonnet in edits-only mode (sonnet does Read/Edit;
orchestrator runs cargo verifies). 158 Bucket B sites updated +
2 Bucket A residuals caught that slice 1's audit grep missed.

Memory saved: `feedback_verify_sonnet_tool_claims.md` codifies
the 30-second-probe discipline. MEMORY.md indexed.

## Honest deltas

1. **Slice 1's Bucket A audit grep had blind spots.** It targeted
   `wat__core__lambda | WatLambda | parse_lambda_signature |
   _lambda_body_ | rhs_spawn_lambda` — a narrow surface. Slice 2
   discovered 2 more live identifiers (`name_from_keyword_or_lambda`,
   `require_lambda`) and 5 format-string sites (`format!("<lambda@{}>")`)
   not covered. **Future arcs (arc 163) should use a broader audit
   grep** — e.g., `\b\w*lambda\w*\b` to catch any token containing
   "lambda" in any position.

2. **Token-burn lesson.** Slice 1's manual orchestrator path cost
   ~30 min of Opus tokens that should have been sonnet's. Root cause:
   I accepted sonnet's false tool-unavailability claim without
   verification. Per recovery doc § 7 (preexisting): *"Empirically
   verify before accepting workarounds rooted in tool-unavailability
   claims."* The discipline existed; I didn't apply it. Memory
   `feedback_verify_sonnet_tool_claims.md` captures the recovery
   pattern (30-sec probe). Sibling new failure mode added to
   recovery doc as FM 14 codifying the surface-retirement-leftover
   discipline going forward.

3. **No test fn name needed legacy-form preservation.** Sonnet
   audited and found no `lambda_post_retirement_*` style names —
   all test fn names containing "lambda" described live behavior,
   not retirement-specific behavior, so all renamed to "fn".

4. **No hybrid sentences.** Sonnet found zero comments mixing
   live-concept use + retirement context in one sentence. Every
   site classified cleanly.

5. **`tests/wat_arc155_fn_rename.rs` untouched.** That file is
   a 36-site test fixture verifying arc 155's retirement diagnostic
   fires. Bucket D — preserved exactly.

## Tests

No new tests added by arc 162. Existing tests verify:
- The rename compiles (Rust compiler caught every match arm
  via cascading errors during slice 1's variant rename)
- Workspace stays green: 2041 passed / 0 failed throughout

## Out of scope

- **Arc 163 — retirement leftover audit** (queued; opens immediately).
  User flagged that arc 154 (`let*` → `let`), arc 153
  (`unit` → `nil`), and other historical retirement arcs likely
  have similar Bucket B leftovers. Arc 163 systematically applies
  the same Bucket A/B/C/D framework across the historical record.
  DESIGN at `docs/arc/2026/05/163-retirement-leftover-audit/DESIGN.md`.

## Cross-references

- **Arc 155** — original surface retirement that scoped out internals
- **Arc 113** — orphaned-scaffolding precedent (variant names + Display)
- **Arc 154** — companion surface retirement (`let*`); arc 163 audits its leftovers
- **Recovery doc § 6 FM 14** — surface retirement leaving internal
  identifiers as leftovers (codified in this session)
- **Recovery doc § 7 FM 7** — sonnet falsely claiming tool
  unavailability (lesson re-applied)
- **Memory `feedback_surface_retirement_internals.md`** — the
  discipline going forward
- **Memory `feedback_verify_sonnet_tool_claims.md`** — the 30-sec
  probe pattern

## Commit chain

- `93a20a6` arc 162 opens (BRIEF + EXPECTATIONS)
- `a91e940` arc 162 slice 1: Bucket A live-identifier rename
- `f295742` arc 162 slice 1 (cont'd): more Bucket B comment text sweep
- `c135eac` arc 162 slice 2 BRIEF
- `0d18d07` arc 163 queued: retirement leftover audit
- `b489fb2` arc 162 slice 2: Bucket B sweep across tests + comments + residual A
- (this commit) arc 162 slice 3: closure paperwork
