# EXPECTATIONS — 294.h · written BEFORE the strike, so the result cannot move the goalposts

Baseline at draw time, HEAD `6b5c8232`, measured by the orchestrator:

```
floor (scripts/floor.sh)  4675 run / 4675 passed / 30 skipped   0 FAIL  0 TIMEOUT
clippy                    0
#[ignore] attributes      24   (identity-tested; the containment grep says 76, of which 52 are prose)
```

## The scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | the trait is gone | `grep -rn "HolonRepresentable" src/ crates/` | **0** |
| 2 | comms holds no holon method calls | `grep -rn "to_holon_ast\|from_holon_ast" src/comms/` | **0** |
| 3 | the VSA round-trip survives | `grep -n "write_holon_ast_tagged\|read_holon_ast_tagged" src/lib.rs src/edn_shim.rs` | still exported at `lib.rs:138`, still defined at `edn_shim.rs:4265`/`:4274` |
| 4 | the VSA Bind/Bundle path is untouched | `git diff --stat src/runtime.rs src/edn_shim.rs` | **no change** |
| 5 | `String` + `Value` impls untouched | `git diff src/comms/mod.rs \| grep -c "^-.*self.clone()\|^-.*value_to_edn_string_with"` | **0** |
| 6 | the stale prose is corrected | `grep -n "to_holon_ast\|from_holon_ast" src/comms/process.rs` | **0** |
| 7 | floor green | `scripts/floor.sh` → read the **Summary line** | `0 FAIL, 0 TIMEOUT` |
| 8 | clippy clean | `cargo clippy --release --all-targets` | **0** |
| 9 | **no wat probe was taken** | the rider's per-file KEPT list | every `call_beside_value` / `startup_from_file` probe present and passing |
| 10 | the arithmetic is accounted for | floor Summary vs baseline | run-count drops by exactly the removed probes, each named |

**Row 9 is the load-bearing row.** Rows 1–2 can be satisfied by `rm` on five files, which would pass
every other row and silently destroy the arc-216 wat-side coverage. If row 9 is not evidenced by an
explicit KEPT list, the strike is not scored regardless of a green floor.

**Row 4 is the confusion trap.** `coerce_to_holon_ast` and `edn_to_holon_ast*` contain the string
`holon_ast` and are **not** the trait. A diff touching them means the rider deleted the VSA path.

## Independent prediction

**Runtime: 25–40 min.** It is a deletion with a mechanical classification rule, and the only thinking
is the per-probe body classification across five files. The build is the long pole — a
`src/comms/mod.rs` change rebuilds the crate, and `scripts/floor.sh` alone is ~200s.

**Run-count delta, predicted:** −14 to −18 tests.
`stone1` −1 · `stone2` −2 · `stone3` −1 to −3 · `stone7` −4 · `stone6` −9 (whole file) = −17 to −19,
minus whatever `tests/comms/foundation.rs` keeps after `ToyType` is re-pointed (it should keep its
tests, not lose them). Skips should stay at **30** — this stone touches no `#[ignore]`.

## Trap-doors named in advance

1. **`tests/comms/foundation.rs` loses tests instead of re-pointing them.** `ToyType`'s round-trip and
   error-honesty probes are about `WireError` behaving honestly, not about HolonAST. They must survive
   as `EdnRepresentable` tests. Deleting them is the cheap read of the instruction and it is wrong.
2. **The files' own `//! The N probes` headers undercount the removals.** Measured: `stone2`'s header
   names probe 11 only; probe 12 also calls `from_holon_ast`. A rider trusting the header leaves a
   compile error, which is loud — but a rider trusting the header *and* deleting extra to make it
   compile is quiet. Watch for over-removal in the KEPT list.
3. **`i64` is not a container and not a counterexample.** `comms::thread::pair<T: Send + 'static>`
   needs no `EdnRepresentable` at all. If the rider "fixes" `pair::<i64>()` sites at
   `kernel/peer.rs:647-648`, it has misread the tier.
4. **A green floor with a shrunken suite reads identical to a green floor.** Row 10 exists so the
   shrink is stated, not discovered later.

## What would make this a Mode B

Any of: row 9 unevidenced; a `src/` file outside `comms/` in the diff; the run-count delta unexplained;
a STOP trigger hit and worked around instead of reported.
