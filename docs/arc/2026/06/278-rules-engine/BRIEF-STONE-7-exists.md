# BRIEF — Stone 7-exists: `:exists` = NegationNode's filter, flipped (oracle + native + differential)

**You are a single-hop executor. Do NOT spawn sub-agents. Do NOT run git. Do NOT run `./target/release/wat`
(orchestrator-only; you MAY `cargo build`/`cargo test`).** Work ONLY in `/home/watmin/work/holon/wat-rs`.
**After ANY edit to `wat/rete.wat`, run a rete.wat-loading test** (`cargo test --release -p wat --test
probe_arc278_7a_negation_oracle`) — `cargo build` does NOT type-check wat.

## The work (one paragraph)
Add `(:wat::rete::exists <inner>)`: a LHS condition that passes its parent token **iff ≥1 element matches the
inner condition** for the token's bindings, **binds nothing, fires the token once** (no multiplicity). It is
`:not`/NegationNode with the filter predicate **inverted** (negation: pass iff ZERO compatible; exists: pass
iff ≥1). Build a **sibling `ExistsNode`** (additive — do NOT change `NegationNode`). Ship oracle + native +
the differential in this one strike. Contract: `DESIGN-STONE-7-exists.md`.

## Read in order (the rooms)
1. `DESIGN-STONE-7-exists.md` — the contract.
2. `wat/rete.wat`:
   - **`NegationNode` record (~`:109`) + `Node` enum (~`:148`) + `node-children` arm (~`:301`)** — mirror for
     `ExistsNode` (same shape: `id <- i64`, `alpha-id <- i64`, `children <- PV<i64>`). Use the same field
     names the NegationNode arm reads (it stores `negated-alpha-id`; for ExistsNode call it `alpha-id` or
     mirror exactly — your call, keep it consistent with the arm you write).
   - **`compile-condition` `:not` branch (~`:563`-`598`)** — the MODEL. Add `is-exists` detection beside
     `is-not` (`head-nm == ":wat::rete::exists"`), and an `is-exists` branch IDENTICAL to the `:not` branch
     except it mints an `ExistsNode` instead of a `NegationNode`. Keep the parent≥0 guard (leading `:exists`
     raises "exists must follow a binding condition"). Insert it in the `if`-chain (where → not → **exists** →
     accumulate → join-else).
   - **filter-pass `NegationNode` arm (~`:1217`-`1245`)** — the MODEL. Add an `ExistsNode` arm: SAME gather
     (`token-element-compatible?` over `alpha-memory[alpha-id]`), but pass the token iff **`any-compat`** is
     TRUE (the flip of negation's `(not any-compat)`). Bind nothing; pass the token unchanged once.
3. `src/rete/kernel.rs`:
   - **`node_children` `NegationNode` arm (`:446`)** — add an `ExistsNode` arm (children slot = same index as
     NegationNode's, `sf[2]`, IF you mirror the field order id/alpha/children).
   - **`fire_fixpoint_delta` filter-pass `NegationNode` branch (~`:1742`-`1790`)** — add an `ExistsNode`
     branch: SAME gather (`token_element_compatible` over `wm.alpha[alpha_id]`, `:725`), pass the token iff
     **≥1** compatible (the flip). Mirror exactly; only the verdict inverts.
4. `tests/probe_arc278_7exists_native_differential.rs` — the contract, RED now (5 tests). Do NOT weaken it.

## The flip, stated once
Negation arm: `if not any-compat -> pass`. Exists arm: `if any-compat -> pass`. Everything else (the gather,
the bind-nothing, the once-per-token) is identical. That is the entire semantic difference.

## Blast radius (bounded)
`wat/rete.wat` + `src/rete/kernel.rs` ONLY. No `matcher.rs`/`runtime.rs`/`check.rs`. No new intrinsic. Do NOT
touch `NegationNode`.

## STOP triggers (halt + surface; do not improvise)
1. If detecting `head-nm == ":wat::rete::exists"` collides with where/`:not`/accumulate detection — STOP.
2. If `find-or-mint-alpha` / `token-element-compatible?` / `token_element_compatible` can't be reused — STOP.
3. If the differential stays RED (native ≠ oracle) and you can't localize it — STOP; report native vs oracle
   counts + hypothesis. Do NOT weaken the probe.
4. If greening needs anything beyond `wat/rete.wat` + `kernel.rs` — STOP.

## Done = green
`cargo test --release -p wat --test probe_arc278_7exists_native_differential` → 5/5 (native==oracle). AND no
regressions: `--test probe_arc278_7a_negation_oracle` → 3/3 ; `--test probe_arc278_7b_negation_native_differential`
→ 4/4 ; `--test probe_arc278_8b_accumulate_native_differential` → 5/5 ; `--test
probe_arc278_northstar_cold_and_windy -- --include-ignored` → 1/0. Then `cargo build --release` clean +
`cargo test --release -p wat --lib -- --test-threads=1 | grep result` → 941/36.

## Report back
The exact diffs to `wat/rete.wat` + `src/rete/kernel.rs`, every test count from Done (verbatim from your runs),
and any STOP. Your final message is all I see.
