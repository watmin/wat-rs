# EXPECTATIONS — 296 H-3: Option and Result live in wat

Written BEFORE the strike.

⚠ **THE FAILURE MODE HERE IS SILENT AND THE FLOOR MAY NOT SEE IT.** A non-parametric Option still
compiles and most of the corpus still passes; what breaks is generic inference at sites no test
exercises. Row 1 is the guard, it is GREEN AT HEAD, and it is the only row that can see the defect.

| # | what | command | expected |
|---|---|---|---|
| 1 | ⛔ the params survive, in order | `cargo nextest run --release -E 'test(probe_arc296_h3)'` | PASSES. `Option` → `["T"]`, `Result` → `["T","E"]`. It passes at HEAD — a red means the move dropped them |
| 2 | the declaration is IN WAT | `grep -n 'defenum :wat::core::\(Option\|Result\)' wat/core.wat` | two hits |
| 3 | the Rust literals are GONE | `grep -n 'name: ":wat::core::Option"' src/types.rs` | 0 — replaced by `wat_enum_register_from!` |
| 4 | the binder capture works at arity 3 | a unit test over `ServiceEvent :- [I O A]` | `["I","O","A"]` in order (STOP-2 — arity 1 can pass while 3 fails) |
| 5 | the re-declaration is a NoOp | the stdlib loads | no `Duplicate`; the gate absorbs it (STOP-3) |
| 6 | the wire did not move | `cargo nextest run --release -E 'test(probe_arc296_h2)'` | 3/3, `#[ignore]` 0 |
| 7 | Option/Result round-trip | `cargo nextest run --release -E 'test(option_result_tagged)'` | green — `#wat.core/Option.Some {:value 7}` both directions |
| 8 | the floor | `./scripts/floor.sh` unpiped, Summary line | `0 failed` |
| 9 | clippy | `cargo clippy --release --all-targets --workspace` | 0 errors |
| 10 | purity unchanged | `git diff` | both stay `Pure` (STOP-4) |

## RUNTIME PREDICTION

**35–55 min.** The declaration is mechanical; the cost is the binder capture in the proc-macro and
proving it at arity 3.

## TRAP DOORS

- **A green floor is weak evidence here.** Row 1 exists because dropping the params is invisible to
  almost everything: the corpus mostly uses `Option` through `Some`/`None` constructors that do not
  re-check the parameter list.
- **The offset hazard is documented IN THE FILE BEING EDITED** (`lib.rs:48-49`) and it is silent by
  its own description. Row 4 is aimed at it, and one-param cases cannot see it.
- **`purity` is a DECISION with an argued rationale** at `types.rs:1233-1242` — it gates
  wire-crossing and `:durable`. It reads like a field to transcribe and is not.
- **Two register sites share the defect** — `wat_enum_register_from!` (`:727`) and
  `wat_record_from!` (`:497`). Fixing only the one this stone needs leaves the other primed for the
  next parametric record; if you fix only one, say so and why.
