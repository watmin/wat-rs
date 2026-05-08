# Arc 163 — Retirement leftover audit (cross-arc Bucket B sweep)

**Status:** queued 2026-05-07. Not yet started.

**Gates:** none. Open after arc 162 closes.

## Background

Arc 162 closed the lambda → fn internal-identifier leftover. User
direction 2026-05-07: *"once this sweep is complete.. we need an
audit for other things we've removed.. i found `let*` refs when we
killed it in favor of just `let`. there's been a ton of refactor
work that we've been slowly grinding through."*

Same pattern as arc 162: a user-facing retirement arc shipped, the
internal-identifier sweep was scoped out, and ~months later a leftover
gets noticed. FM 14 (recovery doc) codifies the discipline going
forward; arc 163 sweeps the historical accumulated leftovers.

## Scope

For each prior retirement arc, run the audit grep (live identifiers,
comment text describing live concept) and apply Bucket A/B/C/D
classification per arc 162's framework.

### Retirement arcs to audit

A non-exhaustive starting list (sonnet expands during the audit):

| Arc | Retired surface | Likely identifier leftover keyword |
|---|---|---|
| 154 | `:wat::core::let*` → `:wat::core::let` (sequential) | `let_star`, `let*` in comments |
| 153 | `:wat::core::unit` → `:wat::core::nil` | `unit`, `Unit` in comments where they referred to the value |
| 109 slice 1d | `:()` retired as type | `Tuple([])` historical naming |
| 155 | `:wat::core::lambda` + `:fn(...)` → `:wat::core::Fn` | done by arc 162 |
| 109 family | `:wat::std::*` namespace | `wat__std__*`, `std::` references in comments |
| 109 slice 1f | `Vec<T>` → `Vector` | `Vec<` in non-Rust-semantic comments |
| 109 slice 1g | list retires; `Tuple` mints | `list_*` identifier names |
| 109 slice 1h | Option variants FQDN | `:Some` / `:None` bare-keyword refs |
| 109 slice 1i | Result variants FQDN | `:Ok` / `:Err` bare-keyword refs |
| 109 slice K.kernel-channel | Queue* → Channel/Sender/Receiver | `Queue*` identifiers in code/comments |
| 091 slice 8 | quasiquote retired? | TBD |
| 102 | arc 066 wrap reverted | `wrapped_holon_ast` etc. |
| 114 | spawn's `R` parameter | `R` in spawn-related code/comments |
| 138 | Errors got coordinates | TBD |
| 145 | typed-let backout | `:T` annotation in let comments |
| 146 | Multimethod → Dispatch (slice 1b) | `multimethod`, `Multimethod` |
| 159 | per-binding `:T` from let | `LegacyTypedLetBinding`-context ✓ done |
| (others) | TBD | Sonnet expands the list during audit |

### Audit workflow per retired surface

For each `<retired_surface>`:

1. **Audit grep**:
   ```bash
   grep -rn "<retired_keyword>" --include="*.rs" --include="*.wat" wat-rs/ | wc -l
   ```
2. **Classify hits** per Bucket A/B/C/D (see arc 162 BRIEF-SLICE-1):
   - **A**: live identifiers — RENAME (rare for already-retired
     surfaces; usually arc 162-style cleanup already happened)
   - **B**: comment text using legacy name as live concept — UPDATE
   - **C**: retirement-context comments — KEEP
   - **D**: orphaned scaffolding (variants + Display) — KEEP
3. **Sweep B** items.
4. **Verify** (cargo build + test + grep counts post-fix).

## Slice plan

### Slice 1 — audit pass + targeted Bucket B sweep

Sonnet:
1. For each retired surface in the table above (and any others
   sonnet finds in arc INSCRIPTIONs), run the audit grep.
2. Classify hits and apply Bucket B updates.
3. Report per-surface counts pre/post.

Estimated ~60-90 min sonnet wall-clock for the full sweep.

### Slice 2 — closure

INSCRIPTION + 058 changelog row. Sweep summary table:
"retired surface | pre-fix sites | Bucket A renamed | Bucket B
updated | Bucket C/D preserved | post-fix sites."

## Sources

- Arc INSCRIPTIONs in `docs/arc/2026/04/` and `docs/arc/2026/05/`
  for the canonical retirement record per arc.
- 058 FOUNDATION-CHANGELOG (in lab repo) for compact summaries.
- `BareLegacy*` variants in `src/check.rs` are the substrate-side
  ledger of what got retired; grep that list for clues.

## Why arc 163 is the right shape

Four questions:
- Obvious — same pattern as arc 162; just applied to the historical
  accumulated set
- Simple — mechanical audit + sweep per surface
- Honest — closes the FM 14 backlog
- Good UX — every reader sees consistent vocabulary; no "wait, that
  was retired, why is it still here?" surprises

## Cross-references

- **Arc 162** — first-instance of this pattern (lambda → fn)
- **Recovery doc § 6 FM 14** — discipline going forward
- **Memory `feedback_surface_retirement_internals.md`** — codified
  pattern
- **Memory `feedback_verify_sonnet_tool_claims.md`** — sonnet should
  run audit greps directly (verify Bash via 30-sec probe if it
  hesitates)

## When this opens

After arc 162 closes (slice 2 + slice 2 closure paperwork ship).
