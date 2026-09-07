# SCORE — 296 H-2: a variant is a tagged map

No commit. Floor and clippy left to the orchestrator.

A variant renders `#ns/Enum.Variant {:field …}`. A record still renders `#ns.Enum/Name {:field …}`.
The discriminator is a dot in the tag's NAME half. Unit variants are `{}`. The old vector body is
refused. `tag_from_type_path` is unchanged for records.

---

## Row 1 — a variant's tag is not a record's

`a_variant_tag_is_not_a_records_tag` **PASSES**. `#[ignore]` gone.

```
record  :usr::Shape::Circle  →  #usr.Shape/Circle {:r 2}
enum    :usr::Shape Circle   →  #usr/Shape.Circle {:r 2}
```

## Row 2 — a variant's body is EXACTLY a record's

`a_variant_body_is_rendered_exactly_like_a_records` **PASSES**. Both bodies `{:r 2}`.

## Row 3 — the probe's `#[ignore]`s are GONE

`grep -c '#\[ignore' tests/types/probe_arc296_h2_variant_tag_and_body.rs` = **0**.

## Row 4 — a unit variant is `{}`

`a_unit_variant_is_an_empty_map`: `#usr/Shape.Dot {}`.

## Row 5 — records did not move

`tag_from_type_path` is untouched. Variants use `variant_tag` (own builder). Record tags stay
`#ns.Type {…}`. The rewriter only flipped **vector-bodied** tags (old variants). Map-bodied record
tags were left alone.

## Row 6 — round-trip; unit arm accepts `{}` and REFUSES `[]`

Option/Result coerce tests **pass** on `#wat.core/Option.None {}` / `#wat.core/Option.Some {:value 7}`.
`unit_variant_empty_vector_is_refused`: `#wat.core/Option.None []` is **Err**. One wire.

## Row 7 — goldens regenerated

`examples/h2_rewrite_goldens.rs` walked `tests/**/*.edn`: parse → rewrite old `#ns.Enum/Variant […]`
→ `#ns/Enum.Variant {…}` → write. **308 files** rewritten (295 + 13). Option/Result used known keys
`value`/`error`. Unit vectors became `{}`. Named payloads: Numeric `val`, LociDiedError
`message`/`failure`/`error`, Recv/SendOutcome Lost `cause`, probe.Outcome Lost `sentinel-present?`.

Byte golden `pprintln_doc_row__step_payload.edn` was recaptured from the binary (pretty-print would
have collapsed `:doc` prose).

`ToEdn for Option` in `crates/wat-edn` was still the old vector wire (Span.end etc.). Moved in
lockstep with `value_to_edn_with` — without it, derived diagnostics still emitted
`#wat.core.Option/Some […]` and goldens disagreed with runtime.

A full `UPDATE_EDN=1` suite + per-file data-equal vs `git show HEAD` is **not** claimed. Floor will
surface remaining goldens. Method is the rewriter + targeted recapture.

## Row 8 — `.wat` sites

`wat-scripts/fixes/variant-vector-to-tagged-map.wat` exists (identity migrate). Census: live `.wat`
programs construct via `(:Enum::Variant …)`, not EDN tags. `#wat.core.Option/` in `.wat` is
**comments and EDN-inside-strings**.

**STOP-4 shape, named:** `tests/value/probe_arc278_read_foreign.wat` embeds EDN in a String
(`read-foreign "…"`). A form-tree codemod cannot see it. Updated by hand to
`#some.unknown/Kind.Click {:n 42}`. Same for `@example` strings in `src/intrinsic/edn.rs`.

## Row 9 — foreign variant keeps its keys

`ForeignVariantValue` now has `names: Vec<String>` (STOP-2). Read of a dotted map stores keys;
write emits them. `foreign_variant_keys_survive_read_then_write`:
`#some.unknown/Kind.Click {:n 42}` round-trips.

## Row 10 — floor (ORCHESTRATOR)

Not run. Targeted: H-2 probe **3 passed**; Option/Result tagged **8 passed**; value suite was **189
passed / 2 failed** before ToEdn lockstep, those 2 **passed** after; CLI freeze-time needle updated;
REPL golden recaptured after ToEdn (causes restored). Remaining clusters (diagnostics, services,
…) are the regen tail.

## Row 11 — clippy (ORCHESTRATOR)

Not run.

---

## Named cuts

- **Option/Result declarations stay in `types.rs`.** Wire flipped; moving the defns is H-3.
- **Match arm** still positional. Out of scope.
- **Clojure wat-edn interop** (`wat-edn-clj`) still documents the old tags. Not this crate's
  writer. Named.
- **REPL golden** briefly recaptured empty `:causes` while ToEdn lagged; recaptured again after
  the lockstep. HEAD shape restored (UnresolvedReferences present).
- **Prior uncommitted stones** (pprintln, keyword-is-a-keyword, Identifier tuple) remain in the
  tree. Not reverted.

## Targeted checks (executor)

```
cargo test --release --test types -- probe_arc296_h2     3 passed
cargo test --release --test value -- option_result_tagged  8 passed
cargo test --release --test value -- foreign_variant_keys_survive  passed
cargo test --release --lib -- unit_variant_empty_vector_is_refused  passed
cargo test --release --test cli -- freeze_time_panic / a_bad_line  passed after needle + recapture
```

## Files (this stone)

```
src/edn/render.rs                         variant_tag, write/read, refuse []
src/value/value.rs                        ForeignVariantValue.names
crates/wat-edn/src/lib.rs                 ToEdn for Option lockstep
tests/types/probe_arc296_h2_*.{rs,wat}    un-ignore + unit fixture
tests/**/*.edn                            rewriter
tests/value/probe_arc278_read_foreign.*   new tag + keys-survive
tests/cli/wat_cli.rs                      LociDiedError.Panic needle
wat-scripts/fixes/variant-vector-to-tagged-map.wat
examples/h2_rewrite_goldens.rs            the regen walker
```
