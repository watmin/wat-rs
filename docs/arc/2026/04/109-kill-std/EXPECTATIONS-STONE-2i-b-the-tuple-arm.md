# EXPECTATIONS — arc 109 Stone ②-i-b: the Tuple arm

Written BEFORE the strike, against `984ce63fc`. The orchestrator scores against these rows after its
own independent re-run; the rider's report does not move them.

## The scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | the three faults are gone | `./target/release/wat ./wat-scripts/scratch-pad/arc109-tuple-arm-faults.wat` | rows below, exactly |
| 2 | the reader still takes what the writer now emits | `./target/release/wat --check ./wat-scripts/scratch-pad/arc109-tuple-bracket-reader.wat` | EXIT 0 |
| 3 | the verb's contract suite | `cargo nextest run --release -E 'test(contract_0)'` | 9 pass, 0 fail |
| 4 | the reflection goldens | `cargo nextest run --release -E 'test(structured_signature_types)'` | all pass |
| 5 | every wat-scripts file still loads | `cargo nextest run --release -E 'test(every_wat_scripts_file_loads)'` | pass |
| 6 | the floor | `scripts/floor.sh` | 4855+/4855+, **0 FAIL** |
| 7 | clippy | `cargo clippy --all-targets -- -D warnings` | 0 warnings |
| 8 | the codemod's guard stops firing | re-run ②-ii's dry-run on `/tmp` copies of `wat/sqlite.wat` + `wat/fix.wat` | no `wat.type/`-guard refusals; previously-skipped tuple sites now convert |

Row 1's expected output, predicted exactly:

```
1 nil bare       : :wat::core::nil
2 nil nested     : (:wat::core::Result [:wat::core::nil :wat::core::String])
3 tuple 3-ary    : (:wat::core::Tuple [:wat::core::i64 :wat::core::i64 :wat::core::String])
4 tuple 1-ary    : (:wat::core::Tuple [:wat::core::i64])
5 tuple empty    : (:wat::core::Tuple [])
6 control parm   : (:wat::core::Vector [:wat::core::i64])
```

Row 6 must be byte-identical to today — it is the control. If it moves, half (a) reached further
than `nil` and STOP-2 fired. Rows 3, 4 and 5 are the arity ladder: **no rung may emit a bare head,
and none may be special-cased in the code** — one path, `args.len()` never consulted.

## Independent prediction

**Runtime: 12–20 minutes.** One new function that is a copy of its neighbour with one argument
flipped, one match arm rewritten against a worked example two arms above it, one call site, one doc
paragraph, four one-line goldens, and one new test assertion with its fixture. Every room is exact
and the hard question — does the reader accept the output — was answered before the brief.

## Trap-doors, named in advance

1. **A fifth golden.** The four in the brief are the orchestrator's measurement:
   `(wat.type/Tuple` appears in `tests/` at contract-06, contract-07, contract-08, the c09 fixture's
   param type, and `wat_arc201_structured_signature_types__tuple.edn`. The c09 fixture is
   deliberately NOT in the list — it pins that the FLAT form still *reads*, which this stone does
   not change. If a fifth pin surfaces at floor time, the census was incomplete, not the rider.
2. **`wat.type/nil` as an emitted spelling.** After half (a), Clojure mode renders
   `Path(":wat::core::nil")` through ladder case 1 → `wat.type/nil`. It should read back
   (`wat.type/nil` → `:wat::type::nil` → canonicalize → `:wat::core::nil` → `Tuple(vec![])`), but
   that round-trip is asserted here, not yet measured — row 3's new `c07b` is what measures it, and
   if `wat.type/nil` does NOT read back, that is a real finding about the Clojure spelling of unit
   and it belongs to the builder, not to a patch in this stone.
3. **The reflection path moves too.** `signature-of-defn` renders a nil return as
   `(wat.type/Tuple)` today and `(wat.type/Tuple [])` after (b) — half (a) cannot reach it (it gets
   an already-canonicalized `TypeExpr`, with no source keyword left to preserve). Measured: no
   golden currently renders a nil return through that path. If one goes red at the floor, this is
   why, and the honest fix is the golden, not the arm.
4. **`reject_any` forgotten.** The new entry point is a copy of `parse_type_expr_with_span`; the
   easy miss is dropping the `reject_any` call, which would silently open an `:Any` hole on the
   renderer path. Row 6 would not necessarily catch it.

## What would make this a Mode B

Any of: the `Fn` arm touched · `src/types.rs:4728` edited · `parse_type_node` edited to make the
output read back (STOP-1 exists precisely so that surfaces as a finding) · a golden updated to match
observed output rather than to the predicted bytes above · cargo run by the rider.
