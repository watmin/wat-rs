# EXPECTATIONS — Stone 249.5f: canonical scope renumbering at hash time

Written BEFORE the strike. The Score grades against THIS, re-run independently.

## Scorecard

| # | What | Command | Expected |
|---|---|---|---|
| 1 | Cross-run hash determinism for macro programs | `cargo test --release --test probe_hash_scope_renumber` | 2 passed (bug + guard) |
| 2 | The discrimination guard holds | (same run) | `distinct_scope_structure_hashes_differently` passes — renumber, not strip |
| 3 | The caveat is retired | `grep -n "REMAINS deferred\|separable follow-on\|not across runs" src/hash.rs` | 0 hits |
| 4 | Renumberer is private, no API change | `grep -n "pub fn canonical_edn\|pub fn hash_canonical" src/hash.rs` | signatures unchanged; `ScopeRenumber` not `pub` |
| 5 | Prior hygiene contracts hold | `cargo test --release --test probe_macro_hygiene_capture --test probe_argspec_rest_param_hygiene --test probe_check_scoped_param_resolution` | 2 + 1 + 2 = 5 passed |
| 6 | Library suite — no regressions | `cargo test --release --lib -p wat` | ≥ 907 passed, 0 failed (non-macro hashes byte-identical) |
| 7 | Bounded blast radius | `git diff --stat` | only `src/hash.rs` + the probe |

Rows 1, 2, 5, 6 load-bearing (orchestrator re-run). 3, 4, 7 discipline guards.

## Runtime prediction

6–10 min (Mode A). Self-contained in one file; the only care points are threading
`&mut ScopeRenumber` through every `write_canonical_wat` recursion arm (the build
enforces completeness) and retiring the caveat doc honestly.

## Trap-doors named

- **The discrimination guard is the anti-strip witness.** A naive "drop all scopes"
  passes the bug test but fails the guard (collapses capture into non-capture). Row 2
  is the proof the fix is a canonical renumber. If it's red, the fix is wrong even if
  the bug test is green.
- **Non-macro hashes must NOT move.** Empty scope sets emit zero scope bytes → the
  renumberer is never consulted → byte-identical output. The lib suite (907) is the
  witness; any movement means the renumberer is touching the non-scoped path.
- **Program-wide numbering.** `canonical_edn_program` must use ONE renumberer across
  all forms (a scope shared between two top-level forms gets one canonical index). A
  per-form renumberer would be wrong for cross-form scope sharing — unlikely in
  practice but structurally incorrect. The Score confirms one renumberer per program.

## What this stone closes

The macro-hygiene class, in full: **runtime keying** (249.5b/d, `env_key`) +
**check keying** (249.5e, `env_key`) + **hash identity** (249.5f, canonical
renumber). After this, every site that consumes an `Identifier`'s scopes —
resolution at eval, resolution at check, identity at hash — is scope-aware, and the
hash one is deterministic across runs. No identifier-keying site left name-only or
non-deterministic. The 249.5 hygiene completion is genuinely complete (modulo the
ward-close paperwork).
