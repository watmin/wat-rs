# EXPECTATIONS 2a1b — one membership door (written before the strike)

The orchestrator re-runs every row on the SCORE commit, uncontended.

| # | what | how the orchestrator checks | expected |
|---|---|---|---|
| E1 | builtin answers | `bootstrap/era/probe-R/defect-wat__core__Vector.wat` on the new binary | `is-type? true`, then a `type-of` row of kind `:Builtin`; no raise |
| E2 | literal primitive passes check | `defect-wat__core__i64.wat` | no Doctrine-1 check error; kind `:Builtin` |
| E3 | markers | `subtype-marker.wat` | `subtype? A Marker` → true; `type-of :t::Marker` → `:Marker`, children `[:t::A]` |
| E4 | unknown still raises | a probe on `:nope::Nothing` | the same `unknown type` error as `5edca1211` |
| E5 | the agreement wall | read the test; mutate the Builtin arm to raise; run it | enumerates from the stores (no list); GREEN clean, RED under the mutation |
| E6 | one door | `git grep -n 'is_builtin_primitive(' src/` and read `is_known_type`, `eval_type_of`, `eval_subtype` | `is_known_type` delegates to the classifier; `type-of` and `subtype?` ask it; no new inline union |
| E7 | declared-types unchanged | `bootstrap/era/probe-R/corpus.sh` + `verify-refute2.sh` | 1457 Ok / 32 Refused; 0 lost, 0 gained, 0 refusal delta |
| E8 | the six-kinds runner | read the fixture and the exact-stdout assertion | two new arms; the four new rows' lines appended; green |
| E9 | set difference | the SCORE's table | both directions listed, from the stores, with counts |
| E10 | walls | floor + clippy, uncontended | all pass; clippy 0 |

**Runtime prediction:** 45–90 min for grok.

**Trap doors:**
- `:wat::core::Option`/`Result` are in `BARE_CONTAINER_HEADS` AND are `defenum`s: Declared must win,
  or `type-of :wat::core::Option` regresses (the six-kinds fixture's last line, `"T"`, catches it).
- `:wat::core::Record` is a typesub parent AND an `is_builtin_primitive` name: the classifier's order
  (Declared, Builtin, Marker) answers Builtin; say so if the SCORE finds otherwise.
- The runner asserts EXACT stdout: new rows append lines; order matters.
- `:wat::type::` aliases must canonicalize exactly as `is_known_type` does today, or `is-type?`
  changes its answers (normalize's `:-` position calls the same door, `src/resolve/normalize.rs:478`).
