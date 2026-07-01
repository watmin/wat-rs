# EXPECTATIONS — 296 N3: per-phase tag namespaces

Independent scorecard, fixed BEFORE the strike. The orchestrator re-runs each row itself and weighs the emitted wire EDN
+ the golden diffs (every golden change must be a pure `#wat.kernel/`→`#wat.<phase>/` prefix swap), not the report.

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | the single-source module exists | `grep -c "pub const" src/error_ns.rs` | 10 consts |
| 2 | the derive gained the `namespace` sub-key | `grep -n "namespace" crates/wat-macros/src/to_edn_derive.rs` | the sub-key parse + `#namespace_tokens` emit |
| 3 | the 7 families annotated | `grep -rn "to_edn(namespace = crate::error_ns::" src/` | 7 (Config/Check/Type/Stdlib/Load/Runtime/Macro) |
| 4 | RED probe un-ignored + GREEN | `cargo nextest run --release -E 'test(error_families_tag_under_their_phase_namespace)'` | 1 passed |
| 5 | phase namespaces on the wire | orchestrator captures a CheckError + RuntimeError + LoadError EDN | `#wat.check/…`, `#wat.runtime/…`, `#wat.load/…` with nested `#wat.kernel/NotFound` |
| 6 | REFACTOR GUARANTEE — no scattered ns literals in production | `grep -rn '"wat\.\(check\|runtime\|macro\|type\|parse\|config\|load\|resolve\|stdlib\)"' src/ crates/wat-macros/src/ --include=*.rs \| grep -v error_ns.rs \| grep -v '/tests/'` | 0 (only error_ns.rs holds them) |
| 7 | Failure/ProcessDiedError untouched | `grep -rn "wat.kernel/Failure\|wat.kernel/ProcessDiedError" src/ tests/` | still `#wat.kernel/…` (rides the de-stringify strike) |
| 8 | full gate | `cargo nextest run --release` | 0 failed |
| 9 | clean build | `cargo build --release` | clean; warning delta ~0 |

## Independent prediction
- **Runtime:** 25–45 min. The mechanism is small (one module + one derive sub-key + 7 annotations + 6 wrapper edits),
  but the golden cascade is WIDE (~60+ byte-identical goldens across 5 derive-identical probes + the CLI tests + any
  stray `#wat.kernel/<ErrorVariant>` assertion). The cascade is mechanical (pure prefix swap) but must be exact.
- **Trap-doors:**
  - A byte-identical golden where the `#wat.kernel/` appears in TWO places (outer tag + a nested error cause) — the outer
    flips to the phase ns, a nested cause flips to ITS phase, a nested shared block STAYS kernel. Each occurrence judged,
    not blind-replaced. (e.g. `#wat.macro/ProgramBodyEvalFailed {:cause #wat.runtime/… }`.)
  - The derive's path attribute resolving: `crate::error_ns::CHECK` in generated code must resolve in the `wat` crate. If
    the derive bakes a wrong path, the whole crate fails to compile — a loud, immediate signal.
  - A non-error type tagged `#wat.kernel/` that a blind sed would wrongly flip (STOP-2).
- **The weigh:** the orchestrator re-runs rows 4/5/6/8 itself, `git diff`s every touched golden to confirm each change is
  a pure prefix swap (nothing else moved), and greps row 6 by hand. A bent/softened golden = auto-reject (PROBATIO FLEXA
  MENTITVR — the widest cascade is where a weakening hides; read the iron in the dark).
