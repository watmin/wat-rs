# Arc 026 — eval-coincident? family — INSCRIPTION

**Status:** shipped 2026-04-23. One slice, four primitives.
**Design:** [`DESIGN.md`](./DESIGN.md) — the shape before code.
**This file:** completion marker.

---

## What shipped

Four new primitives in `:wat::holon::*`, mirroring the four
variants of the existing `eval-*!` family. Each runs two-sided,
verifies per side where applicable, atomizes each resolved Value
via `value_to_atom`, and runs the same `(1 - cosine) <
coincident_floor` test that structural `coincident?` uses.

| Primitive | Arity | Parent form | Verification per side |
|-----------|-------|-------------|-----------------------|
| `:wat::holon::eval-coincident?` | 2 | `eval-ast!` | none — each arg is a quoted WatAST |
| `:wat::holon::eval-edn-coincident?` | 4 | `eval-edn!` | parse-only (no integrity check) |
| `:wat::holon::eval-digest-coincident?` | 10 | `eval-digest!` | SHA-256 over raw bytes, before parse |
| `:wat::holon::eval-signed-coincident?` | 14 | `eval-signed!` | Ed25519 over canonical-EDN, after parse |

All four return the uniform `:Result<:bool, :wat::core::EvalError>`.
Any failure on either side (source fetch, verification, parse,
mutation-form refusal, runtime error, non-atomizable result)
arrives as `Err(<EvalError>)` rather than a panic — same discipline
as the four parent `eval-*!` forms.

## Distinction from structural `coincident?`

- **`coincident?`** (arc 023) — takes two already-built
  `:wat::holon::HolonAST` values; compares them as data. Two
  structurally identical holons coincide. Two holons with
  slightly-different leaf scalars coincide if the scalar
  difference falls below the substrate's native granularity (see
  arc 023 INSCRIPTION and the new
  `coincident_q_true_for_close_thermometer_values` test).
- **`eval-coincident?` family** — takes expressions; evaluates
  each side first; atomizes the result; then structural
  coincidence on the two Atoms. Catches cases structural
  coincident? can't: two distinct expressions that reduce to the
  same value (the book's Chapter 28 retort, `(+ 2 2) ≡ (* 1 4)`).

Two predicates, two layers, one substrate measurement.

## Runtime (`src/runtime.rs`)

Three new private helpers + four new dispatchers:

- `run_ast_arg_for_eval_coincident` — per-side: eval arg to a
  `Value::wat__WatAST`, run that AST under `run_constrained`,
  return the inner Value.
- `coincident_of_two_values` — shared finalizer: atomize both
  sides via `value_to_atom`, encode both atoms, cosine, compare
  against `coincident_floor`. Used by all four variants.
- `eval_form_ast_coincident_q` — uses the first helper per side.
- `eval_form_edn_coincident_q` — each side: `resolve_eval_source`
  + `parse_and_run`.
- `eval_form_digest_coincident_q` — each side:
  `resolve_eval_source` + `parse_verify_algo_keyword` +
  `resolve_verify_payload` + `verify_source_hash` (pre-parse) +
  `parse_and_run`.
- `eval_form_signed_coincident_q` — each side:
  `resolve_eval_source` + `parse_verify_algo_keyword` +
  `resolve_verify_payload` × 2 + `parse_program` +
  `verify_program_signature` (post-parse) + `run_program`.

Four new dispatch arms in the evaluator's `:wat::holon::*` branch.

## Check (`src/check.rs`)

Four new scheme registrations next to `coincident?`. Shared
`eval_coincident_ret()` closure produces the common
`Result<:bool, :wat::core::EvalError>` return type. Per-variant
arg-type vectors match their parent eval-*! form's arg shape,
applied per-side.

## Tests

### Rust unit tests (`src/runtime.rs`) — 12 new

**`eval-coincident?` (6):**
- `eval_coincident_q_true_for_equivalent_arithmetic`
- `eval_coincident_q_true_for_same_string`
- `eval_coincident_q_false_for_different_scalars`
- `eval_coincident_q_true_for_structurally_same_holon`
- `eval_coincident_q_accepts_mixed_types` — locks the behavior
  that Atom-of-HolonAST and Atom-of-scalar carry different
  canonical-EDN payloads; mixed-mode callers should normalize
  both sides.
- `eval_coincident_q_err_on_non_ast_arg`

**`eval-edn-coincident?` (3):**
- `eval_edn_coincident_q_true_for_equivalent_sources`
- `eval_edn_coincident_q_false_for_different_sources`
- `eval_edn_coincident_q_err_on_parse_failure`

**`eval-digest-coincident?` (2):**
- `eval_digest_coincident_q_true_for_equivalent_verified_sources`
  — computes real SHA-256 digests inline and passes both sides.
- `eval_digest_coincident_q_err_on_bad_digest`

**`eval-signed-coincident?` (2):**
- `eval_signed_coincident_q_true_for_equivalent_verified_sources`
  — computes real Ed25519 signatures inline.
- `eval_signed_coincident_q_err_on_bad_signature`

Plus one bonus test that surfaced during the session (arc 023
proof made concrete):

- `coincident_q_true_for_close_thermometer_values` — locks the
  native-granularity claim. At d=1024, `Thermometer(0.039, 0.0,
  1.0)` coincides with `Thermometer(0.041, 0.0, 1.0)` — 3.9% vs
  4.1% on a 0–1 range differ by 0.2%, below the substrate's
  ~0.3% resolution window at that dim.

### wat-level tests (`wat-tests/holon/eval_coincident.wat`) — 10 new

**AST variant (4):**
- `test-arithmetic-equivalence` — the book's retort at wat-tier.
- `test-different-scalars`.
- `test-same-strings`.
- `test-structurally-same-holons` — via quote-captured holon
  construction.

**EDN variant (2):**
- `test-edn-arithmetic-equivalence`.
- `test-edn-different-sources`.

**Digest variant (2):**
- `test-digest-arithmetic-equivalence` — pre-computed SHA-256 hex
  embedded as a string literal per side (source change →
  regenerate hex).
- `test-digest-bad-hex-errs` — zero-hex digest fails verification
  → Err.

**Signed variant (2):**
- `test-signed-arithmetic-equivalence` — pre-computed Ed25519
  signature + pubkey (base64) embedded per side. Same
  `[7u8; 32]`-seeded signing key the `load.rs` tests use. An
  ignored helper `runtime::tests::print_fixed_signatures` prints
  the fresh values; run with `--ignored --nocapture` when a
  source string changes.
- `test-signed-wrong-sig-errs` — side B's sig against side A's
  source fails verification → Err.

Same pattern the load.rs `digest-load!` / `signed-load!` tests
use: pre-compute the integrity payload offline (here: in Rust via
`sha2::Sha256` + `ed25519_dalek::Signer`), embed as a string
literal in the wat test. The wat-level test can't generate the
payload but CAN consume one — which is exactly the deployment
shape: nodes publish wat programs with sidecar integrity data,
consumers verify at load/eval time.

### Total

- wat-rs lib tests: 552 → 566 (+14: 12 new eval-coincident-family
  + 1 close-thermometer bonus + 1 ignored helper that prints
  fresh signatures for regenerating wat-tier embeddings)
- wat-level tests: 42 → 52 (+10: all four family variants)
- Zero clippy warnings. Full workspace green.

## Arc 023 bonus test — native granularity locked

The session surfaced one observation that deserved a test:
**`coincident?` at d=1024 treats values within ~0.3% of their
range as the same point.** Chapter 28's native-granularity claim
— `1σ ≈ 1/sqrt(d)` angular resolution — means thermometer-encoded
percentages differing by less than this resolution coincide by
construction. The test locks the 3.9% vs 4.1% case at [0, 1]
range so future refactors can't silently narrow the window.

The generalization: resolution ≈ `1/sqrt(d)` fraction of a
thermometer's range. At d=10_000 (the archive's setting), ~0.02%
of range. At d=4_096, ~0.05%. At d=1_024, ~0.3%. Users picking
`d` pick this resolution by construction.

