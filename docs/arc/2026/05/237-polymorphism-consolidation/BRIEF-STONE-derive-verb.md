# BRIEF — Stone (237 follow-on): the `:wat::core::derive` verb

Executor: Shadowdancer (sonnet). Anchor: `/home/watmin/work/holon/wat-rs/` (verify `pwd`; ONLY here;
ignore `.claude/worktrees/`). Run `cargo` PLAINLY (no setsid/timeout). Trust your own clean build over
rust-analyzer (its mid-edit snapshots lie). **Do NOT commit — the Inquisitor weighs.** Full rationale:
`DESIGN-STONE-derive-verb.md` (this dir).

## Work in one paragraph
Ship `(:wat::core::derive :Child :Parent)` — a declaration form that registers a `typesub` edge
Child→Parent (Clojure's `derive`/`isa?` axis; a marker relationship, NO methods). It is **exactly
`extend-type`'s edge-registration half, minus the method-impls and minus the protocol requirement**.
Mirror the three `extend-type` sites: parse, edge-registration (pre-check), check-arm. Nothing else —
`is_subtype`/`subtype?`/`register_subtype`/the arc-267 `assignable` arm are all consumed as-is.

## Rooms (read `extend-type` as the template, then build `derive` beside it)

1. **`src/runtime.rs` `parse_extend_type_form` (~5790)** — the parse model. Build
   `parse_derive_form(form) -> Result<(String, String), RuntimeError>`: `items[1]` = `:Child` keyword,
   `items[2]` = `:Parent` keyword (both `k.clone()`); require exactly 3 items; NO method-impl loop.

2. **The edge-registration site.** `extend-type` registers its edge via
   `env.register_subtype(&type_name, &protocol_name, span)` (types.rs:1571, in `splice_type_decls`).
   Register the derive edge the SAME way at the SAME pre-check point so `assignable` sees it:
   `env.register_subtype(&child, &parent, span)`. Find where `extend-type` is dispatched in that
   pre-expansion/splice flow and add a sibling `:wat::core::derive` arm. (The cycle check in
   `register_subtype` already rejects a cycle-closing derive — surface it as the existing
   `TypeError::CyclicSubtype`, do not swallow.)

3. **The check arm.** `extend-type` / `declare-acronyms` / `defprotocol` each have an arm in
   `infer_list` (+ `collect_splice_defs_ctx`) that type-checks the declaration form to **unit**
   (`:wat::core::nil`) and is accepted at top level. Add the `:wat::core::derive` arm the same way:
   it returns unit, registers nothing new at check beyond what room 2 did.

There is **no runtime eval artifact** beyond the edge (like `declare-acronyms`): a `derive` form
evaluates to unit.

## Gate (run all; report verbatim from YOUR runs)
```
cargo test --release -p wat --test probe_arc237_derive_verb                       # 2 passed (RED→GREEN: A & B derive :t::Marker, accepted at the bound)
cargo test --release -p wat --test probe_arc237_sA_hierarchy -- --test-threads=1  # passes (typesub mechanism unbroken)
cargo test --release -p wat --test probe_arc232_2_protocol_assignable             # passes (extend-type bound path unbroken)
cargo test --release -p wat --test probe_arc232_1_defprotocol_extend_register     # passes (extend-type registration unbroken)
cargo test --release -p wat --lib -- --test-threads=1                             # zero NEW vs baseline 917/36
cargo test --release -p wat --test nursery -- --test-threads=1                    # zero NEW vs baseline 895/4
cargo test --release --workspace --no-run                                         # compiles
```

## STOP triggers (REJECT — surface; do not improvise)
1. `register_subtype` for the derive edge can't be reached at a pre-check point (the probe stays RED
   because `assignable` doesn't see the edge) → STOP; report where `extend-type`'s edge actually
   registers vs where you put derive's.
2. Building `derive` forces a change to `is_subtype`/`subtype?`/`register_subtype`/the 267 `assignable`
   arm/the `extend-type` forms → STOP (derive only ADDS a parse + a registration arm + a check arm).
3. The marker `:t::Marker` needs a separate type declaration to be a valid bound annotation → STOP
   and report (grounding says it does NOT — annotations resolve permissively; if that's wrong, the
   scope changed).
4. Any baseline-green lib/nursery test goes red → STOP and report which.

## Blast radius
`src/runtime.rs` (`parse_derive_form` + the registration arm) + `src/check.rs` (the `infer_list` /
`collect_splice_defs_ctx` derive arm). Model every piece on the existing `extend-type` handling. NO
other files. The probe is already committed.

## Return
Report: `parse_derive_form` (file:line); the edge-registration arm (file:line) + which `extend-type`
site you mirrored; the check arm (file:line); every gate command's counts from YOUR runs; confirm the
232 + 237-S-A probes still pass; any honest delta. If a STOP fires, STOP and report. Do NOT commit.
