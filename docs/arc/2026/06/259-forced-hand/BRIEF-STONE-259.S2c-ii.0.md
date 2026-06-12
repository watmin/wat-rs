# BRIEF — Stone 259.S2c-ii.0: defclause dispatch on a record's specific class (`class_fqdn`)

**The work, in one paragraph.** Teach the defclause runtime dispatch to match a record
value by its **specific class (`class_fqdn`)** instead of the generic variant tag. Today
`value_matches_type_pattern` (`src/runtime.rs:4905`) exact-matches `v.type_name()`, which
for a `Record::def` value returns the generic `"wat::Record"` — so a clause keyed on a
specific record type (`:user::Tag`, `:wat::spawn::ThreadOpts`) never matches
(`NoMatchingClause`). Every record value already carries its specific class as `class_fqdn`
(`Value::wat__Record { class_fqdn, .. }` / `Value::wat__holon__Record { class_fqdn, .. }`,
`src/value/value.rs:321/337`) — the dispatch just isn't reading it. The committed probe
`s2cii0_defclause_dispatches_on_record_class` flips RED→GREEN. This unblocks the host-type
`spawn-program'` defclause (S2c-ii).

**The contract:** in `value_matches_type_pattern`'s `TypeExpr::Path(p)` arm (the concrete,
non-type-var branch — `stripped == value_tag`), when the value `v` is a record
(`Value::wat__Record { class_fqdn, .. }` or `Value::wat__holon__Record { class_fqdn, .. }`),
compare the pattern's `stripped` against the record's `class_fqdn` (both are FQDN WITHOUT a
leading colon — `value.rs:313`). Use an **exact** `stripped == class_fqdn` match. Leave the
existing behavior for all non-record values (scalars, `Struct`, `Enum`, fn, collections)
untouched.

**Read in order (the rooms):**
1. `tests/nursery/probe_arc259_s2cii0_record_dispatch.rs` — the GREEN target (RED at HEAD:
   `expected :user::Tag, got :wat::Record`). Make it pass.
2. `src/runtime.rs:4905` — `value_matches_type_pattern`, the `TypeExpr::Path(p)` arm. The
   type-var early-return (bare-Uppercase → matches anything) STAYS. In the concrete-path
   branch, BEFORE/INSTEAD of `stripped == v.type_name()`, add: if `v` is a record variant,
   compare `stripped == class_fqdn`.
3. `src/value/value.rs:319-339` — the `wat__Record` / `wat__holon__Record` variants carry
   `class_fqdn: Arc<String>`. Match them to extract it (or use an existing accessor if one
   exists — grep `class_fqdn` in `src/value/value.rs` for a getter first).

**Implementation sketch:**
```rust
// inside the TypeExpr::Path(p) arm, concrete-path branch (after the type-var early return):
let value_tag = v.type_name();
// Arc 259 S2c-ii.0 — a Record::def value's type_name() is the generic variant tag
// "wat::Record"; its SPECIFIC class lives in class_fqdn. Dispatch on the specific class.
match v {
    Value::wat__Record { class_fqdn, .. } | Value::wat__holon__Record { class_fqdn, .. } => {
        stripped == class_fqdn.as_str()
    }
    _ => stripped == value_tag,
}
```

**Blast radius:** `src/runtime.rs` — the single `value_matches_type_pattern` function. No
other files; no check-side change (the checker already does the specific-class thing via
`is_subtype`); no parser/wat changes.

**STOP triggers (halt + report; do not work around):**
- **STOP-1:** if the fix would change matching for NON-record values (scalars, `Struct`,
  `Enum`, fn, collections), STOP — only the record path changes.
- **STOP-2:** if a record value's `class_fqdn` is NOT reachable in this function (no field /
  no accessor), STOP and report — do not reach into a constructor or change the value
  representation.

**Done = green:**
- `cargo test --release -p wat --test nursery probe_arc259_s2cii0` → passes (7).
- `cargo build --release` clean.
- `cargo test --release -p wat --test nursery -- --test-threads=1` → only the 4 known
  pre-existing reds (arc-255 reflection ×2 + undefined-builtin ×2). In particular the
  existing defclause tests (the `wat/core.wat` `+`/`-`/`*` arithmetic clauses, arc-256
  generic defclause, arc-237 dispatch) must stay green — record-dispatch is additive.

**Note for the future (rune-worthy, S3):** this uses an EXACT `class_fqdn` match, correct
for the concrete host opts (`ThreadOpts`/`ProcessOpts`, no subtypes). When record-SUBTYPE
dispatch lands (S3: `bracket::Env <: program::Env`), generalize to `is_subtype(class_fqdn,
pattern, types)` to mirror the check side — note it with a `rune:exigere(scope-affirmative)`.
