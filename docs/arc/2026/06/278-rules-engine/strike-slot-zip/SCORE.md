# SCORE — A3: the slot zip

`from_parts` takes the zip. Two independent arrays cannot be handed in. Floor GREEN.

LATENT: the safe writer was already safe by construction; the wire writer already hand-checked. I did not manufacture a before-red. The pairing is now unrepresentable.

## Scorecard

| # | result |
|---|---|
| 1 ★ one form | **HOLD.** `from_parts` takes `SlotZip`. Fields `slot_keys` / `output_slots` are gone. Accessors are derived iterators over the zip. |
| 2 ★ wire parse | **HOLD.** `unpack_compiled_cond` interleaves `items[3]` and `items[4]` into pairs; length mismatch fails there, same malformed string. |
| 3 ★ compile error | **HOLD.** `CompiledCond::from_parts(ops, Arc<[Value]>, Arc<[usize]>, …)` → `error[E0061]: this function takes 7 arguments but 8 arguments were supplied` / `argument #2 of type \`SlotZip\` is missing`. Captured at `gather_probe_cost.rs:976`, reverted. |
| 4 ★ guard converted | **HOLD.** `materialize_into`'s silent `i >= slot_keys.len()` is now `debug_assert!(i < compiled.zip.len(), "… output slot {slot} has no slot_key")`, sibling of the unbound-slot arm. Not deleted. |
| 5 malformed import | **HOLD.** `unpack_refuses_mismatched_slot_key_and_output_slot_lengths`: keys len 1, slots len 0 → refuses; message contains `slot_keys length`. |
| 6 wire ABI | **HOLD.** `pack_compiled_cond` still writes two sequences at indices 3 and 4. Round-trip green on the floor. |
| 7 floor | **HOLD.** `Summary [ 453.701s] 5439 tests run: 5439 passed (1 slow), 21 skipped`. `.floor/2026-09-05T23-09-20Z/`. |
| 8 clippy | **HOLD.** `cargo clippy --all-targets --release -- -D warnings` rc=0. |
| 9 cost | **HOLD.** `accum_cost` still on the floor; no `*_cost` assertion moved. STOP-2 did not fire. |

★ load-bearing. Row 7 is the deliverable.

## First floor was RED — captured, not re-run

`.floor/2026-09-05T22-57-32Z/` — `Summary [ 453.940s] 5439 tests run: 5438 passed (2 slow), 1 failed, 21 skipped`.

Arm: `wat::lint no_loose_string_assert::tests_carry_no_loose_string_assert` at `export.rs:2561` (`msg.contains`). Fix: `rune:lint(loose-assert)` on the contains line, same form as `probe_arc278_export.rs`'s classes-length wall. The red floor was not re-run.

## What the shape does not claim

`SlotZip::from_pairs(vec![])` alongside a longer op list still compiles. The zip couples *keys with slots*, not the zip with `ops`. An empty zip is a real form (class-only cond). A hand-built `from_parts` could pass Bind ops and an empty zip; that is not a length-mismatch of the pair, and this strike does not close it.

`intern_cond_keys` now sizes off `zip.len()`. No separate sizing change — it follows from the zip (CUT as named).

## Doors

| door | why it is safe |
|---|---|
| `SlotZip::from_pairs` | the only constructor; `pairs` is private |
| `SlotZip::key` / `slot` / `len` / `iter` | reads |
| `CompiledCond::slot_keys` / `output_slots` | derived iterators, not stored arrays; packer still emits two wire sequences |

No `#[cfg(test)]` hatch on the type. The malformed-import test drives `unpack_compiled_cond` directly.

## Still open (named, cut)

The other 12 `pack_`/`unpack_` pairs (`solvere` L2-3). D2p, F2, A4.
