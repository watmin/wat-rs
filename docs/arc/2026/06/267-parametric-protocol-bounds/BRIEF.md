# BRIEF — Arc 267: parametric protocol bounds (one arm in `assignable`)

Executor: Shadowdancer (sonnet). Anchor: `/home/watmin/work/holon/wat-rs/` (verify `pwd`; ONLY here;
ignore `.claude/worktrees/`). Run `cargo` PLAINLY (no setsid/timeout). Trust your own clean build over
rust-analyzer (its mid-edit snapshots lie). **Do NOT commit — the Inquisitor weighs.** Full rationale:
`DESIGN.md` (this dir).

## Work in one paragraph
A `Parametric` type (`Box<i64>`, `Thread'<I,O>`) must satisfy a plain `Path` protocol bound (`:P`)
when its constructor `extend-type`s `:P`. This needs TWO arms — one at the check layer, one at the
runtime-dispatch layer. (The first strike applied part 1; the probe caught that part 2 was missing.
Part 1 may already be present in the working tree — ensure BOTH are present.)

## Room 1 — check (`src/check.rs:13673`, `fn assignable`)

Body:
```rust
let a = reduce(&walk(actual, subst), subst, types);
let e = reduce(&walk(expected, subst), subst, types);
if let (TypeExpr::Path(ap), TypeExpr::Path(ep)) = (&a, &e) {
    if ap != ep && crate::types::is_subtype(ap, ep, types) {
        return true;
    }
}
unify(actual, expected, subst, types).is_ok()
```
Immediately after the `(Path, Path)` block, before the `unify` fallthrough, ensure this arm is present:
```rust
// Arc 267 — a parametric type satisfies a plain protocol bound iff its CONSTRUCTOR
// extend-types the protocol. Edge keys carry the leading colon (types.rs:1402);
// Parametric.head does not — reconcile with `format!(":{head}")`.
if let (TypeExpr::Parametric { head, .. }, TypeExpr::Path(ep)) = (&a, &e) {
    if crate::types::is_subtype(&format!(":{head}"), ep, types) {
        return true;
    }
}
```

## Room 2 — runtime dispatch (`src/runtime.rs:4953`)

The protocol-method dispatch extracts the receiver's concrete-type FQDN with a match that today only
handles Record:
```rust
let concrete_type_fqdn: String = match &receiver {
    Value::wat__Record { class_fqdn, .. }
    | Value::wat__holon__Record { class_fqdn, .. } => {
        format!(":{}", class_fqdn)   // class_fqdn has NO leading colon
    }
    other_val => { /* MalformedForm "receiver must be a Record type…" */ }
};
```
Add two arms BEFORE the `other_val =>` fallback (these FQDNs are ALREADY colon-prefixed — use directly,
do NOT re-add a colon):
```rust
Value::Struct(sv) => sv.type_name.clone(),
Value::RustOpaque(inner) => inner.type_path.clone(),
```
Keep the `other_val =>` error arm as the genuine fallback. (Grounded: `StructValue.type_name` is
colon-form — runtime.rs:18753; `RustOpaque.type_path` is colon-form — `THREAD_PEER_TYPE_PATH =
":wat::kernel::Thread'"`; both match the `extend:<P>:<T>` key the Record path also targets.) No
over-acceptance: a Struct/opaque that doesn't extend `:P` still fails the `extend_key` lookup → the
existing clean "type `…` does not extend protocol `…`" error.

## Gate (run all; report verbatim from YOUR runs)
```
cargo test --release -p wat --test probe_arc267_parametric_extend_type                  # 1 passed (RED→GREEN: Box<i64> satisfies :t::Tagged)
cargo test --release -p wat --test probe_arc209_handle_protocol -- --test-threads=1     # 1 passed (end-to-end: Thread' satisfies :wat::kernel::Spawned)
cargo test --release -p wat --test probe_arc232_2_protocol_assignable                   # passes (non-parametric path UNBROKEN)
cargo test --release -p wat --test probe_arc232_3_protocol_dispatch                     # passes (dispatch UNBROKEN)
cargo test --release -p wat --lib -- --test-threads=1                                   # zero NEW vs baseline 917/36
cargo test --release -p wat --test nursery -- --test-threads=1                          # zero NEW vs baseline 895/4
cargo test --release --workspace --no-run                                               # compiles
```

## STOP triggers (REJECT — surface; do not improvise)
1. Either FQDN form doesn't match the registered key (a probe stays RED after both arms) → STOP;
   report the actual key string vs the produced FQDN (don't guess another form blindly).
2. Either arm breaks a 232 non-parametric case (probe_arc232_2/3 regress) → STOP.
3. The fix would require touching `unify`, `is_subtype`, `register_subtype`, the extend-key format,
   or the 232 forms → STOP (the change is the two arms only: one in `assignable`, one in the
   `runtime.rs:4953` receiver match).
4. Any lib/nursery test green at baseline goes red → STOP and report which.

## Blast radius
`src/check.rs` — `fn assignable` (one arm) AND `src/runtime.rs` — the `concrete_type_fqdn` match at
~4953 (two arms). NOTHING else: no `unify`/`is_subtype`/`register_subtype`/extend-key/232-forms
changes. The probes are already committed. (Part 1 may already be applied in the working tree from the
prior strike — ensure both arms are present and the gate is green.)

## Return
Report: the exact arm added (file:line); every gate command's counts from YOUR runs; confirmation the
232 non-parametric probes still pass; any honest delta. If a STOP fires, STOP and report. Do NOT commit.