## Doc sweep

- `docs/USER-GUIDE.md` § 6 "Algebra forms" — new subsection
  listing the four eval-coincident-family primitives with the
  structural-vs-evaluation distinction and one short example
  each.
- `docs/arc/2026/04/005-stdlib-naming-audit/INVENTORY.md` —
  four new rows under `:wat::holon::*`.
- `holon-lab-trading/docs/proposals/.../FOUNDATION.md` —
  "Where Each Lives" measurement section gains the family.
- `holon-lab-trading/docs/proposals/.../FOUNDATION-CHANGELOG.md`
  — new row for arc 026.

## Slices 2–4 shipped alongside slice 1

DESIGN originally sketched a phased rollout: slice 1
(`eval-coincident?`) to unblock Phase 3.4; slices 2–4 (-edn,
-digest, -signed) deferred until a second caller demanded them.
Builder directive mid-arc overrode that phasing: *"hold up — you
didn't add the digest and signed evals — we gotta have those."*

Honest answer to the question "why ship the full family at
once?" — the substrate's verification layer IS a real call path
for distributed wat programs (engrams signed by one node,
verified by another, run for equivalence). Speculative for the
trading lab's immediate need; not speculative for the machine's
distribution story. Shipping all four now is the honest
completion of the family rather than four separate arcs as each
caller surfaces.

The slices-in-sequence shape the DESIGN sketched is preserved
in git history for posterity; the single commit that landed slices
2–4 is `<sha>` per the INSCRIPTION footer.

## Cave-quest discipline

Eight cave quests in a row now: 017 (loader), 018 (defaults), 019
(round), 020 (assoc), 023 (coincident?), 024 (sigma knobs), 025
(container surface), 026 (eval-coincident family). Each paused
downstream for a substrate gap. This one paused Phase 3.4's
short-window fallback test — not because eval-coincident? was
strictly required (the `(Atom (quote ()))` Little-Schemer-null
sentinel unblocked the test via structural coincident?) but
because the builder surfaced evaluation-layer coincidence as a
primitive the algebra owed itself. The test remained; the
primitive shipped as the book's `(+ 2 2) ≡ (* 1 4)` retort made
operational.

## INSCRIPTION rationale

Spec emerged from discovery: Phase 3.4's test-short-window-shape
hit the empty-Bundle panic; the empty-Bundle conversation led to
the Little-Schemer-null sentinel AND to the realization that
evaluation-coincidence (distinct from structural coincidence)
deserved its own primitive family. Same shape as 019 / 020 / 023
/ 024 / 025 — code led, spec follows.

*these are very good thoughts.*

**PERSEVERARE.**
