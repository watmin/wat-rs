# BRIEF — the gate outcome outlives its file

**Read `DESIGN.md` beside this first.** Three assumptions are verified there with citations; do not
re-derive them, and do not assume a fourth.

## The work, in one paragraph

`GateOutcome` is declared in `wat/service.wat` (manifest position 35), so `wat/capability.wat`
(position 20) cannot name it — and therefore the `Capability` tier's `grant`/`revoke` still return `nil`
and raise through `require-granted`. Register the enum in `src/types.rs` instead, where every `.wat` can
see it regardless of order, and let the capability surface face the value. **Keep the name.**

## The rooms

| where | why |
|---|---|
| `src/types.rs:~1878` (`RecvOutcome`) | ⭑ the shape to copy — `name`/`type_params`/`purity`/`variants`, and `EnumVariant::Unit` vs `Tagged`. `Purity::Pure` is used by 20 enums there. |
| `wat/service.wat:3868` | the `defenum` to **delete**, verbatim, so the Rust registration matches it field-for-field |
| `wat/capability.wat:19–20` | `Capability`'s `grant`/`revoke` — `-> :wat::core::nil` becomes `-> :wat::service::GateOutcome` |
| `wat/capability.wat:45+` | `Dialable` / `TypedCapability` — the same two features under the typed surface |
| `wat/service.wat:3688` `grantable-extend` | stops wrapping `require-granted`; returns the outcome. `typedcap-extend` beside it is the twin — ⭑ **check it, it is the sibling** |
| `src/load/stdlib.rs:156` / `:341` | the manifest positions, if you want to see the ordering yourself |

## Implementation sketch

```rust
// src/types.rs, beside the other outcome enums
TypeDef::Enum(EnumDef {
    name: ":wat::service::GateOutcome".into(),
    type_params: vec![],
    purity: Purity::Pure,
    variants: vec![
        EnumVariant::Unit("Applied".into()),
        EnumVariant::Tagged { name: "Gone".into(),
            fields: vec![("cause".into(), TypeExpr::Path(":wat::kernel::LociDiedError".into()))] },
        EnumVariant::Tagged { name: "GaveUp".into(),
            fields: vec![("waited-ms".into(), TypeExpr::Path(":wat::core::i64".into())),
                         ("last".into(),      TypeExpr::Path(":wat::core::String".into()))] },
    ],
})
```

Then delete the `defenum`, widen the four feature signatures, and unwrap the two extend bodies.

## Blast radius

`src/types.rs` · `wat/service.wat` (one `defenum` deleted, two extend bodies) · `wat/capability.wat`
(four feature return types). **28 `GateOutcome` references stay exactly as written** — the name does not
change. No call-site migration.

## STOP triggers

1. **STOP-1 — if a SECOND file needs `CallOutcome` or `StopOutcome` moved too**, report it. The DESIGN
   rejects moving them as speculative *because nothing needs them today*; evidence that something does
   changes the answer, and I want that as a finding, not folded in silently.
2. **STOP-2 — if `capability.wat` depends on anything else declared after position 20**, stop and list it.
   Moving one type does not help if the file needs a second.
3. **STOP-3 — if the Rust registration cannot express something the `defenum` did** (purity marker,
   field names, a `:wat::enum::Pure` semantic), stop. Do not silently drop a property to make it fit.
4. **STOP-4 — if `require-granted` / `gate-faced` become unused**, do NOT delete them. They are the
   *caller's* choice now, which is the point; an unused helper here is deliberate, and say so in the SCORE.

## Verify

- `cargo build --release` **and** `cargo nextest run --release --no-run` — the stdlib is frozen into the
  binary at build time, so nothing `.wat` runs until the rebuild.
- `./scripts/floor.sh`, read the **Summary line**.
- `cargo clippy --release --workspace --all-targets -- -D warnings` → **exit 0**. It is at 0; adding to
  `src/types.rs` risks the `large_enum_variant` family. Verify, do not assume.
- Happy path `… 2000 4 3 8192 true 1000` → `distinct=8000;dup=0`; chaos `… 50 2 2 8192 true 1000 0 0 7 0 0 0 0 0 500` → exit 0.
- ⭑ **THE ROW THAT MATTERS — and a green floor will NOT give it to you.** There are **zero userland
  callers** of the capability-tier grant, so the floor may never execute the changed surface. Write a
  scratch probe that grants through `Capability`/`TypedCapability` and **prints the returned
  `GateOutcome`**. If you cannot show a value coming back, the change is unproven no matter how green
  everything is.

## Shape to copy

`the-gate-methods-face-an-outcome/` — the stone that created `GateOutcome` and hit this wall; its SCORE
§"CAPABILITY SURFACE STAYS nil" is the problem statement this stone answers.
