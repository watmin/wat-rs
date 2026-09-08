# SCORE — STONE N RELAND 5: re-sweep with the repaired tool

No commit. Floor left to the orchestrator. Lands on DRAW N RELAND-5 + CURARE(294).

The last full sweep predates RELAND-2's fmap repair (`e77311bf9`). RELAND 3/4 closed named
failures by hand. This strike re-applied the repaired wrap over the **full** 1875-file worklist
(`wat/`, `wat-tests/`, `wat-scripts/`, `tests/`), then a second pass, then only the screams.

## Pass 1 — repaired wrap, full worklist

1875 paths, 8 parallel chunks, `./target/release/wat ./wat-scripts/fixes/positional-ctor-to-map.wat`.

| | |
|---|---|
| visited | 1875 / 1875 |
| sha256 vs pre-sweep | **28** changed, 1847 unchanged, 0 missing |
| of those 28 | 1 is the skip-path edit (below); **27** are wrap |

The 27 wrap files are one class the repaired tool now sees:

| class | files | example |
|---|---|---|
| user tagged enum | `tests/types/enums_tagged_variant.wat` | `Event::Candle 100.0 105.0` → `{:open 100.0 :close 105.0}` |
| generated `EchoResponse::Ok` | 20 arc-170/278 probes | `Ok (concat …)` → `Ok {:reply (concat …)}` |
| generated `RunResponse::Ok` | 4 s2s probes | `Ok out` → `Ok {:out out}` |
| generated `InstallResponse` | `probe-arc278-rules-cross-the-wire.wat` | `Rejected "declared"` → `Rejected {:reason "declared"}`; `Derived v` → `Derived {:n v}` |
| unit variants | `277-the-node-kind-boundary.wat` | `NodeKind::IntLit` → `NodeKind::IntLit {}` |

The brief's named proof file `tests/types/enums_mixed_unit_tagged.wat` was **already** map-form
at HEAD (`Event::Open {:size …}`). The same class was still live in `enums_tagged_variant.wat`
and the generated response ctors above. Sweeping the failing subset would have missed them.

Skip-path added (census bait, STOP-3): `tests/cli/grep_smoke_target.wat` and prefix
`wat-scripts/fixes/`. `wat/service.wat` and the wrap script itself were already skipped.

## Pass 2 — 0 bytes

Same 1875 paths, same 8 chunks, all `EXIT=0`. sha256 vs post-pass-1: **changed=0**.

Idempotence holds. It is necessary, not sufficient (STOP-4). The bar is the floor.

## UNRESOLVED screams — read, classified, three hand-edits

Pass-1 logs: **9494** screams, **1049** unique heads, **9340** unique sites.

The tool reports every Pascal-leaf (and unquote) head whose fields it cannot type-of. That is
**not** "this is a positional enum ctor I refused to guess." Most screams are not wrap work.

### Declined, not hand-edited (why)

| class | why the tool declined | action |
|---|---|---|
| operators (`=`, `+`, `-`, `*`, `<`, `>`, `->`, `->>`, …) | `pascal-leaf?` treats a non-letter first char as uppercase (`to-uppercase("=") == "="`) | not an enum ctor |
| collections / types (`PersistentVector`, `Tuple`, `List`, `PersistentMap`) | `fill-enum` only fills `TypeBody::Enum` | not an enum ctor |
| records (`EchoRequest`, `Record`, `State`, `WriteLogsRequest`, `Node`, kwargs `PageState` / `Tally`) | not in fmap as enum variants; many already kwargs | record grammar, not 296 M |
| rete patterns (`:dea::Ok`, `:nc::Ok`, `:neg::Ok`, …) | Pascal `Ok` leaf; these are fact patterns, not ctors | leave |
| unit records (`Shape::Nothing`, `Mini::PingRequest`, `Span::CloseRequest`) | `defrecord []`, not `defenum` | leave |
| already-map generated responses the fmap still misses | report fires on Pascal leaf even when the arg is already `{:k v}` | already legal |

### Hand-edited (the scream named a real positional enum ctor)

All three hide the `defenum` **inside a `defmacro` quasiquote**. `file-decls` only walks
top-level declaration heads, so type-of never sees the fields. Same class as RELAND-4's
let/do splice.

| site | scream | wrap | why declined |
|---|---|---|---|
| `tests/macros/probe_do_splice_enum_via_macro.wat:12` | `:my::probe::Event::Created` | `Created 1` → `Created {:id 1}` | defenum lives in the macro body, not a top-level decl |
| `tests/macros/probe_let_splice_enum_via_macro.wat:12` | same | same | same, `let []` wrapper |
| `wat-scripts/scratch-pad/probe-sift-rules-stop1-dump.wat:48` | `:probe::Wrapped::EchoResponse::Ok` | `Ok (EchoRequest/c req)` → `Ok {:c (EchoRequest/c req)}` | defenum is quasiquoted DATA inside `defmacro :probe::wrapped-surface`; never registered |

`--check` on all three: **0**. No other positional enum ctor remained in the scream list.

## Census bait (STOP-3)

| path | result |
|---|---|
| `tests/cli/grep_smoke_target.wat` | **UNCHANGED**. Still parse-only bait `(:wat::core::Some 1)` / `Ok` / `Err` |
| `wat-scripts/fixes/*.wat` | skipped by prefix. Only `positional-ctor-to-map.wat` moved, and that is the skip-path edit, not a wrap |
| `wat/service.wat` | **UNCHANGED** (already skipped) |

## Probe / clippy

- `probe_arc296_enum_map_ctor`: **5 passed** (control, map payload, unit `{}`, Option map, retired positional refused)
- positional control fixture still has `(:probe::Box::Full 7)` — skip-path held
- `cargo clippy --release --all-targets --workspace`: **0 errors**, 5 pre-existing dead-code warnings

## Floor

**Not run.** Orchestrator.

Delta this strike can name: 27 wrap files + 3 scream hand-edits + skip-path. The rest of the
1875 were already map-form or not enum ctors.

## STOP rows

| STOP | result |
|---|---|
| STOP-1 hand-edit a failing file before the full re-sweep | **held.** Sweep first; hand-edits only after pass 2, only screams |
| STOP-2 scope to the failing subset | **held.** 1875 paths, not the wake list |
| STOP-3 census bait migrated | **held.** skip-path; bait files byte-identical |
| STOP-4 idempotence offered as done-ness | **held.** Pass 2 is 0 bytes; floor is the orchestrator's |
| STOP-5 wall weakened | **held.** Positional control still positional; retired ctor still refused |
