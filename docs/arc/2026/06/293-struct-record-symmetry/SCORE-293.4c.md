# SCORE — 293.4c: `extend-type` as the foreign-accessor adapter

**Verdict: GREEN, weighed by the orchestrator's own re-run.** `cargo nextest run --release` = **4092 passed / 0 failed
/ 93 skipped**. `extend-type` now teaches a foreign type (a non-aggregate built-in) to satisfy + dispatch a surface; the
293.4c probe flipped RED→GREEN; both negatives (collision, non-extended) reject; the arc-232 protocol path is untouched.

## Scorecard (each row re-run by the orchestrator)
| # | what | result |
|---|---|---|
| 1 | 293.4c probe GREEN (un-ignored) | **PASS** — `(:t::probe)` → 42 |
| 2 | foreign type satisfies + dispatches via extend-type | **PASS** — `:wat::core::String/tag` adapter, dispatched on a String |
| 3 | collision = DuplicateDefine | **PASS** — `_dup.wat.bad` rejected |
| 4 | non-extended foreign type rejected | **PASS** — `_notextended.wat.bad` rejected |
| 5 | protocol extend-type un-regressed | **PASS** — `binary(function)` 221/221 |
| 6 | 293.4a + 293.4b un-regressed | **PASS** |
| 7 | acceptance demo stays RED | **PASS** — still `#[ignore]`'d |
| 8 | whole workspace green | **PASS** — 4092 / 0 / 93 (own forced run) |

## What shipped
- **`src/runtime.rs`** — (a) `parse_extend_type_form`: capture an optional `-> :ret` on impl clauses (protocol impls
  without it take the `:nil` branch unchanged — backward-compatible); (b) both extend-type registration arms
  (user + stdlib) branch on `TypeDef::Surface` → register each impl as a `:<T>/<method>` Function in `sym.functions`
  (collision = `DuplicateDefine`); protocol path is the unchanged `else`; (c) the 293.4b dispatcher derives the concrete
  FQDN from `format!(":{}", receiver.type_name())` (Record/Struct/RustOpaque preserved), so non-Record receivers dispatch.
- **`src/check.rs`** — (a) the register-extend pass branches on Surface → register each impl as a `TypeScheme` in
  `env.schemes` under the SAME `<type>/<method>` key; (b) `assignable` non-aggregate path: after the Aggregate gate
  fails, a foreign type satisfies a surface iff the surface has **no holder bound** AND **no field members** AND every
  method member resolves via `env.get(key)` (`struct_satisfies_surface(&[], …)`). Properly bounded — STOP-3 guarded.
- **Tests** — probe un-ignored + collision arm + negative arm; `_dup.wat.bad`, `_notextended.wat.bad` new.

## Honest deltas (carried, not hidden)
1. **Check + runtime agree by construction** — both populate/read the one canonical key `<T>/<method>`
   (`env.schemes` / `sym.functions`). Identical key → identical satisfaction outcome. Verified on disk.
2. **No STOP fired.** The probe's `:wat::core::String` has an unambiguous `type_name`, so STOP-2 (FQDN ≠ extend key)
   never bit. The dispatcher generalization is `type_name()`-based; a value whose `type_name` differs from the `:<T>`
   the user wrote (e.g. a holon Vector variant) would mis-map — relevant at 293.4d, not here.
3. **⚠ FLAG FOR 293.4d — the field/method accessor symmetry is NOT YET built.** A foreign type satisfies METHOD members
   only; a FIELD member is (correctly, for now) unsatisfiable by a foreign type with no struct fields. But the acceptance
   demo's `:geo::Shape` has `color` as a **field** member, and the holon-Vector monkeypatch backs `color` with a
   **method** impl — the DESIGN's "field-vs-method is the satisfier's private choice." **293.4d must let a field member
   be satisfied by a `:T/<name>` accessor (field OR method).** The demo probe will RED on exactly this.

## Next
**293.4d — the arc's GREEN gate.** (a) The field-member-by-accessor symmetry (the flag above); (b) annihilate
`defprotocol` (ONE live use `:wat::spawn::Locus`, `wat/spawn.wat:224`; rip the Rust machinery across 6 files;
retirement-table the head); (c) un-ignore `probe_arc293_acceptance_demo` = GREEN (R1 *FORMA SOLA SUFFICIT* fulfilled).
Then 293.1-owed `src/aggregate/` home + 293.5 close.
