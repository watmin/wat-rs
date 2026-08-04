# EXPECTATIONS — #74, `<Op>Response` is law

Written BEFORE the strike so the result cannot move the goalposts. Every row is scored against the
orchestrator's **own** re-run, never the rider's report.

## Scorecard

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | the law is real | `target/release/wat --check` on a fresh non-conforming serviceable surface | REFUSED, located, printing BOTH the declared and the required name |
| 2 | **the law is not vacuous** | the same file with the name corrected | ACCEPTED, silent |
| 3 | **the law does not over-reach** | a `:holder :wat::core::Struct` surface with `(tag [self <- :S] -> :wat::core::String)` | ACCEPTED — a non-serviceable surface's methods are in-thread accessors, not wire ops |
| 4 | ★ the acronym rule is the rule | `tests/macros/probe_arc265_acronym_registry_svc.wat` (`create-web-acl → CreateWebACLResponse`) | ACCEPTED. A naive pascal-caser refuses this; if it is refused, the check called the wrong converter |
| 5 | ★ the BASE name is what is compared | `wat-tests/service-parametric-messages.wat` (`GetResponse<K,V>`) + `src/types/surface.rs:1115` (`:t::Cache::GetResponse<V>`) | ACCEPTED. This is #75's class; comparing a rendered type instead of a base name fails here and only here |
| 6 | the census was right | the rider's report of everything the armed check refused | EXACTLY the ten rows of the brief's table — no more, no fewer |
| 7 | the emitter is gone | `grep -rn 'build_op_response_type_constants\|RESPONSE-TYPE' src/ wat/ tests/ wat-tests/ wat-scripts/ crates/` | zero hits outside prose that records the retirement |
| 8 | the decode branches are gone | `grep -n 'resp-dotted\|rtl-edn\|rm-edn' wat/service.wat` | zero hits |
| 9 | the literal ctor works on real services | `cargo nextest run --release -E 'test(parametric)'` | green — the parametric services exercise the restored literal ctor on the wire |
| 10 | the two negative controls INVERT | `probe_arc278_response_type_from_declaration` + the repl-durable-forms probe | both assert REFUSAL and pass; neither is deleted, neither is migrated into conformance |
| 11 | the loader gate is green | `cargo nextest run --release -E 'test(every_wat_scripts_file_loads)'` | green — this is the gate that caught R64; it must still parse and type-check every scratch file |
| 12 | ★ the whole floor | `cargo nextest run --release`, **Summary line** | `4348 run / 4348 passed / 0 failed / 262 skipped` — identical to the pre-change floor |
| 13 | clippy | `cargo clippy --release --all-targets` | clean; `-D warnings` is armed in CI |
| 14 | net deletion | `git diff --stat` | **negative**. This stone deletes a Rust emitter, a runtime constant, and four decode blocks, and adds one check |

Rows 4, 5 and 12 are load-bearing and get re-run by hand regardless of what the rider reports.

## Runtime prediction

**25–40 minutes.** Two `cargo build --release` cycles plus one full `nextest --release` dominate.
Time-box at 2× the upper bound (80 min).

## Trap doors — named in advance so a hit is data, not a surprise

1. **The pascal converter.** Calling a naive kebab→pascal instead of
   `kebab_to_pascal_with_acronyms` refuses `CreateWebACLResponse`. The orchestrator made exactly
   this error while measuring this stone. Row 4 is its detector.
2. **Base name vs rendered type.** `GetResponse<K,V>` conforms. Comparing the rendered type refuses
   every parametric service. Three instances of this class in three days (#75). Row 5 is its
   detector.
3. **The colon asymmetry.** `TypeExpr::Path` carries a leading `:`; `Parametric.head` does not, and
   that is deliberate, not a bug to fix upstream (`types.rs:3117-3131`). Normalize at the read site
   or the comparison is off by one character on every parametric response.
4. **The peer gate.** A check placed in the alias-minting loop (`types.rs:3251`) is not peer-gated
   and will refuse ordinary struct-surface accessors. Row 3 is its detector.
5. **`:max-request-bytes` collateral.** The mint of three new Response enums touches surfaces whose
   ops must also carry `:max-request-bytes`. If a migrated probe suddenly fails that sibling lock,
   it is because it was previously escaping the whole synthesis via the request-arg bail — real,
   and in scope.
6. **The stale-binary trap.** `target/release/wat` carries a baked stdlib. Any `--check` run against
   an unrebuilt binary after touching `wat/service.wat` is an instrument artifact. Rebuild first.

## What would make this a FAILURE rather than a delta

- The floor is not exactly `4348/4348/0/262`, and the difference is not explained line by line.
- The check refuses anything the census did not predict, and it was migrated instead of surfaced.
- `git diff --stat` is net positive — that would mean the deletion half did not happen and only the
  check landed, which is the more expensive half of the stone shipped alone.
- Either negative control was deleted or migrated into conformance. Both must survive as proof of
  the wall.
