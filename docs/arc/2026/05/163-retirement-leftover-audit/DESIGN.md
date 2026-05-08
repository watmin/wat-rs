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

### Slice 1 — `let*` retirement sweep — SHIPPED at `a8cc381`

Bucket A/B/C/D framework applied to arc 154's `let*` retirement.
132 sites cleared (243 → 111 residual; 111 are Bucket C/D).

### Slice 2 — Vec / list / Queue / stream consumer sweep — IN FLIGHT

Four remaining surfaces (`:Vec<`, `:wat::core::list`, Queue family,
`:wat::std::stream::*`). Bucket A consumer keyword sweeps — replace
legacy spellings with canonical names. Total ~168 sites pre-flight.

### Slice 3 — Substrate hard-retirement (NEW per user direction 2026-05-07)

User direction: *"Hard retire — kill typealiases."* Pre-flight
investigation confirmed the persisting `:Vec<T>` / `:wat::core::list`
expressions exist because the substrate keeps transitional
scaffolding:

- `src/types.rs`: `typealias :wat::core::Vector<T> = :Vec<T>` —
  legacy spelling parses cleanly (typealias)
- `src/runtime.rs:3088`: `":wat::core::list" => eval_list_ctor(...)`
  — runtime alias arm

Slice 3 retires this scaffolding per arc 154/155 Path B full-
retirement pattern:

1. Delete the typealias entry in `types.rs` (`:Vec<T>` no longer
   parses cleanly post-slice-3)
2. Delete the runtime alias arm for `:wat::core::list`
3. Add walker(s) — `BareLegacyVec` / `BareLegacyKernelList` — that
   fire Pattern 2 poison on legacy keyword usage (mirror arc 154's
   `BareLegacyLetStar` recipe)
4. Variant + Display preserved as orphaned scaffolding (arc 113
   precedent)

Pre-condition: slice 2 consumer sweep complete (zero in-tree consumer
keyword usage of legacy spellings; otherwise slice 3 substrate
retirement breaks the workspace).

### Slice 3e — Substrate container heads to FQDN — SHIPPED at `25860be`

User direction 2026-05-07: *"wat internals are fully qualified - no
exceptions... if there's a short form - its illegal... if the internal
code is mapping to a rust primitive then we use the rust form."*

Substrate stored `head: "Vec"`, `head: "Option"`, etc. — short forms.
These violated the FQDN rule for wat-internal storage. Slice 3e
rewrote substrate-internal container head strings to FQDN form
(`"wat::core::Vector"`, etc.), deleted the downgrade arm, retired
vestigial typealiases, and added a TEMPORARY canonicalize=true
upgrade arm (bare → FQDN) bridging fixtures still using bare-form
wat source. Waterfall 848 → 0 across 7 sweep iterations.

### Slice 3f — Substrate primitive paths to FQDN

Same rule, separate category. `":i64"` → `":wat::core::i64"` etc.
across substrate-internal storage. ~142 sites + 5 canonicalize arms
reshape.

Plus parallel work to slice 3e: Value::type_name primitive arms
flip back to FQDN once the substrate-internal storage is FQDN
(was reverted to bare in slice 3e to keep dispatch dispatch
matching aligned during slice 3e atomicity).

### Slice 3g — User-source bare primitive sweep

Original SURVEY slice 3f (renumbered after 3e/3f introduced for
substrate). Mass test-fixture sweep of bare `:i64`/`:f64`/`:String`/
`:bool` to FQDN. ~4040 sites in tree. Last because it leverages all
prior slice patterns + benefits from settled substrate foundation.

### Slice 3h — Retire canonicalize=true upgrade arms (GATES ARC CLOSURE)

The canonicalize=true upgrade arms in `parse_type_inner`
(container heads, src/types.rs:1683-1694; primitive paths, slice
3f extension) are TEMPORARY bridge scaffolding (same retirement
shape as arc 111's `arc_111_migration_hint`, per substrate-as-
teacher § "Retire the hint when its window closes").

After slice 3g closes (test-fixture wat sources all FQDN), no bare
raw_head/raw_path reaches `parse_type_inner` because:
- Walker rejects bare user-source as fatal
- Substrate Rust constructs FQDN strings directly
- Test fixtures use FQDN

At that point, slice 3h **deletes the upgrade arms**. The match
arms come out; raw_head/raw_path passes through unchanged. The
substrate is now FQDN-uniform without bridge scaffolding.

**Verification:** post-deletion `cargo test --release --workspace`
stays green (2041+/0). If any fixture or Rust site still constructs
bare-form, the build/test will surface it; that becomes a Bucket A
correction in the slice.

**Arc 163 closure depends on slice 3h shipping.** Per user direction
2026-05-07 (post-slice-3e): *"the current arc is closed when the
temporary state is removed."*

### Slice 3z — closure

INSCRIPTION + 058 changelog row. Sweep summary table:
"retired surface | pre-fix sites | Bucket A renamed | Bucket B
updated | Bucket C/D preserved | post-fix sites." Documents the
full waterfall + scaffolding-retirement pattern (FM 11 affirmative
language; INSCRIPTION = DONE).

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
