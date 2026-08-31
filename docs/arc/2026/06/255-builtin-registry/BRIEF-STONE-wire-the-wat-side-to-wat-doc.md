# BRIEF — STONE: wire the wat side to `wat-doc`

Give `wat-doc` a **metadata-map entry point** beside its text parser, so a wat `defn` can declare its
properties as wat DATA and have them read by the same crate, the same enums, and the same required
-directive enforcement the Rust `///` path uses. Then walk **one** verb through it end to end. DESIGN:
`docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-wire-the-wat-side-to-wat-doc.md`.

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. The orchestrator builds, floors
and clippies centrally — you do not run `cargo build`/`test`/`nextest`/`clippy` or
`scripts/floor.sh`. You may run the pre-existing `target/release/wat` for a fast read, remembering it
does not contain your Rust changes. **You may not spawn sub-agents.** Work only in
`/home/john/work/holon/wat-rs`; verify with `pwd` first. Do not commit, push, stash, revert, or
`git checkout --` anything.

## Read in order

1. The DESIGN above — the pinned contract decision, the finding that made it, and the enum-symbol
   value form.
2. `crates/wat-doc/src/lib.rs` — the crate header (its "ONE shared leaf crate… parity by
   construction" claim), `pub fn parse`, the `DocComment` struct, the `DocError` variants, and the
   required-directive enforcement. **`parse` is the sibling your function must agree with.**
3. `crates/wat-doc/Cargo.toml` — it already depends on `wat-reader`, which re-exports `WatAST`
   (`crates/wat-reader/src/lib.rs:15`). **Your signature needs no new dependency.**
4. `crates/wat-reader/src/ast.rs:161` — `Map(Vec<(WatAST, WatAST)>, Span)`, the shape you read.
5. `src/runtime.rs:855-865` — where a def's metadata map is already captured and inserted into
   `sym.binding_metadata`. Read the comment above it: deleting that insert turns a named test RED,
   so the slot is proven live, not aspirational.
6. `wat/runtime-meta.wat` — the eight `defenum`s whose variants are the legal values.

## The work

### 1 — `wat_doc::from_metadata`

A sibling of `parse`, taking the metadata map instead of joined `///` text and producing **the same
`DocComment`**:

```rust
pub fn parse(raw: &str)                 -> Result<DocComment, DocError>   // unchanged
pub fn from_metadata(map: &WatAST)      -> Result<DocComment, DocError>   // new
```

**It must enforce the same required set and raise the same `DocError`s.** A metadata map missing
`:purity` fails with the *same* error a `///` block missing `@Purity` fails with. Two entry points,
one contract — that is the whole reason this crate exists, and its header says so.

Keys mirror the directives: `:doc` `:added` `:category` `:purity` `:determinism` `:totality`
`:expand-time` `:args` `:ret` `:examples` `:see` `:yields` `:deprecated`.

### 2 — the values are ENUM SYMBOLS, not bare keywords

```clojure
:purity      :wat::runtime::Purity::Pure
:totality    :wat::runtime::Totality::Total
:category    :wat::runtime::Category::Transform
```

A bare `:Pure` is a keyword nothing validates; `:wat::runtime::Purity::Pure` names a variant that
either exists or does not. Reject a value that is not a variant of the axis's own enum, and say
which enum it should have been.

### 3 — walk ONE verb through it

Pick one existing wat `defn` — `:wat::string::capitalize` (`wat/string.wat:17`) is a good candidate:
small, pure, total, and its prose already sits in `;;` comments directly above it, which is exactly
the text that becomes data. Give it a metadata map, and make its declaration reach a place the
reflection layer can read.

⚠ **ONE verb. Not 409.** The stone is the door plus one walk through it.

### 4 — the proof

A probe under `wat-scripts/scratch-pad/`, following the shape of the others there: the chosen verb
still works, and its declared properties are readable. State plainly in your report which part of
the round trip you could and could not verify without a rebuild.

## Blast radius

`crates/wat-doc/src/lib.rs` · the wat def-registration path in `src/runtime.rs` · one `.wat` verb ·
a new probe. No changes to `wat_doc::parse`, to `crates/wat-macros/`, or to any `#[wat_intrinsic]`
doc block.

## STOP triggers — each rejects; ship nothing further on that point and report

**STOP-1 — no new dependency on `wat-doc`.** It is a deliberate leaf ("no signature knowledge, no
registry, no type system, no codegen") and already has `wat-reader`. If your entry point seems to
need anything else — the registry, the checker, the type system — STOP and report: that is the
signal the work belongs in the *consumer*, exactly as `parse`'s own doc says (`@see` resolution and
type-checking are "the consumer's job").

**STOP-2 — do not invent docstrings.** `doc_string: Option<String>` is `None` at all 7 `Binding`
sites and arc 141 never shipped. Populating it, or routing wat prose into `wat_doc::parse`, is
explicitly out of scope — this stone routes DATA. If you believe text is required, STOP and report
why.

**STOP-3 — same errors, not new ones.** If the metadata path needs a `DocError` the text path does
not have, STOP and report which and why. A second error vocabulary for one contract is the drift the
shared crate exists to prevent.

**STOP-4 — do not touch `:defined-in`.** `metadata-of` hard-codes `DefinedIn::Rust`
(`src/runtime.rs:13619`). That is a known defect with its own stone. Registering a wat verb behind
that constant makes it *actively* wrong, so if your work would put a wat verb where `metadata-of`
reports it as `Rust`, STOP and report rather than shipping a new lie.

**STOP-5 — the Rust path stays untouched.** `wat_doc::parse` keeps exactly its two callers
(`wat_intrinsic.rs`, `wat_special_form.rs`). If closing the wat side seems to require changing the
Rust side, STOP and report.

## Report

Per-file diff summary; the signature you landed on and how you kept the two entry points agreeing on
the required set and the error vocabulary; which verb you walked through and how far the round trip
actually got. Then the part the orchestrator cannot reconstruct: what surprised you — a `DocComment`
field with no sensible data form, a place where the text grammar's shape did not translate, or a key
whose name should differ from its directive.
