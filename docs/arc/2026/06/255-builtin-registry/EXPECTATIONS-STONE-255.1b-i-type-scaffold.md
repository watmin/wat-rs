# EXPECTATIONS — STONE 255.1b-i

Written **before** the strike.

## Scorecard

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | the enums exist and are enums | read the new module | `Arity`, `Purity`, `Determinism`, `ExpandTime`, `DefKind` — **no bools** anywhere in the baseline |
| 2 | **the forcing is real** | omit a baseline field, compile | **compile error**, and its **actual text reported** |
| 3 | no `Default` on the baseline | `grep -n "Default" <new module>` | zero derives/impls of `Default` on `Registration` |
| 4 | `MetaField<T>` is a named sum | read | `Unspecified \| Specified(T)`; **`Option<T>` appears nowhere** in the metadata layer |
| 5 | `DefDetail` delegates | read | `Type(TypeDef)` — **not** flattened variants; `StructDef`/`RecordDef`/`ProtocolDef` appear nowhere |
| 6 | **not dead code** | `cargo clippy --release --all-targets` | zero warnings **and no `#[allow(dead_code)]`** in the new module |
| 7 | a module is a directory | `git diff --stat` | new code in `src/<ns>/…`, not a bare `src/*.rs` |
| 8 | inert | `git diff` | no behaviour change — no registration, no `Function` split, no resolver/runtime edit |
| 9 | build | `cargo build --release` | exit 0 |
| 10 | **floor** | orchestrator's own `scripts/floor.sh` | zero new failures vs **4398/4398**; a changed count either way is a finding |

**Row 2 is the stone.** Everything else is scaffolding around it: if omitting a field still compiles,
the baseline is a suggestion and this slice bought nothing. **Row 6 is its shadow** — a `dead_code`
allow means the baseline was never wired, and the stone silently became "define some types."

## Runtime prediction

**25–40 minutes.** Mostly type definitions with one wiring site; the cost is in choosing the wiring
path honestly, not in the code. Predicted overrun: STOP-2 — no existing path knows all eight
baseline facts.

Time-box: 80 minutes.

## Trap doors — named in advance

- **`Option<T>` creeping into the metadata layer.** The design rejects it explicitly (*"gross/unwrap
  culture"*), and this project has been bitten this month: an `Option` whose `None` meant "skip the
  check" collapsed two situations and passed both (arc 278 R66). `MetaField` is a *named* sum for
  that reason. Row 4 exists because this is the easy mistake.
- **A `Default` to make the wiring compile.** Any `Default` on the baseline deletes the forcing —
  the whole point. That is STOP-2 wearing a convenience's clothes.
- **`#[allow(dead_code)]` to make clippy pass.** Same shape: the allow is the tell that "wire it onto
  one path" was skipped. Row 6 catches it; row 2 is what it would hollow out.
- **Building `FnDef` by *moving* fields off `Function`.** That is 255.1b-ii (~31 sites). This stone
  *defines* `FnDef`; it does not split `Function`. A diff that touches `Function`'s fields has
  scope-crept.
- **A fourth stale citation.** The note caught `StructDef`/`RecordDef`/`ProtocolDef` missing. The
  design has been wrong about the disk once per section so far; assume nothing it cites exists until
  grepped. STOP-1.

## What this stone does NOT claim

It registers nothing. It changes no behaviour. It does not close #95, does not touch `wat.type`,
does not carve `runtime.rs`, and does not split `Function`. It lays the types and proves the forcing
is a wall rather than a convention.

Any report claiming more than "the scaffold exists, it is wired, and omitting a field fails to
compile" is overclaiming.
