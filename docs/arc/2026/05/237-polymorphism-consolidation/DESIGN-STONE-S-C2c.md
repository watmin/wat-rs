# Stone S-C.2c — mint base `Value::wat__Record { class_fqdn, struct_form }`

**Status:** sub-DESIGN (pre-probe). Authoritative parent model:
`DESIGN-RECORDS-AS-FIRST-CLASS-TYPES.md` §§ **CORRECTION 1 + CORRECTION 2** (CORRECTION 2
refines/supersedes 1; the stray "structural HolonRep" phrase in CORRECTION 1 line 327 is
superseded — base has NO holon projection). Live order: `REMAINING-ORDER.md`. Prior stone
shape to mirror: `SCORE-STONE-S-C2ab.md` + `BRIEF-STONE-S-C2ab.md`.

---

## What this stone does (one sentence)

Mint the **base** record variant `Value::wat__Record { class_fqdn, struct_form }` beside the
existing **holonic** `Value::wat__holon__Record { class_fqdn, struct_form, holon_form }`, with
structural Eq/Hash/Display, variant-agnostic field access (riding S-C.2ab's `RecordDef.field_names`
path), and **holon-ops as a teaching error** (holonic-only). **Additive: nothing constructs base
yet** (the macro split that produces it is S-C.3).

## Why now (forced chain)

S-C.1 freed the `wat__Record` name (rename → `wat__holon__Record`). S-C.2ab moved field NAMES
onto `RecordDef.field_names` and re-routed name→index access off `holon_form` — making name-based
access **variant-agnostic**, the prerequisite for a base variant that has no `holon_form`. S-C.2c
mints base on the freed name using that path. S-C.3 (macro split) cannot dispatch to a base
variant that does not exist; S-D cannot migrate to macros that do not exist. Order is forced.

## The model (locked — do NOT rebuild rejected shapes)

- **TWO variants, not `holon_form: Option`.** Flavor is the variant, decoded by `match`
  (`feedback_no_semantic_abuse_of_option`).
- **base = struct only; holonic = struct + holon in permanent parity.** **NO on-demand
  projection** — holonic *stores* both; base *has only* the struct. A base record's holon flavor
  does not exist (not "lazily computed"); asking for it is an error.
- **Liskov:** `:wat::holon::Record <: :wat::Record`. A func wanting holonic rejects base; a func
  wanting base takes both. (Already wired by S-A1 `assignable`; nothing to add here.)
- **base is wat-local.** No holon projection ⇒ base does not cross a process boundary as a holon.
  If a value must cross / do holon-ops, it must be holonic. This is the whole point of the split:
  you pay for the hologram only when you ask for holonic.

## The variant

```rust
/// S-C.2c — base (wat) record: the reduced flavor. EDN-restricted data held in a
/// positional struct_form; NO holon_form. Field NAMES live on the class
/// (RecordDef.field_names, S-C.2ab); name→index access rides that path.
/// Structural identity over (class_fqdn, struct_form). Holon-ops are a teaching
/// error — base has no holon flavor (use a holonic record). Unconstructed until
/// S-C.3 mints :wat::Record::def → base.
wat__Record {
    class_fqdn: Arc<String>,
    struct_form: Arc<Vec<Value>>,
}
```

## The cascade — classified (verified line numbers @ HEAD acce22fe)

Three buckets. The compiler (substrate-as-teacher) names every non-exhaustive `match`; each one
is exactly one of these. The known load-bearing sites:

### Bucket A — base-structural (NEW arm, distinct from holonic)
- **PartialEq** (`runtime.rs:898`): add
  `(Value::wat__Record { class_fqdn: a, struct_form: sa }, Value::wat__Record { class_fqdn: b, struct_form: sb }) => a == b && sa == sb`.
  Base-vs-holonic cross pairs fall to the existing `_ => false` (different flavors are different
  values — on-doctrine; holonic identity is `holon_form`, base identity is `struct_form`).
- **Hash** (`runtime.rs:1115`): add
  `Value::wat__Record { class_fqdn, struct_form } => { "wat__Record".hash(state); class_fqdn.hash(state); struct_form.hash(state); }`.
  Distinct discriminant tag from `"wat__holon__Record"` keeps Eq/Hash consistent (cross-variant
  never collides).
- **assoc** (`runtime.rs:16719`): holonic rebuilds BOTH forms (parity). Base arm rebuilds
  `struct_form` ONLY and returns `Value::wat__Record { class_fqdn, struct_form: new }`. (Holonic
  arm unchanged — preserve the parity invariant at `16852`/holon-rebuild.)

### Bucket B — or-pattern (shared fields; identical behavior)
- **type_name** (`runtime.rs:1219`) → `":wat::Record"` for both.
- **declared_type_name** (`runtime.rs:1298`/`1311`) → `class_fqdn.to_string()` for both.
- **field-at positional accessor** (`runtime.rs:16543`): `struct_form` for both.
- **keyword-accessor / field dispatch** (`runtime.rs:6381`): rides `RecordDef.field_names` +
  `struct_form` (S-C.2ab) — variant-agnostic; or-pattern the two record arms.
