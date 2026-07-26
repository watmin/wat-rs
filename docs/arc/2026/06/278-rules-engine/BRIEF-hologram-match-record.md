# BRIEF — `Hologram/find` returns a RECORD, not a tuple: `:wat::holon::Match`

> **Builder ruling, 2026-07-26:** *"i think we need a record to hold this, not a tuple.. if we need
> a holon-entry so be it."* The name and its fields are settled three ways — the builder's call, an
> independent intueri cast that reached `Match` / `key` / `value` without being told his choice, and
> the orchestrator's own weigh. **Do not re-open the naming.**
>
> Small stone: one Rust return type, one Rust construction, one wat caller, one registration.

## What ships

```clojure
(:wat::holon::Match [key <- :wat::holon::HolonAST  value <- :wat::holon::HolonAST])

;; before —  Hologram/find (h, probe) -> Option<(HolonAST, HolonAST)>
;; after  —  Hologram/find (h, probe) -> Option<:wat::holon::Match>
```

## Why a record, and why `Match` specifically

A Hologram matches by **similarity**, so the key `find` hands back is **not necessarily the probe**
— it is whatever coincided above the filter's floor. That asymmetry is the entire reason `find`
exists as distinct from `get` (`get` answers *"what value did my probe reach?"*; `find` answers
*"WHICH stored key did my probe reach, and what did it hold?"*). A caller reaches for `find`
precisely because it needs that matched key back — `HolographicLru::get` bumps that key's recency
and cannot bump what it cannot name.

`Match` carries that asymmetry in the word itself, the way every regex API does: nobody reads
`match.group()` and assumes it equals the pattern. `Entry` would say "a stored pair I looked up",
which is the `get`-shaped mental model this type must not invite.

Fields stay `key` / `value`, matching `:wat::cache::Entry`'s precedent — the type name carries the
semantics, so a field called `matched-key` on a type called `Match` would be redundant.

## Read in order

1. **`src/check.rs:17845-17851`** — `Hologram/find`'s registered `TypeScheme`. Its `ret` is
   `Option<Tuple(holon_ty, holon_ty)>`; it becomes `Option<Path(":wat::holon::Match")>`.
   **Its two siblings need NOTHING** — `Hologram/get` (`:17833`) and `Hologram/remove` (`:17856`)
   both return `Option<HolonAST>`, a single value, no pair. Leave them exactly as they are.
2. **`src/runtime.rs:16844` `eval_hologram_find`** — ~32 lines in, the `Some((k, v))` arm builds
   `Value::Tuple(Arc::new(vec![holon(k), holon(v)]))`. That becomes a record value of
   `:wat::holon::Match`. Ground how a builtin record value is constructed at runtime before you
   write it; do not guess the `Value` shape.
3. **`src/types.rs:1052-1070`** — the `:wat::kernel::Frame` registration is your exemplar for
   declaring a builtin record: `env.register_builtin(TypeDef::Aggregate(AggregateDef { nature:
   Nature::Record, name, type_params: vec![], fields, restrictions: None }))`. `Match` is the same
   shape with two `HolonAST` fields. Note `Frame`'s header comment explains *why* its fields are
   what they are — write `Match`'s the same way, and make it carry the probe≠key fact.
4. **`wat/cache.wat`, `HolographicLru::get`** — the ONE live caller. Today it reads
   `(first pair)` / `(second pair)`; it becomes `(:wat::holon::Match/key m)` /
   `(:wat::holon::Match/value m)`. The other caller in `crates/wat-holon-lru/` is a dying oracle —
   **do not touch it**; it is scheduled for annihilation and is not on the build path.

## The gate

The existing `wat-tests/cache/HolographicLru.wat` already exercises this path hard (four tests, and
`test-get-bumps-recency` fails if the matched key is read wrong). It must stay green **unchanged**
— that is the regression proof.

Add one small direct gate that the four cache tests cannot give you: a `deftest` that calls
`Hologram/find` and reads `Match/key` and `Match/value` **by name**, asserting the matched key is
the STORED key when probed with a *different but coincident* value. That is the assertion the record
exists for, and a tuple could not express it legibly.

## STOP triggers — rejection criteria; report and ship nothing further

1. **If a builtin record value cannot be constructed from `eval_hologram_find`'s position** — STOP
   and report what the runtime actually needs. Do NOT fall back to keeping the tuple and lifting it
   in wat: the whole point of this stone is fixing the SOURCE, since there is exactly one caller.
2. **If `Match` collides with anything** — STOP and report. (`:wat::core::match` is a special FORM,
   lowercase, different namespace; FQDN-always means neither ever appears bare, so this is expected
   to be a non-issue. If the checker disagrees, that is the finding.)
3. **If the blast radius exceeds `src/check.rs` + `src/runtime.rs` + `src/types.rs` +
   `wat/cache.wat` + the new gate** — STOP and report before spending it.

## Method

- **You MAY run `cargo build --release`** — you will need it: `wat/cache.wat` is baked into the
  binary via `include_str!`, so wat-side edits are invisible to `--check` until you rebuild.
- **Do NOT run `cargo nextest`.** The orchestrator measures the floor centrally, once.
- `target/release/wat --check <f.wat>` after a build is your gate; read its printed output, never
  `$?` through a pipe.
- Run everything in the FOREGROUND to completion. Do not launch a command in the background and
  return control.
- Scratch `.wat` goes in `wat-scripts/scratch-pad/` and is loader-gated — green, or deleted.
- Do not commit.

## Your report

The diff shape; the new gate's assertion quoted from a real run; confirmation the four existing
`HolographicLru` tests are still green **unchanged**; confirmation `Hologram/get` and `/remove` were
left alone; any STOP. No test-suite numbers — those are the orchestrator's.
