# BRIEF — Stone S-C.2c — mint base `Value::wat__Record { class_fqdn, struct_form }`

**Status:** READY TO SPAWN. `model: "sonnet"`.
**Anchor cwd:** `/home/watmin/work/holon/wat-rs/` (`pwd` first; reject any `.claude/worktrees/` path; `git -C` if needed).
**Sub-DESIGN:** `DESIGN-STONE-S-C2c.md` — read it first; the "cascade — classified" + "FM 2-bis
probe" sections ARE the design. Parent model: `DESIGN-RECORDS-AS-FIRST-CLASS-TYPES.md`
§§ CORRECTION 1 + 2 (CORRECTION 2 governs; the "structural HolonRep" phrase in CORRECTION 1 is
superseded — base has NO holon projection). Prior stone to mirror: `SCORE-STONE-S-C2ab.md`.

## What to do (one additive change — nothing constructs base yet)

Mint the **base** record variant beside the existing holonic one. Base holds the struct flavor
ONLY (no `holon_form`); structural identity over `(class_fqdn, struct_form)`; holon-ops are a
teaching error. It is **additive** — there is no wat-surface constructor for base at this stone
(that is S-C.3). The only place a base value exists is the Rust probe (built directly).

There is an UNCOMMITTED, currently-non-compiling probe on disk:
`tests/probe_arc237_sC2c_base_record.rs`. **It is your spec.** It references `Value::wat__Record`
— making it compile + pass 6/6 is the goal. Treat its failure to compile as the
substrate-as-teacher compiler-output naming exactly what to build.

1. **Mint the variant** (`src/runtime.rs:651`, right after `wat__holon__Record`):
   ```rust
   wat__Record {
       class_fqdn: Arc<String>,
       struct_form: Arc<Vec<Value>>,
   }
   ```
   It must compile under the `#[wat_value]` seal — it is NON-wrapping (two `Arc` fields of
   non-`Self` types, same shape-class as `wat__holon__Record`, which already passes the seal).

2. **Bucket A — base-structural arms (NEW, distinct from holonic):**
   - **PartialEq** (`runtime.rs:898`): add a `wat__Record` vs `wat__Record` arm →
     `class_fqdn == class_fqdn && struct_form == struct_form`. Cross pairs (base vs holonic)
     fall to the existing `_ => false` — leave that alone (different flavors are different values).
   - **Hash** (`runtime.rs:1115`): add a `wat__Record` arm → tag `"wat__Record"` then hash
     `class_fqdn` + `struct_form`. Distinct tag from `"wat__holon__Record"` keeps Eq/Hash
     consistent.
   - **assoc** (`eval_record_assoc`, `runtime.rs:16719`/`16852`): add a base arm that rebuilds
     `struct_form` ONLY and returns `Value::wat__Record { .. }`. **Do NOT touch the holonic arm
     — it must still rebuild BOTH forms (parity invariant).**

3. **Bucket B — or-pattern (shared fields; identical behavior):** wherever a `match` reads only
   `class_fqdn` and/or `struct_form` (NOT `holon_form`), or-pattern the base variant in beside
   holonic. Known sites: `type_name` (`1219`→`"wat::Record"`), `declared_type_name`
   (`1311`→`class_fqdn`), `field-at` (`16543`, positional), keyword-accessor field dispatch
   (`6381`, rides `RecordDef.field_names`+`struct_form` after S-C.2ab — variant-agnostic),
   `record->map` (`16643`), `record?` predicate (`16605`→both ARE records),
   struct-destructure walk (`7753`), `":wat::Record"` path (`7507`), conforms class_fqdn check
   (`16157`). The compiler names the rest.

4. **Bucket C — holon-op teaching error (base has no holon flavor):** in `to_holon_inner`
   (`runtime.rs:17425`) and any holon-extraction/coerce site, add a base arm returning a
   `RuntimeError` (match the arc-233 rich shape) with message:
   *"base record `<class>` has no holon flavor; construct a holonic record
   (`:wat::holon::Record::def`) to use holon operations"*. **NOT a panic, NOT `Ok`.** Add a
   co-located `#[cfg(test)]` unit test in `runtime.rs` proving `to_holon_inner(base, &span)` is
   `Err(..)` with that message (the fn is private — the unit test is its only reachable home).

5. **Bucket D — leave the constructor alone** (`runtime.rs:16511`, `wat-record/of`): stays
   holonic-only. Base has no constructor this stone.

## Method — let the compiler drive (substrate-as-teacher)

Add the variant → `cargo build --release -p wat` → the compiler names every non-exhaustive
`match`. Each one is exactly one of Bucket A/B/C. Classify + fix; re-build; repeat to zero
errors. This is the expected, normal cascade — NOT a crisis.

## Discipline + the error-pivot law

- `src/runtime.rs` ONLY (plus the already-on-disk probe). No holon-rs (STOP-5). No base
  constructor / no macro split (that is S-C.3). No on-demand holon projection (base errors).
- **If you hit an error whose message does not make the fix obvious, STOP and surface it
  verbatim — do NOT guess** (`feedback_nonintuitive_error_is_pivot`). A confusing error is a
  substrate defect we pivot on, not an obstacle to code around.

## STOP triggers (REJECTION — not permission-to-defer)

1. You add `holon_form: Option` or any flag instead of a second variant.
2. You give base an on-demand holon projection (base holon-ops MUST error).
3. You add a base constructor or split the macros (that is S-C.3).
4. You touch the holonic parity rebuild in `assoc` (it must still rebuild both forms).
5. You collapse base into `Value::Struct`.
6. You touch holon-rs.
7. A non-obvious error (→ pivot, surface verbatim).
8. Lib baseline drops below **827/0** (additive stone — nothing constructs base, so every
   existing test must be untouched).
9. 60 min (STOP-3); 90 (STOP-4).

## Regression suite

```
cargo build --release -p wat                                       # 0 errors
cargo test --release --lib -p wat                                  # >= 827, 0 failed (+ your co-located to_holon base→Err unit test)
cargo test --release --test probe_arc237_sC2c_base_record          # 6/6 (the new probe)
cargo test --release --test probe_arc237_sC2ab_field_order         # 5/5 (S-C.2ab guard, untouched)
cargo test --release --test probe_arc237_sA1_assignable            # 6/6
cargo test --release --test probe_arc234_stone3c_keyword_accessor  # 6/6 (holonic field access unchanged)
cargo test --release --test probe_arc234_stone3b_record_assoc      # 6/6 (holonic assoc parity unchanged)
cargo test --release --test probe_arc227_stone2_defrecord          # 35/35 (defrecord surface unchanged)
```

## SCORE doc

`SCORE-STONE-S-C2c.md` (NEW). Mirror `SCORE-STONE-S-C2ab.md`: scorecard table + the variant +
the Bucket A/B/C arms (list each match site touched) + the cascade rounds (honest count) +
honest deltas + `git status --short`. **DO NOT commit** (orchestrator commits).

## Calibration

Additive variant + structural Eq/Hash + assoc base arm + or-pattern cascade + holon-op error
arms + 1 co-located unit test. Smaller decision surface than S-C.2ab (no macro/parse changes;
no parity re-route). **Target band: 30–55 min Mode A; 75 STOP-3; 90 STOP-4.** Prior calibration:
S-C.2ab landed in one pass, 0 cascade rounds, all green first run (`SCORE-STONE-S-C2ab.md`).
