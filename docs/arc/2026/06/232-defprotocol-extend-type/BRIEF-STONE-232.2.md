# BRIEF — Stone 232.2: the satisfaction edge (extend-type → register_subtype)

Executor: Shadowdancer (sonnet). Anchor: `/home/watmin/work/holon/wat-rs/` (verify `pwd`; ONLY here;
ignore `.claude/worktrees/`). Run `cargo test`/`cargo build` PLAINLY (no setsid/timeout). Trust your
own build over rust-analyzer. **Do NOT commit — the Inquisitor weighs.**

## Work in one paragraph

Make `extend-type :T :P` register a subtype-parent edge `T → P` so a `:P`-typed parameter accepts
any `T` that extends `P`. This is the ONE change: an `:wat::core::extend-type` arm in
`splice_type_decls` (the TypeEnv pass) that calls `env.register_subtype(type_name, protocol_name,
span)` and keeps the form. `assignable`/`is_subtype` are UNCHANGED — they already consult the edge.
No `TypeDef::Protocol`, no method dispatch.

## Why it's this small (grounded)

- The RED probe fails with `TypeMismatch { expected: ":t::Greeter", got: ":t::Robot" }` — NOT
  "unknown type" — so the annotation already resolves; only the edge is missing.
- `register_subtype` (types.rs:446-449) explicitly ALLOWS edges from unregistered names ("the
  hierarchy is orthogonal to the TypeDef registry"). So `:P` needs no TypeDef.
- `assignable` (check.rs:13566) already returns true when `is_subtype(ap,ep)` holds; `is_subtype`
  (types.rs:3076) walks the multi-parent `subtype_edges` graph. The edge flows through, untouched.

## The change (rooms, read in order)

1. `src/types.rs:1489-1544` `splice_type_decls` — the `do` and `let` child loops. In each, BEFORE
   the `classify_type_decl` match (or in the `None` arm), detect a child whose head is
   `:wat::core::extend-type`: extract `(type_name, protocol_name)` and call
   `env.register_subtype(type_name, protocol_name, decl_span)`. **Push the child through to
   `new_children`** — do NOT strip it (unlike a TypeDef decl); 232.1's CheckEnv + runtime passes
   still need the form downstream. On `register_subtype` `Err` (cycle), propagate the `TypeError`.
2. `src/runtime.rs` `parse_extend_type_form` (pub(crate), from 232.1) — reuse it to get the two
   names (or read `items[1]`/`items[2]` directly if simpler; the form is
   `(:wat::core::extend-type :T :P (impl…)…)`).
3. `src/types.rs:450` `register_subtype` — the registrar (allows unregistered parents; cycle-checks).
   Precedent caller: types.rs:416 (recordtype's parent edge).
4. `src/check.rs:13566` `assignable` + `src/types.rs:3076` `is_subtype` — READ ONLY; confirm no
   change needed.

## Gate (run all; report each verbatim)

```
cargo test --release -p wat --test probe_arc232_2_protocol_assignable                # 2 passed
   (p_typed_param_accepts_an_extender: RED→GREEN; p_typed_param_rejects_a_non_extender: stays green)
cargo test --release -p wat --test probe_arc232_1_defprotocol_extend_register        # 1 passed (232.1 intact)
cargo test --release -p wat --lib -- --test-threads=1                                 # 916/36 (zero NEW)
cargo test --release -p wat --test nursery -- --test-threads=1                        # 895/4 (zero NEW)
cargo test --release --workspace --no-run                                             # compiles
```

## STOP triggers (REJECT — surface; do not improvise)

1. `register_subtype` returns `Err(CyclicSubtype)` for a legitimate extend → STOP (report; should
   never cycle for a protocol extension).
2. The probe won't go GREEN via the edge alone (i.e. you find you must change `assignable`/
   `is_subtype`) → STOP and surface why (the DESIGN says the edge suffices).
3. You're tempted to add `TypeDef::Protocol` or touch method dispatch (232.3) → STOP, out of scope.
4. The negative test (`p_typed_param_rejects_a_non_extender`) goes from green to RED — that means
   the edge over-reached (made non-extenders assignable) → STOP and fix the precision.

## Blast radius

`src/types.rs` (the `splice_type_decls` arm) primarily; a reuse of `parse_extend_type_form` from
`src/runtime.rs`. NO changes to `assignable`, `is_subtype`, the defclause dispatch path, or the
232.1 registries. The two probe files are already committed.

## Return

Report: the exact `splice_type_decls` edit (the arm + that the form is kept, not stripped), every
gate command's pass/fail counts from YOUR runs, and any honest delta. Do NOT commit.
