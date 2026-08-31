# BRIEF — STONE: an example is a FORM, not a string

Make `DocExample` hold a parsed form instead of source text, so a malformed `@example` is a **compile
error at the macro** rather than a late reflection-test failure, and so a wat declaration writes
literal syntax instead of an escaped string. DESIGN:
`docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-an-example-is-a-form-not-a-string.md`.

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. The orchestrator builds, floors
and clippies centrally — you do not run `cargo build`/`test`/`nextest`/`clippy` or
`scripts/floor.sh`. You may run the pre-existing `target/release/wat` for a fast read, remembering it
does not contain your Rust changes. **You may not spawn sub-agents.** Work only in
`/home/john/work/holon/wat-rs`; verify with `pwd` first. Do not commit, push, stash, revert, or
`git checkout --` anything.

## Read in order

1. The DESIGN above — **especially the correction**: the form lives in `DocComment`, and the
   registry entry structurally cannot hold one.
2. `crates/wat-doc/src/lib.rs` — `DocExample` (`expr: String`, `expected: Option<String>`,
   `run: bool`), the `@example` text path (`~:596`, `:1317`), and the metadata path (`~:1019`).
   Note `wat_reader::parse_one_with_file` at `:328` — the reader is already a dependency and already
   used.
3. `src/intrinsic/mod.rs:179` — `ExampleSubmission { expr: &'static str, … }`. **This does not
   change.** It is `&'static` const data a proc macro emits; a `WatAST` cannot live there.
4. `crates/wat-macros/src/wat_intrinsic.rs:791` — where the macro turns `DocComment.examples` into
   those `&'static` literals. It must keep emitting **text**, and it has the text.
5. `src/intrinsic/reflect.rs:93` (`parse_one_with_file(ex.expr, …)`) and `:522`
   (`out.push_str(ex.expr)`) — the two registry-path consumers, both reading `&'static str` from the
   entry. **Neither changes in this stone.**

## The work

### 1 — `DocExample` holds a form

`expr` becomes a parsed `WatAST`; `expected` becomes an optional one. Both entry points produce it:

- **the `///` text path** parses its example text with the reader it already has;
- **the metadata path** already *has* a form — it stops calling `metadata_describe` to stringify it.

### 2 — the macro still emits text

`ExampleSubmission` stays `&'static str`. The macro parsed the text; it emits the **original text**.
The change is *when validation happens*, not what the registry stores.

★ That is the whole win: **a malformed `@example` fails the build.** `Record/field-at` shipped
`#=> <r's first field's value>` and it surfaced far downstream as `TrailingContent`.

### 3 — the wat side writes literal syntax

`wat/string.wat`'s `capitalize` becomes:

```clojure
:examples [{:expr (:wat::string::capitalize "object") :expected "Object"}
           {:expr (:wat::string::capitalize "")       :expected ""}]
```

⚠ Keep whatever key shape `from_metadata` already accepts if it differs — **do not redesign the
metadata spelling in this stone**; only stop stringifying the value.

### 4 — the probe

`wat-scripts/scratch-pad/255-probe-an-example-is-a-form.wat`, following the shape of the others.
Show `capitalize` still works and its examples still read back.

⚠ The negative — a malformed example — is now a **compile** failure, so it cannot live in a committed
`.wat` (the loader gate would go red) *or* in a committed `.rs` (it would not build). Demonstrate it
once out-of-tree and report what you saw, the way the last stone's rider did.

## Blast radius

`crates/wat-doc/src/lib.rs` · `crates/wat-macros/src/wat_intrinsic.rs` (emission only) ·
`wat/string.wat` (one declaration) · possibly `src/intrinsic/reflect.rs` **only if** a signature
forces it · the new probe. `ExampleSubmission` does not change.

## STOP triggers — each rejects; ship nothing further on that point and report

**STOP-1 — `ExampleSubmission` stays `&'static str`.** If making `DocExample` hold a form seems to
require changing the registry entry, STOP and report. That struct is const data emitted by a proc
macro; a `WatAST` cannot live there, and discovering otherwise would be a finding worth the stop.

**STOP-2 — no second representation.** Do **not** store both a form and its source text on
`DocExample`. Two copies of one fact drift, and that is the defect this arc has paid for repeatedly.
If rendering seems to need the text, STOP and report where.

**STOP-3 — do not add a dependency to `wat-doc`.** It has `wat-reader` already. If parsing or
rendering seems to need more, STOP and report — that is the signal the work belongs in the consumer.

**STOP-4 — do not redesign the metadata key shape.** `from_metadata` already accepts some spelling
for `:examples`. Stop stringifying; do not rename or restructure the keys in the same stone.

## Report

Per-file diff summary; the `DocExample` shape you landed on; how each entry point produces a form;
what the macro emits now; and the out-of-tree demonstration that a malformed example fails the build.
Then the part the orchestrator cannot reconstruct: what surprised you — an example in the corpus that
does not parse as a single form, a consumer of `expr` the DESIGN did not name, or a place where the
text path and the metadata path could not be made to agree.
