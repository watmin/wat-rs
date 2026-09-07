# EXPECTATIONS — the match arm is a bracket clause with a map pattern

Written BEFORE the strike.

⚠ **ROW 2 IS THE STONE.** Every other row can be satisfied by an implementation that still binds
positionally and merely accepts a new delimiter. Row 2 cannot: the two readings of the same fixture
produce **different, checkable numbers**.

| # | what | command | expected |
|---|---|---|---|
| 1 | the bracket clause + map pattern is accepted | `cargo nextest run --release -E 'test(probe_arc109_match_arm)'` | `a_map_pattern_arm_binds_its_declared_keys` PASSES, prints `-1` |
| 2 | ⛔ **binding is BY NAME** | same | `a_map_pattern_binds_by_name_not_by_position` PASSES. Fixture declares `[a b]`, constructs `(Two 1 2)`, arm writes `{:b b :a a}`, body is `a - b`. **By name → `-1`. By position → `+1`.** No positional reading reaches `-1` |
| 3 | a unit arm is `{}` | same | `a_unit_variant_arm_is_an_empty_map` PASSES, prints `99` |
| 4 | ⛔ **ONE grammar, not two** | same | `the_retired_positional_clause_is_refused` PASSES. This row **passes at HEAD and inverts** — it is the only row whose red means "the old form still works" |
| 5 | the `#[ignore]`s are gone | `grep -c '#\[ignore' tests/wat_lang/probe_arc109_match_arm_is_a_map_pattern.rs` | `0` |
| 6 | the corpus moved BY CODEMOD | a committed `wat-scripts/fixes/*.wat` + a `/tmp` dry-run diff | exists; no hand-edited `.wat` arm (STOP-4) |
| 7 | `cond` is untouched | `git diff --stat` over cond's 37 files | zero changes (STOP-5) |
| 8 | the wildcard and binding arms still work | the floor | `[_ body]` and `[<bare-symbol> body]` unchanged — 2-element arms, head-discriminated |
| 9 | the floor | `./scripts/floor.sh` unpiped, Summary line | `0 failed` |
| 10 | clippy | `cargo clippy --release --all-targets --workspace` | 0 errors |

## RUNTIME PREDICTION

**2–3 h.** The Rust half (`eval_match` + `eval_match_tail` + the checker's arm grammar) is contained;
the corpus is 2333 forms across 653 files and the codemod's dry-run/diff discipline is the bulk.

## TRAP DOORS

- **The `let` destructure is RIGHT THERE and takes the OPPOSITE ORDER.** Arc 257.2's `{var :field}`
  is binder-first; a pattern is key-first. Reusing that reader silently inverts every binding, and
  the ruling's own first draft made this mistake by reaching for the `let` relative. STOP-1.
- **Row 2 is defeated by a fixture whose key order matches the declaration.** It is written reversed
  ON PURPOSE. If someone "tidies" `{:b b :a a}` into `{:a a :b b}`, the row silently stops testing
  anything — the same shape as an acceptance row a defect can satisfy.
- **Row 4 inverts.** Reading it as "red = not done yet" is backwards: red means the retired grammar
  is still accepted, which is the failure. It is the only row of the four that passes at HEAD.
- **A green floor mid-migration means little.** With 653 files, a partially-converted corpus can be
  green while both grammars are live — which is exactly what row 4 exists to catch.
- **`eval_match` never touches EDN.** This stone does NOT depend on H-2's wire; it depends on
  arc 296 G′ carrying `EnumValue.names`. Do not go looking for a wire dependency, and do not
  "fix" the arm by reaching for the EDN path.