- **record→map extraction** (`runtime.rs:16643`): name-pairs over `field_names` + `struct_form` —
  or-pattern (no `holon_form` dependency after S-C.2ab).
- **record? predicate** (`runtime.rs:16605`): `matches!(v, wat__holon__Record{..} | wat__Record{..})`
  — both ARE records.
- **struct-destructure walk** (`runtime.rs:7753`), **`val_type_path`/`":wat::Record"`** (`7507`),
  **conforms class_fqdn check** (`16157`) — or-pattern.
- Sites in `check.rs` / `types.rs` / `stdlib.rs` / `edn_shim.rs` / `closure_extract.rs`: expected
  reads — classify on cascade (or-pattern unless they touch identity/holon, which they should not).

### Bucket C — holon-op → teaching error (base has no holon flavor)
- **to-holon / coerce / holon-extraction** (`to_holon_inner` `runtime.rs:17425`; sites ~`17589`,
  `19016`, `18536`, `18605`): base arm returns a `RuntimeError` carrying:
  *"base record `<class>` has no holon flavor; construct a holonic record (`:wat::holon::Record::def`)
  to use holon operations"*. Match the existing rich-diagnostic shape (arc 233). This is the ONLY
  place base "fails" — and it fails by TEACHING, not panicking.

### Bucket D — unchanged (base unconstructed until S-C.3)
- **constructor** (`wat-record/of` / record builder `runtime.rs:16511`): stays holonic-only. Base
  has no wat-surface constructor at this stone — that is S-C.3 (the macro split). This is WHY the
  FM 2-bis probe is Rust-layer.

## FM 2-bis probe (Rust-layer; committed before BRIEF)

`tests/probe_arc237_sC2c_base_record.rs` — constructs `Value::wat__Record` directly (no wat
producer exists yet) and asserts the contract. Contracts:

1. **structural Eq — equal:** two base records, same `class_fqdn` + same `struct_form` ⇒ `==`.
2. **structural Eq — class differs:** same struct, different class ⇒ `!=`.
3. **structural Eq — struct differs:** same class, different struct ⇒ `!=`.
4. **base ≠ holonic:** a base record and a holonic record with the "same" logical data ⇒ `!=`
   (different flavors; guards the `_ => false` cross arm).
5. **Hash consistency:** `a == b ⇒ hash(a) == hash(b)` (HashSet insertion-dedup of two equal
   base records yields len 1).
6. **positional field-at:** `field_at(base, i)` returns `struct_form[i]`.
7. **holon-op errors:** `to_holon_inner(base, …)` is `Err(..)` with the teaching message
   (NOT a panic, NOT an `Ok`).

**Pre-stone state:** the probe does not compile (variant absent). That is the disconfirming
state — it is committed on a scratch branch-free basis as part of the stone flight, NOT to the
green baseline ahead of the build (a non-compiling test file would break `cargo test`
workspace-wide). **Adjustment vs the usual "commit RED probe first":** because the RED state here
is a *compile* failure (not a runtime failure), the probe is authored + committed **atomically
with** the substrate change (same flight, one commit), and its design is frozen in THIS doc as
the contract sonnet mirrors. (Same handling as 234.x Rust-layer substrate probes — see
`SCORE-STONE-*` precedent. The probe's VALUE is the frozen contract, captured here regardless of
commit timing.)

## Scorecard (for EXPECTATIONS)

- [ ] `Value::wat__Record` minted; compiles under `#[wat_value]` seal (non-wrapping; two `Arc`
      fields — same shape-class as holonic, which already passes the seal).
- [ ] Bucket A arms added (Eq, Hash, assoc); Bucket B or-patterns; Bucket C teaching errors.
- [ ] `probe_arc237_sC2c_base_record` 7/7 PASS.
- [ ] Lib baseline preserved: **827 pass / 0 fail** (additive stone; nothing constructs base, so
      every existing test is untouched).
- [ ] No new clippy beyond the standing ~54.

## Trap-doors (REJECTION STOPs for the BRIEF)

1. **Do NOT add `holon_form: Option`** or any flag. Two variants, decoded by `match`.
2. **Do NOT give base an on-demand holon projection** (no struct→holon at to-holon time). Base
   holon-ops ERROR. If the build seems to "want" a projection to stay total, that is the wrong
   instinct — the error arm IS the total answer.
3. **Do NOT construct base anywhere** (no wat producer; no `wat-record/of` base path). Base stays
   unconstructed until S-C.3. The probe is the only place a base value exists, built directly in
   Rust.
4. **Do NOT disturb the holonic parity invariant** (assoc still rebuilds both forms for holonic).
5. **Do NOT collapse base into `Value::Struct`** — a record is EDN-restricted; a struct admits
   any rust value. Different contracts; base is its own variant.
6. **STOP on any non-obvious compiler error** — surface it verbatim (`feedback_nonintuitive_error_is_pivot`).
   The cascade should be mechanical; a confusing error is a substrate defect to pivot on, not push through.
```
