# DESIGN — a doc directive may wrap

> *"how can we enable multiline exprs even if we don't have wat-fmt built yet? they can exist in a
> non-linted state until then?"*
> *"for context, there's a bunch a very long oneliner doc comments - i don't want those"*
> — the builder, 2026-08-23

## The defect: a wrapped directive is SILENTLY DISCARDED

Both doc parsers in `crates/wat-doc/src/lib.rs` walk doc lines like this:

```rust
if !tag.starts_with('@') {
    // Non-directive lines after the first directive (e.g. blank lines) — skip.
    continue;
}
```

A continuation line is not *unsupported* — it is **eaten, with no error**. An author who wraps an
`@example` loses the second half and is never told. The rational response is to write one very long
line, and that is exactly what the corpus shows:

```
43 doc lines > 120 columns
16          > 160
10          > 200      ← 6 of the ten are @example / @example-norun, 1 is @ret
```

**The long one-liners are not a style lapse. They are the shape the validator forces.**

## Arc 141 deferred the wrong half, and its condition never fired

`141-core-form-docstrings` is where the smart docs start. Its DESIGN has a *"What this arc does NOT
decide"* section, and multiline is there — twice:

> - *"**Multi-line docstring rendering rules** (STYLE-RULES amendments) — refined **when wat-fmt sees
>   doc-bearing fixtures**."*
> - *"Code-example syntax inside docstrings — refined alongside wat-doctest **if that future arc
>   opens**."*

★ **Both defer to a CONDITION, not to a named arc.** Nothing could be tracked, so nothing was, and the
condition never fired. That is FM 11's shape — *"future arc when X surfaces"* — appearing in a DESIGN's
scope-cut rather than an INSCRIPTION. Legitimate to cut; fatal to track that way.

And the cut is **the wrong half**. `RENDERING` — how to lay a wrapped directive out — genuinely needs
wat-fmt. `ACCEPTANCE` — whether the second line is read at all — never did.

```
ACCEPTANCE   wat-doc joins continuation lines into one payload    ← buildable now
RENDERING    wat-fmt's STYLE-RULES amendments                     ← still deferred, still unbuilt
```

**You do not need to know how to pretty-print a thing to stop discarding it.** The builder's
"non-linted state" is precisely right: accept the wrap, impose no style on it, and let wat-fmt add the
style rules when it exists.

## Why the change is small

Every directive arm already consumes a single string:

```rust
let payload = trimmed[tag.len()..].trim_start();
match tag { "@arg" => …, "@example" => …, "@ret" => … }
```

If the payload is assembled *with continuations joined* before dispatch, **every arm works unchanged** —
`@example`'s `#=>` split, `@arg`'s three-token grammar, all of it. Wat is whitespace-insensitive, so a
single space is the correct joiner for expressions and prose alike.

## The termination rule — today's behaviour preserved, minus the loss

```
line starts with `@`   →  begins a new directive
blank line             →  ends the current one      (today's "skip blank lines", kept)
anything else          →  CONTINUES it              (today: silently discarded)
```

Only the third row changes, and it changes a silent discard into a read.

## ⛔ It must land in ONE place

There are **two** parsers — `parse` (line 265) and `parse_special_form` (line 567) — with two copies of
the line walk AND two copies of the recognized-tag list, differing by one tag (`@yields` vs `@syntax`).
Adding the continuation rule twice would be the tenth instance of "a slot with two implementations is
two slots" in this session.

The rule goes in one helper — raw lines → `(tag, payload)` pairs, continuations joined — and both
parsers consume it.

## What this does NOT do

- **No STYLE rules.** No column limit, no wrap policy, no reflow. That is wat-fmt's, still deferred —
  and this stone deliberately leaves wrapped directives *unlinted*.
- **No rewrapping of the 43 long lines.** Once wrapping is possible, an author may wrap; a mechanical
  reflow is a separate, cosmetic sweep and needs the style rules this stone does not define.
- **No change to any directive's grammar.** Every arm sees the same payload shape it sees today.

## The four questions

- **Obvious?** YES. A wrapped directive reads as one directive, which is what every author already
  assumed — that assumption is why the discard is silent and harmful.
- **Simple?** YES. One helper, one termination rule, zero changes to any arm.
- **Honest?** YES, and it is the failing axis: the validator currently reads half a directive and
  reports success. A parser that silently drops input is lying about what it validated.
- **Good UX?** YES — the 200-column `@example` lines exist *because* the alternative was data loss.
