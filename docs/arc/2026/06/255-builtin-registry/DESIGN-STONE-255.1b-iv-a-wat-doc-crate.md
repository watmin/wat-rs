# Stone 255.1b-iv-a — the `wat-doc` leaf crate (the shared prose+@tag parser)

**Why this stone, why first.** The doc/reflection contract (§10) makes one load-bearing decision: the
prose+`@tag` parser + the mutual-checks live in **ONE shared leaf crate**, depended on by BOTH
`wat-macros` (proc-macro, intrinsics) AND `wat` (runtime/checker, wat forms) — so the two paths
**cannot drift**. Parity by construction, not by discipline. Everything else in 255.1b-iv (macro
enforcement, doctest-gen, the wat-side `defn` docstrings) consumes this crate. So it is built first,
in isolation, fully unit-tested, before anything wires to it.

**Blast radius / floor.** Purely additive: a new workspace member `crates/wat-doc`. Nothing depends on
it yet (iv-b wires `wat-macros` to it). The existing floor cannot break — the only build-graph change
is adding the member. The probe is the crate's own unit tests.

## The one contract decision — the `DocComment` model + the parse/check API

The crate's public surface (pinned; the executor fills the bodies, does not invent the shape):

```rust
/// One parsed `@example` / `@example-norun` directive.
pub struct DocExample {
    pub expr: String,             // the wat form, verbatim (left of `#=>`), trimmed
    pub expected: Option<String>, // right of `#=>`, trimmed; None when no marker
    pub run: bool,                // true = @example (doctested); false = @example-norun
}

pub struct DocArg {
    pub name: String,   // first whitespace-delimited token after `@arg`
    pub desc: String,   // remainder, leading separator (` — `/` -- `/` - `/`: `) stripped, trimmed
}

pub struct Deprecation {
    pub since: String,       // first token after `@deprecated`
    pub use_instead: String, // remainder, trimmed
}

/// A fully-parsed doc comment with all *universal* required directives present.
/// (The `@arg`⇄signature match is NOT enforced here — see `check_args`; it needs
/// the signature, which only the consumer holds.)
pub struct DocComment {
    pub prose: String,                   // GFM body: everything before the first `@`-tag line, trimmed
    pub added: String,                   // @added <ver>
    pub args: Vec<DocArg>,               // @arg ×N (count checked against the signature, not here)
    pub ret: String,                     // @ret <desc>
    pub examples: Vec<DocExample>,       // @example | @example-norun, ≥1 (either kind satisfies)
    pub deprecated: Option<Deprecation>, // @deprecated <ver> <use-instead>
    pub see: Vec<String>,                // @see <fqdn> ×N
}

/// Parse failure. Carries enough to render a precise `compile_error!` (macro)
/// or a runtime/check error (wat). Closed enum — diagnostic completeness by shape.
pub enum DocError {
    MissingProse,
    MissingAdded,
    MissingRet,
    MissingExample,                                   // need ≥1 @example|@example-norun
    MalformedDirective { tag: String, why: &'static str },
    UnknownDirective { tag: String },
    ExampleMissingMarker { expr: String },            // @example without `#=>` (norun may omit)
    DuplicateSingleton { tag: String },               // a second @added / @ret
    // mutual-check failures (raised by check_args, same enum):
    ArgCountMismatch { documented: usize, signature: usize },
    ArgNameMismatch { position: usize, documented: String, signature: String },
}

/// Parse a joined `///` block (one string, `\n`-separated) → DocComment,
/// enforcing the UNIVERSAL required directives (prose, @added, @ret, ≥1 @example).
pub fn parse(raw: &str) -> Result<DocComment, DocError>;

/// The `@arg` ⇄ signature mutual check. `params` = the wat-arg names, in order.
/// A 0-param intrinsic must document 0 args; an N-param one must document N with
/// matching names in order. This is what makes "@arg required ×params" true.
pub fn check_args(doc: &DocComment, params: &[&str]) -> Result<(), DocError>;
```

### Grammar (pinned, v1 — line-based)

- The raw is the joined `///` text. **Prose** = every line before the first line whose first
  non-whitespace token starts with `@` (a recognized directive). Prose is GFM, kept verbatim, trimmed
  of leading/trailing blank lines. Prose is **required** (non-empty) → `MissingProse`.
- A **directive line** begins (after optional leading whitespace) with `@<word>`. Recognized:
  `@added`, `@arg`, `@ret`, `@example`, `@example-norun`, `@deprecated`, `@see`. Any other `@word`
  → `UnknownDirective` (fail loud; no silent skip).
- `@added <ver>` — required, singleton (a second → `DuplicateSingleton`). Value = remainder trimmed;
  empty → `MalformedDirective`.
- `@arg <name> [sep] <desc>` — repeatable. name = first token; desc = remainder with a leading
  ` — `/` -- `/` - `/`: ` stripped, trimmed; empty desc → `MalformedDirective`. (Count/names checked
  by `check_args`, not `parse`.)
- `@ret [sep] <desc>` — required, singleton. desc = remainder, leading sep stripped, trimmed; empty
  → `MalformedDirective`.
- `@example <expr> #=> <expected>` — repeatable. MUST contain `#=>` (split on first occurrence):
  expr = left trimmed, expected = `Some(right trimmed)`, `run = true`. No `#=>` → `ExampleMissingMarker`.
- `@example-norun <expr> [#=> <expected>]` — repeatable. `#=>` OPTIONAL: if present, `expected =
  Some(...)`; else `None`. `run = false`.
- At least one `@example` OR `@example-norun` → else `MissingExample`.
- `@deprecated <ver> <use-instead>` — optional, singleton. since = first token; use_instead = remainder.
- `@see <fqdn>` — optional, repeatable. value = remainder trimmed. (Registry-resolution is NOT this
  crate's job — that's a consumer check, iv-b/255.1b-v.)
- v1 constraint, NAMED not silent: each `@example(-norun)` is **one line**. Multi-line examples are a
  future extension if a real intrinsic needs one — not built now (no forcing function for a hypothetical).

### What this crate is NOT (affirmative cuts, out-of-scope = rejected)

- **No signature knowledge.** `parse` never sees the fn signature; `check_args` takes `params` from the
  caller. (The macro passes its sniffed arg idents; wat passes the `defn` params.) This is what keeps
  the crate a pure leaf both sides can depend on.
- **No `@see` registry resolution, no `@arg`/`@ret` type-checking.** Those need the registry / the
  `TypeScheme` — consumer-side, later stones (iv-b, 255.2). This crate is text → structured model +
  the two checks that need no external state (required-present, arg-name/count-vs-supplied-params).
- **No doctest generation.** That's `wat-macros` emitting Rust from `DocComment.examples` (iv-b).
- **No enum-valued metadata.** `Kind`/`DefinedIn`/`Layer` are the registry's (iv-c), not the doc model's.

## Probe (disconfirming, RED at skeleton)

The crate skeleton ships with `parse`/`check_args` as `todo!()` and a unit-test module asserting the
exact parse of the real `core::Bytes::to-hex` doc block (the reference intrinsic) + the negative cases
(missing prose, missing @added, @example-without-`#=>`, unknown directive, arg count/name mismatch).
At skeleton the tests panic/fail (`todo!`) → RED on exactly the gap (the parser body). The executor
fills `parse`/`check_args` until green. The tests ARE the spec.

## Sequencing out

iv-a (this) → iv-b (wire to the macro; the integration + decorate-Bytes forcing function) → iv-c (enum
flip). The wat-side `defn` docstring path reuses `wat-doc` verbatim in the post-Rust-side decoration
sweep (§10, FUTURE).
