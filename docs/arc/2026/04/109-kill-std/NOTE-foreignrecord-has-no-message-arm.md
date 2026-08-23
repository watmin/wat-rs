# NOTE — a Lex failure inside `fix.wat` cannot REPORT itself: `ForeignRecord` has no `message`

**Found 2026-08-23**, while measuring whether the recorded codemod could run with the arc 109 lexer
wall up (`DESIGN-STONE-annihilate-the-angle-bracket.md`, the sequencing section).

## What happened

With the wall imposed, `angle-brackets-to-binder.wat` was pointed at a file containing an angle form.
The expected outcome was a clean report that the input could not be read. What came back:

```
[#wat.kernel.LociDiedError/RuntimeError ["#wat.runtime/UnknownFunction {:message
 \"unknown function: type `:wat::edn::ForeignRecord` does not implement surface method `message`
 — expected a `defn :wat::edn::ForeignRecord/message` but none is registered\"
 :location #wat.core/Span {:file \"wat-scripts/fixes/angle-brackets-to-binder.wat\" :line 256 …}}]]
```

The lex error never surfaced. What surfaced is the **error path failing on the error**: something in
`fix.wat`'s read-failure branch calls `message` on the returned value, that value is a
`:wat::edn::ForeignRecord`, and `ForeignRecord` carries no `message` arm.

## Why it matters beyond this stone

This is the shape where a diagnostic layer destroys the signal it exists to carry — the same class as
the `… ; echo "EXIT=$?"` wrapper that reported a RED floor as exit 0. Here the true cause (a Lex
error, with a message that names its own remedy) is replaced by an `UnknownFunction` pointing at
line 256 of the *codemod*, which sends the reader hunting in the wrong file entirely.

Every future codemod inherits this: any `fix.wat`-based migration pointed at a file the reader
refuses will report a missing surface method instead of the refusal.

## Scope

Out of `STONE-annihilate-the-angle-bracket`'s scope — that stone's sequencing simply runs the codemod
before the wall goes up, so it never takes this path. Not tracked elsewhere yet; this NOTE is the
record. The fix is one of: give `:wat::edn::ForeignRecord` a `message` arm, or make `fix.wat`'s
read-failure branch stop assuming its error value satisfies that surface.

## Kin

- `[[feedback_a_truncating_pager_makes_absence_unfalsifiable]]` — an instrument that cannot show the
  result reports a null, and the null gets read as an answer.
- `294/SEAM.md`, "the instruments that lied".
