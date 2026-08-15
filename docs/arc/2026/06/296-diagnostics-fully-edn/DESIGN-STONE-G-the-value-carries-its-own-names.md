# 296 · DESIGN STONE G — the value carries its own field names

> **STATUS: SHAPE DESIGNED, NOT BUILT.** The two generator arms it needs are LANDED and green.
> The ~150-site migration is rider work and has not been done. This doc is the brief.

## THE DEFECT

`Value::Aggregate` carries only positional `fields`. Naming them at render time therefore needs an
external registry lookup — and the four ways that lookup can fail all collapse into one `_` arm
answering `{:field-0 1 :field-1 2}`.

That is not a degraded rendering. It is **a lie with a plausible shape**, and it shipped: `str`
for twelve hours under a green 8/8 probe, `send'`'s wire (bridged with a thread-local in 258.5b),
and every failing `deftest`'s diagnostic. Builder: *"the field-??? values are dishonest."*

**The sibling variant already knew better.** `Value::ForeignRecord` carries its own keys and has
never had this bug — `edn_shim` says why, verbatim: *"keys/fields are SELF-carried (not
registry-looked-up, which would fall to `field-{i}` and lose the foreign names)."*

## THE SHAPE

`AggregateValue` gains one field:

```rust
pub names: Arc<Vec<String>>,   // declaration order, same length as `fields`
```

and all three constructors take it: `struct_(class, names, fields)`, `record(class, names, fields)`,
`holon_record(class, names, fields, hologram)`.

**This does not fix the four causes — it deletes the question.** No registry is consulted, so an
absent class and a shape disagreement cannot arise; and because names are supplied ALONGSIDE the
values by whoever builds them, a names/values arity mismatch is unrepresentable rather than
rendered.

## ⛔ WHERE THE NAMES COME FROM — never a human's fingers

This is the part the first attempt got wrong and the builder stopped.

| site kind | source of names |
|---|---|
| holds a registry | `AggregateDef::names_arc()` — **LANDED** (`src/types.rs`) |
| type known statically, no registry | a `wat_field_names_from!` const — **LANDED** (`wat-source-derive`) |
| rebuilding from an existing value | `a.names.clone()` — carry the source value's own |
| generic constructor (`struct-new`, `Record::of`) | registry lookup; an unregistered class is an **ERROR**, not a fallback |

The first draft gave the static sites `static_field_names!("message", "location", "causes")` — a
hand-transcription of a declaration that already exists. Builder: *"we did that exact move
recently?"* A literal there is a SECOND place the names are stated, and a **right-count/wrong-name**
literal renders confidently and wrongly — worse than the `:field-N` this arc annihilates, because
it looks like an answer.

`wat_field_names_from!(FAULT_FIELDS, "wat/core.wat", ":wat::core::Fault")` reads the same `.wat`
declaration that `wat_record_from!` reads to generate the type's registration. One source, and no
arm of this design has a human typing a field name into Rust.

## THE WORKED EXEMPLAR (built, measured, then reverted with the rest of the shape)

```rust
::wat_source_derive::wat_field_names_from!(FAULT_FIELDS, "wat/core.wat", ":wat::core::Fault");

/// `OnceLock` so a hot error path allocates the name vector once, not per raised fault.
fn fault_names() -> Arc<Vec<String>> {
    static N: std::sync::OnceLock<Arc<Vec<String>>> = std::sync::OnceLock::new();
    N.get_or_init(|| AggregateValue::names_from_static(FAULT_FIELDS)).clone()
}

pub(crate) fn fault_from_runtime_error(err: &RuntimeError) -> Value {
    Value::Aggregate(Arc::new(AggregateValue::record(
        "wat::core::Fault".to_string(),
        fault_names(),
        Arc::new(vec![ /* … */ ]),
    )))
}
```

## THE WORKLIST — measured by imposing the change and reading rustc

Adding `names` to the constructors produced **97 errors** across:

| file | sites |
|---|---|
| `src/value/value.rs` | 54 |
| `src/runtime.rs` | 44 |
| `src/rete/kernel.rs` | 24 |
| `src/edn_shim.rs` | 12 |
| `src/rete/matcher.rs` | 4 |
| `src/rete/purity.rs` · `compiled_rhs.rs` · `intrinsic/reflect.rs` · `freeze.rs` · `channel/transfer.rs` · `capability/registry.rs` | 2 each |

**Impose the check and read the screams** — do not survey for the worklist. Every earlier count on
this arc taken by grep was wrong (four times); the compiler's is not.

## THEN, AND ONLY THEN — delete the fallback

`src/edn_shim.rs` holds **7** `format!("field-{}", i)` sites across four clusters
(`value_to_json_natural` struct + enum arms, `value_to_edn_with` struct + Aggregate arms), plus
three silent `return vec![]` arms in `enum_variant_field_names` that a caller turns into the same
positional output. All of it goes: with names on the value there is nothing left to fall back
*from*.

An earlier session deleted these fallbacks WITHOUT G and read the screams: **4 reds out of 4413**,
each one real — a heretic test that pinned `{:field-0 3 :field-1 4}` as its EXPECTED value, a CLI
freeze-panic path whose doc comment claimed *"those values only carry primitive Strings"* (false),
and two self-inflicted. That measurement is the disconfirming evidence that the fallback is not
load-bearing.

## STOP TRIGGERS

- **STOP-1 — a construction site has no honest source of names.** Do NOT invent a literal. Report
  it; a site that cannot name its own fields is the finding.
- **STOP-2 — a generic constructor reaches an unregistered class.** Raise. Do not fall back to
  positional; that is the defect returning under a new name.
- **STOP-3 — the arity of `names` and `fields` disagree at any site.** The two are built together
  by construction; a disagreement means the site is assembling them from different places.

## WHAT IS ALREADY LANDED (green, do not rebuild)

- `wat_field_names_from!` — `crates/wat-source-derive` (this commit)
- `AggregateDef::names_arc()` — `src/types.rs` (this commit)
- all 13 builtin aggregates declared in wat + their registrations generated (`e79322c0`,
  `f806a4db`, `9f07564b`, `0514498c`)
- the differential gate on the generator's source files (`473f9373`)
