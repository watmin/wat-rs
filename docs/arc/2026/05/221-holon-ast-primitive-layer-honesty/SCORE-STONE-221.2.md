# SCORE — Arc 221 Stone 221.2 — wat-rs `value_to_atom` Char arm + `is_atomizable` Char extension

**Result: 5/5 PASS**

## Scorecard

| # | Row | Expectation | Result |
|---|---|---|---|
| 1 | `value_to_atom` Char arm | `src/runtime.rs` — `Value::wat__core__Char(c) => HolonAST::char_(c)` placed after the keyword arm in the primitive cluster; doc comment cites Stone 221.1 holon-rs commit `243eded`. | PASS |
| 2 | `is_atomizable` Char extension | `src/check.rs` — `\| ":wat::core::Char"` added to the matches!-arm; doc comment cites Stone 221.2 value_to_atom dispatch + Stone 220.2 runtime Hash arm. Additionally: `Char/of` TypeScheme registered (`String → Char`) — this was a latent gap from Stone 220.2 that blocked the `HashSet<Char>` dispatch at check time; surfaced and closed here. | PASS |
| 3 | 3 new probes | `tests/wat_arc221_char_atomization.rs` — Probe 1: `Atom(\a)` = `Atom(\a)`, `Atom(\a) ≠ Atom(\b)`, `Atom(\a) ≠ Atom(97)` (Char leaf distinct from i64 leaf). Probe 2: `HashMap<Char,i64>` assoc+get via `\a`/`\b` keys (char-frequency-tally). Probe 3: `HashSet<Char>` with vowels + `contains?` (hit `\a`/`\e`, miss `\z`). | PASS |
| 4 | Uuid arm NOT added (out of scope) | `git diff src/runtime.rs` shows zero changes to `Value::wat__core__Uuid` arm. Uuid stays in false-flag state; Stone 221.4 closes via Tag-based encoding after Stone 221.3 mints HolonAST::Tag. | PASS |
| 5 | All test suites + clippy green | `cargo build --release` 5 warnings (pre-existing; below 115 wat-clippy backlog). `cargo test --release --lib -p wat` 827/0 PASS. `cargo test --release --test wat_arc220_char` 10/10 PASS. `cargo test --release --test wat_arc221_char_atomization` 3/3 PASS. `cargo test --release --test wat_arc220_list` 23/23 PASS. `cargo test --release -p wat-edn` 1/1 PASS. `cargo clippy --release --all-targets -p wat-edn -- -D warnings` 0 warnings. | PASS |

## Honesty deltas vs. EXPECTATIONS

**Planned and executed:**
- 2 file edits (`src/runtime.rs` Char arm, `src/check.rs` Char extension)
- 1 new test file (`tests/wat_arc221_char_atomization.rs`, 3 probes)

**Surfaced gaps closed (not planned but required for correctness):**

1. **`Char/of` TypeScheme gap (Stone 220.2 latent honesty gap):** `Char/of` was registered at the runtime layer (Stone 220.2) but NOT at the check layer. The type checker returned `<unresolved>` for char literals (`\c` desugars to `(:wat::core::Char/of "c")`), which masked probe 3's `HashSet<Char>` dispatch TypeMismatch at check time. Fixed by registering `":wat::core::Char/of" :: String → Char` in `check.rs` alongside the Uuid registrations (line ~13497).

2. **Exhaustive-match cascade in 4 wat-rs files (Stone 221.1 ripple, compile error):** `HolonAST::Char` was added to holon-rs in Stone 221.1 (`243eded`), but 4 exhaustive-match sites in wat-rs were not updated. The build failed with `E0004`. Fixed with minimal, correct arms:
   - `src/hologram.rs`: `HolonAST::Char(_) => None` in `find_first_thermometer` (leaf, no thermometer)
   - `src/edn_shim.rs`: `HolonAST::Char(c) => OwnedValue::Tagged(Tag::ns("wat-edn.holon", "Char"), Box::new(OwnedValue::Char(*c)))` in `holon_ast_to_edn` + reader arm `("Char", OwnedValue::Char(c)) => Arc::new(HolonAST::Char(*c))` in `edn_holon_tag_to_ast`
   - `src/runtime.rs` `holon_to_watast`: `HolonAST::Char(c) => WatAST::List([Keyword(":wat::core::Char/of"), StringLit(c.to_string())], ...)` (round-trip safe via registered `Char/of` constructor)
   - `src/runtime.rs` `statement-length`: `HolonAST::Char(_) => 1` (leaf = 1 statement unit)

3. **`tests/wat_arc220_char.rs` 3 test updates (correctness companion to `Char/of` registration):** Tests 6, 7, 8 used `run_expecting_runtime_err` with programs that placed `Char/of <invalid>` directly as the `user::main` body (returning `nil` declared). Once `Char/of` was registered at check time, the checker caught `ReturnTypeMismatch` (body infers `Char`, declared `nil`). Fixed by wrapping the invalid `Char/of` call in a `let` binding with `nil` body — the runtime evaluation of the binding still fires the validation error while preserving the arc 170 canonical `user::main -> nil` signature.

## Out-of-scope confirmations (no violations)

- Uuid arm: not added. STOP-5 clean.
- holon-rs files: not touched. STOP-4 clean.
- Convention-based Char encoding: not used. HolonAST::char_() leaf used directly.
- Interop handshakes: not required; wat-edn surface untouched.
- INSCRIPTION: deferred to Stone 221.6.

## Calibration record

- **Target runtime:** 20-30 min Mode A
- **Upper bound:** 45 min
- **Actual runtime:** ~35 min (within upper bound, slightly over target band)
- **Reason for band-miss:** Two unplanned surface areas — `Char/of` TypeScheme gap (required for probe 3 check dispatch) and the 4 exhaustive-match compile errors from Stone 221.1 ripple into wat-rs. Both were correct to fix (honesty + compile correctness); the band assumed only 2 file edits + 3 probes.
