# Stone S-C.2d — mint `:wat::Record/same-data?` (type-BLIND record data equality)

**Status:** sub-DESIGN (composition proven). Parent: `DESIGN-RECORDS-AS-FIRST-CLASS-TYPES.md`.
Depends on: S-C.2c ✓ (base variant + `record->map` or-patterned for both flavors), arc 238 ✓
(`=` compares maps). Prior stone shapes to mirror: `eval_record_assoc` / `eval_record_to_map`.

## What this is

`:wat::Record/same-data?` — the user's **type-BLIND** record comparison. The clean split (locked):
- **`=`** (type-strict, arc 238): same type + identical values. `Pt[0,0] = Coord[0,0]` → false.
- **`same-data?`** (type-blind, this stone): same field DATA, ignoring class AND flavor.
  `(same-data? Pt[0,0] Coord[0,0])` → **true**. The 2×2 flavor grid + cross-type.

## Semantics (name-keyed; proven viable)

`same-data?` is true iff the two records have the **same field-name→value map**. It is:
- **name-keyed** — compares `{x:0, y:0}` maps, NOT positional tuples. `Pt[x,y]` vs `Foo[a,b]`
  (different field names) → false; `Pt[x:0,y:0]` vs `Coord[x:0,y:0]` (same names+values) → true.
  This *narrows* the units-footgun: `Meters[m:5.0]` vs `Feet[f:5.0]` → false (names differ).
- **type-blind** — `record->map` drops `class_fqdn`; only the field data is compared.
- **flavor-blind** — `record->map` works on base + holonic (or-patterned at S-C.2c).

## The impl is the proven composition

FM-2-bis (`tests/probe_arc237_sC2d_same_data.rs`, `comp_*` group, 3/3 GREEN NOW): the wat
expression `(:wat::core::= (:wat::core::record->map a) (:wat::core::record->map b))` already gives
exactly this — name-keyed (record->map), type-blind (drops class), and `=` on maps is
order-independent + structural (arc 238). `same-data?` is that composition, **named** (so users
reach for it, not re-derive the record->map+= trick — the user's explicit ask).

**Home:** substrate primitive `:wat::Record/same-data?` (the `:wat::Record/` namespace is
substrate-registered — `field-at`/`assoc` are eval fns; `same-data?` joins the family, not a
wat-defn into a reserved namespace). Impl = the composition in Rust:

```rust
// dispatch (next to assoc, ~runtime.rs:5345):
":wat::Record/same-data?" => eval_record_same_data(args, list_span, env, sym),

// eval fn — mirrors eval_record_assoc's arg-parse, then the proven composition:
fn eval_record_same_data(args, list_span, env, sym) -> Result<Value, RuntimeError> {
    // arity 2; eval both args to records.
    // extract each record's field map (the eval_record_to_map core — refactor its body into a
    //   reusable `record_field_map(&Value, sym, span) -> Result<Value /*HashMap*/, _>` and call twice),
    // then: Ok(Value::bool(values_equal(&map_a, &map_b) == Some(true)))
    //   (values_equal on two HashMap values is total → always Some after arc 238).
}
```

Non-record args → `record_field_map` errors (TypeMismatch), surfaced honestly. The checker scheme
`[a <- :wat::Record  b <- :wat::Record] -> :wat::core::bool` (mirror assoc's registration) bars
non-records at check time.

## Probe (FM-2-bis, on disk)

`tests/probe_arc237_sC2d_same_data.rs`:
- `comp_*` (3) — the composition; GREEN NOW (grounds the design; keep as regression).
- `samedata_*` (3) — the verb: same-type equal → true; cross-type same-data → true (type-blind);
  diff value → false. RED NOW (verb absent) → GREEN after the stone. Load-bearing.

(Holonic records only — base unconstructable until S-C.3. The impl handles base via `record->map`'s
or-pattern; a base-flavor same-data? case lands as a wat-surface test at S-C.3.)

## Scorecard
- [ ] `:wat::Record/same-data?` minted (dispatch + eval fn + checker scheme).
- [ ] `record_field_map` helper factored from `eval_record_to_map` (or eval_record_to_map reused);
      eval_record_to_map behavior unchanged (its probe stays green).
- [ ] `probe_arc237_sC2d_same_data` 6/6 (comp_* stay green; samedata_* flip RED→GREEN).
- [ ] lib baseline preserved (834/0; +0 unless a co-located unit test is added).
- [ ] no clippy beyond standing.

## Trap-doors (REJECTION STOPs)
1. Do NOT make `same-data?` type-AWARE — it is type-BLIND (that's the whole point; `=` is the
   type-strict one). Cross-type same-named-fields → true.
2. Do NOT compare positionally — name-keyed (via `record->map`). Different field names → false.
3. Do NOT re-implement map equality — reuse `values_equal` (arc 238 made it total for maps).
4. Do NOT break `eval_record_to_map` if you refactor its core (its probe must stay green).
5. Do NOT touch holon-rs / values_equal's existing arms.
6. Non-obvious error → STOP + surface verbatim.

## Calibration
One substrate primitive mirroring assoc + a small helper refactor. **Target band: 25–45 min Mode A.**
