# DESIGN — the doc validator adjudicates with the READER, and the prose follows

> *"rip the heresy being spoken from our repo — no reader of wat shall ever make the mistake on how to
> communicate parameterized types"*
> *"those comments are validated - our docs are 'smart' - how are these allowed to be expressed?"*
> — the builder, 2026-08-23

## The builder's question found the defect, and it is not the prose

`<K,V>` cannot be written, minted, rendered, or parsed. It is still **published** — in `@arg` / `@ret`
annotations across `src/intrinsic/`, which are the user-facing API reference for the language:

```
/// @ret     :wat::core::Option<wat::kernel::Process<I,O>>
/// @arg     listener :wat::kernel::Listener<S,R> the listener to accept a connection from
```

These are **validated**. `crates/wat-doc` parses the directive grammar, enforces tag order, rejects
separators in the type position — and then adjudicates the TYPE itself like this, at **five** sites:

```rust
// Type token must start with `:` (all wat types are keywords).
if !ty_token.starts_with(':') { … }
```

**The first character is the entire type check.** So `:wat::kernel::Listener<S,R>` passes: it starts
with a colon. The validator validates the *grammar of the annotation* and never asks whether the type
it names can exist.

★ That is the arc's recurring shape one more time — **a shape test standing in for a parse**, hand-rolled
five times, in the one place whose output users read to learn the language.

## The fix is the reader, and it is available and cycle-free

`wat-doc` → `wat-source-derive` → `wat-reader`, and `wat-source-derive`'s own manifest states the
reason it exists: *"Depends on nothing of wat's but wat-reader, so both wat-doc and the main crate can
use it **without a cycle**."* `wat-reader` depends only on `wat-edn`. A direct dependency adds nothing
to the graph.

**Measured, running the real lexer over doc type tokens:**

```
:wat::kernel::Listener<S,R>                  REFUSED
:wat::core::Listener<S>                      REFUSED     ← single-param too
:wat::core::Bytes                            LEXES
:wat::core::<                                LEXES       ← the operator survives
(:wat::core::Vector :- [:wat::core::i64])    LEXES       ← the surviving spelling
```

The reader already answers this question correctly and completely. The doc validator should ask it
instead of guessing from the first byte.

## Why this beats the sweep I was about to brief

I had drafted a classified prose sweep plus a rune over comment TEXT. That design was worse on every axis:

- **It could not be exhaustive.** A regex over comments cannot separate a wat type from a Rust generic
  — 801+ of the 1,662 `.rs` hits are `Vec<T>`, `Option<String>`, `Arc<Function>`, and a blind pass
  would have rewritten Rust into wat syntax.
- **It was a second opinion about what a type may be spelled.** Exactly the thing this whole campaign
  has been deleting.
- **It rots.** A sweep is a moment; a validator is a property.

With the validator taught, **the 506 `.rs` doc lines announce themselves at build time, by file and
line**, and Rust generics in ordinary comments are untouched because only `@arg`/`@ret` TYPE TOKENS are
adjudicated. The census stops being something I have to get right.

## What ships

```
1. wat-doc gains a direct wat-reader dependency
2. the five `starts_with(':')` type checks become ONE call to the reader
3. the build screams; every @arg/@ret naming an inexpressible type is fixed
4. the .wat corpus prose — 798 lines / 252 files — swept under FM 14's A/B/C/D
```

Steps 1-3 are the stone. Step 4 is its sibling and is genuinely a sweep, because a `;;` comment is
not validated by anything and never will be — there is no door to teach.

## ⛔ What must survive

- **`:wat::core::<`** and the operator family. Measured above: the reader lexes them.
- **Rust generics in Rust comments.** Only `@arg`/`@ret` type tokens go through the reader.
- **Class C prose — the lines RECORDING the retirement.** ~45 of the 798. *"`Head<K,V>` was the only
  construct that gave a comma meaning"* is the record of why the language has its shape; deleting it
  leaves the law looking arbitrary. Earned `rune:lint(...)` exemptions with a reason, or the sweep's
  own classification.
- **`docs/arc/**` and `.wat.bad`.** Archived, and negative fixtures that must keep their illegal text
  or they stop testing the refusal.

## The four questions

- **Obvious?** YES. "What may a type be spelled?" gets one answer — the reader's — everywhere,
  including in the documentation.
- **Simple?** YES. Five shape tests collapse to one call. The stone deletes a hand-rolled opinion.
- **Honest?** YES, and this is the axis that was failing hardest: the API reference for the language
  published a spelling the language refuses, and the validator that was supposed to catch it was
  checking a colon.
- **Good UX?** YES — an author who writes an impossible type in a doc comment learns at build time,
  at their own line, instead of shipping it to a reader who then can't compile what they copied.
