# NOTE — the formatter had never met a real file. One run found three defects. (2026-09-05)

> **Builder:** *"so.... 'nothing does it for an fn' .... so.... that's our next step?.. or?...."*

**Or.** Adding `fn`'s arg-spec rule is one more file. But **every measurement in this arc has been a
hand-made fixture of three to eight lines**, and `wat/io.wat` — the only real file used — is 45 lines
and mostly comments. So before another rule, the formatter was pointed at real stdlib.

## WHAT HELD — and it is not nothing

```
wat/deporder.wat   323 lines    FORMS=17  COMMENTS=85   IDEMPOTENT=true
wat/grep.wat       450 lines    FORMS=25  COMMENTS=152  IDEMPOTENT=true

deporder max line   306 -> 163        over 120:  3 -> 2        lines: 323 -> 435 (exploded, as ruled)
```

Idempotent on real input, every comment preserved, and the 306-column monster is gone. The
architecture works.

## ⛔ AND THREE DEFECTS NO FIXTURE COULD EVER HAVE SHOWN

### 1 — a comment's indent POLLUTES the form after it

```
ORIGINAL                                      FORMATTED
;; defined-name — the ast-name of child[1]      ;; defined-name — …        ← indented 2
(:wat::core::defn :wat::deporder::defined-name  (:wat::core::defn …
  [form <- :wat::WatAST]                            [form <- :wat::WatAST]  ← indent 4, not 2
  -> :wat::core::String                             -> :wat::core::String
```

**Two faults, one cause.** A top-level comment is emitted at column 2 instead of 0, and the form
following it then indents its children to 4 instead of 2 — the emitter's column state is carrying
across from the comment.

★ **Every fixture in this arc is a bare form with no leading comment block.** The one real file used
before now, `wat/io.wat`, has comments — but its forms are one-liners, so a wrong child indent had
nothing to show.

### 2 — the formatter CREATES trailing whitespace

```
trailing-whitespace lines:   original 7   ->   formatted 26
```

R9 (*no trailing whitespace*) exists in the style table precisely to remove these. **The formatter is
manufacturing them faster than the rule would clean them** — the same shape as R16's finding, that a
tool which emits un-canonical text manufactures findings for its own linter.

### 3 — two lines still exceed 120

```
163 cols   [referencer <- :wat::core::String referencer-pos <- :wat::core::i64 definer <- …
153 cols   (:wat::deporder::Violation :referencer path :referencer-pos ref-pos :definer d…
```

The first is a `defn` arg-spec that never broke one-per-line — **the same gap as `fn`'s**, and
therefore evidence the gap is wider than one form. The second is a keyword-argument construction,
which R7 covers in the style table and no rule implements.

## ★ THE LESSON, AND IT IS THE ARC'S OWN

`[[feedback_a_design_is_unfalsifiable_until_something_consumes_it]]`. Eight stones were verified
against fixtures I wrote to match the rule I was verifying. **The first contact with real input found
three defects in a single run**, two of which are in the emitter — the one component every rule
depends on.

⛔ **So the next step is not another rule file.** A new rule inherits a broken emitter. The order is:

```
1  the emitter's column state must not leak across a comment      (defects 1 and 2)
2  THEN the arg-spec rules — fn's, and defn's for the long case   (defect 3)
3  THEN wat fmt --check over the corpus, which is the real gate
```

⚠ **And a standing correction to how this arc measures:** a fixture proves a rule fires. **Only a
real file proves the formatter works.** Every stone from here should end by running the corpus, not
a fixture — the cost is one command and it would have caught all three of these eight stones ago.
