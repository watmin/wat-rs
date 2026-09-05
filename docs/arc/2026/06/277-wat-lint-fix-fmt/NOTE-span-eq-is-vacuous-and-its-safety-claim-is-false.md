# NOTE — `Span::eq` is unconditionally `true`, its safety claim is FALSE, and a formatter is the worst possible consumer

> **Builder, 2026-09-05:** *"is that 'this is intentional' still warranted?.. was it ever warranted?"*

Asked after the reader stone's rider reported that its own witness would have been vacuous had it
compared `Vec<SpannedToken>` with `assert_eq!`.

## THE MECHANISM

```rust
// crates/wat-reader/src/span.rs:137
impl PartialEq for Span { fn eq(&self, _: &Self) -> bool { true } }
impl std::hash::Hash for Span { fn hash<H: Hasher>(&self, _: &mut H) {} }
```

`WatAST` is `#[derive(PartialEq)]` over 14 variants, every one carrying a `Span`. So the derive
inherits the blindness — which is the point.

## WAS THE REQUIREMENT WARRANTED? YES.

The module doc's reasoning is sound: *"two ASTs with the same shape but different source locations
MUST hash to the same bytes… a parsed-at-runtime AST and a synthetic AST with the same structure
should compare equal regardless of where they came from."* Structural identity independent of
position is a real, load-bearing requirement — macro expansion, caching, and every
`synthetic == parsed` assertion depend on it.

## ⛔ WAS THE IMPLEMENTATION WARRANTED? NO — IT IS BROADER THAN THE REQUIREMENT.

The requirement is about **`WatAST` equality**. The implementation makes **`Span` itself** lie, so
*every* span comparison anywhere in the tree is vacuous — including deliberate ones.

And the doc states a safety claim that is measurably false:

> *"Downstream code that wants to reason about source locations reads the Span's fields directly
> (`file`, `line`, `col`); **it never compares Span values for equality**."*

**It does. Three sites, at least two of which mean it:**

```
src/runtime.rs:19751
    assert_eq!(span, outer_span, "outer-form span should survive a rewrite");
    ⛔ The assertion's ENTIRE STATED PURPOSE is unfalsifiable. It passes if the rewrite
       destroys the span, moves it, or replaces it with any other span in the program.

tests/diagnostics/probe_arc243_stone3_typeerror_pattern_a.rs:106
tests/diagnostics/probe_arc243_stone6_checkerror_pattern_a.rs:111
    let actual_span: &Span = err.span();
    assert_eq!(actual_span, expected_span);
    ⛔ Both are `&Span`. Proves `err.span()` does not panic. Proves nothing about WHICH span.
```

★ And a fourth site shows someone hit this, **misdiagnosed why, and worked around it correctly by
accident** — `tests/value/probe_runtime_error_one_door.rs:38`:

```rust
// `Span` doesn't derive `PartialEq`; compare via `Debug` instead
assert_eq!(format!("{:?}", e.span()), format!("{:?}", span));
```

`Span` **does** impl `PartialEq` — vacuously. The `Debug` comparison is right; the reason recorded
beside it is wrong. **Tenth comment-caused defect of this campaign, and it is inside a workaround
for this very hazard.**

## ⛔ AND A FORMATTER IS THE WORST POSSIBLE CONSUMER

wat-fmt is *entirely about positions*. Comment attachment is a span computation
(`[[DESIGN-STONE-the-reader-can-see-comments]]`). Every future assertion of the form *"this comment
attached to that node"* or *"this span survived the rewrite"* is **silently vacuous** unless written
with `Debug` or hand-read fields — and nothing warns you. The reader stone's rider only avoided it
by noticing the property independently.

**This is a muted wall of exactly the kind arc 255's walls effort exists to find:** an assertion
that cannot fail on the axis it names.

## THE NARROW FIX — preserve the requirement, remove the hazard

```
WatAST      a manual PartialEq over its 14 variants that compares structure and SKIPS the span
Span        compares file/line/col/end HONESTLY
Hash        unchanged — a no-op span hash is still correct and still needed
```

The stated requirement (`synthetic == parsed` regardless of position) is preserved exactly, because
that requirement was always about `WatAST`, never about `Span`. And a span assertion starts meaning
what it says.

⚠ **Blast radius NOT measured.** Anything relying on two `Span`s comparing equal would break — and
by construction those sites are invisible today, because they all currently pass. The three vacuous
assertions above are the ones I found by grep; a `grep` cannot find a comparison inside a derived
`PartialEq` on some other type that carries a `Span`. **The compiler is the census here** — make
`Span::eq` honest and read the failures — and that measurement is the first move of any stone, not
a step inside one.

## STATUS

Recorded, not acted on. It threatens arc 277 specifically, and it is not on any list. The decision
is the builder's: fix it narrowly before wat-fmt writes position assertions against a vacuous
equality, or accept it and require every span assertion to go through `Debug`/fields with a note
saying why.
