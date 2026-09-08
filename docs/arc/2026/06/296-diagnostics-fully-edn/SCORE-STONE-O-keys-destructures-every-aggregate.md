# SCORE — STONE O: `{:keys …}` destructures every aggregate

No commit. Floor left to the orchestrator (row 8). Lands on the committed RED probe.

The guard was `TypeDef::Aggregate` **and** `nature == Struct`. It is now `TypeDef::Aggregate`.
Runtime had the same nature equality on `Value::Aggregate`; both moved.

## Peer — RULED, not admitted by deletion (STOP-2)

`Nature::Peer` is a `TypeDef::Aggregate` arm (`types.rs` `root_keyword` / `from_root_keyword`).
The predicate is **"is this an aggregate?"** (`TypeDef::Aggregate` / `Value::Aggregate`), not a
nature list (STOP-1). Peer is **in**: destructuring reads named fields; it does not send on the
channel. Purity is the wrong axis for a read. A Peer with no matching field still gets
`keys-destructure: field … is not declared`. HashMap, enum, and surface stay out — they are
other `TypeDef` arms, not aggregates.

## `defholon` — measured (trap door)

`:wat::core::defholon` is **not a declaration**. 294.c.2a deleted its hologram quasiquote;
HolonRecord is `:wat::holon::defrecord` (`wat/Record.wat`). The committed `defholon.wat` fixture
called a missing form (Doctrine 1 on the field types, then a type-var on `{:keys}`). Pointed
at `holon::defrecord`. The HolonRecord kind is covered twice (that fixture + `holon_defrecord`).

## Expectations

| # | result |
|---|---|
| 1 | `probe_arc296_keys_on_aggregates`: **5 passed**, `#[ignore]` = 0 |
| 2 | `./target/release/wat` on all four kinds + binder-first: **EXIT=0**, prints `"3"` |
| 3 | `Some(TypeDef::Aggregate(a)) => a.clone()` — not a nature `matches!` |
| 4 | Peer **in**, as an aggregate. Stated above. |
| 5 | i64 `--check`: `keys-destructure (outcome) expects an aggregate type; got :wat::core::i64` |
| 6 | `git diff -- tests/wat_lang/probe_arc257_keys_destructure.wat` **EMPTY** |
| 7 | rows 1–2 still green |
| 8 | **orchestrator** |
| 9 | `cargo clippy --release --all-targets --workspace`: **0 errors**, 5 pre-existing dead-code warnings |

Goldens recaptured for the message change (STOP-3), not for a wrong value:
`struct_destructure__non_struct_subject_is_clean_type_mismatch.edn`,
`struct_destructure__unknown_field_name_is_clean_malformed_form.edn`.

## STOP rows

| STOP | result |
|---|---|
| STOP-1 nature list | **held.** `TypeDef::Aggregate`. |
| STOP-2 Peer silent | **held.** Ruled in: aggregate, field-read, not send. |
| STOP-3 "expects a struct type" | **held.** `an aggregate type`. |
| STOP-4 checker-only | **held.** Runtime `Value::Aggregate` + `TypeDef::Aggregate`. Fixtures **run**. |
| STOP-5 257.2 probe edited | **held.** EMPTY diff. |

## Working tree

```
src/check.rs
src/runtime.rs
tests/types/probe_arc296_keys_on_aggregates.rs          un-ignore
tests/types/probe_arc296_keys_on_aggregates__defholon.wat
tests/types/struct_destructure__*.edn                   message recapture
```

Do not commit unless a later brief says to.
