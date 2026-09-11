# NOTE — a keyword literal means two things, and only the REGISTRY decides which

**MEASURED 2026-09-11**, `877e59ffb`, floor green. Not a defect to fix now — an **acceptance row
the symbol-head flip must satisfy.** Recorded so the flip OWNS it rather than inherits it.

## The measurement

```
(:wat::core::defenum :u::Op :wat::enum::Pure :Mark [] :Other [])

-> :wat::core::keyword    body  :not-a-variant     check=0   types as keyword
-> :wat::core::keyword    body  :u::Op.Mark        check=1   body produces [:-> :u::Op.Mark]
-> :u::Op.Mark            body  :u::Op.Mark        check=1   body produces [:-> :u::Op.Mark]
-> [:-> :u::Op.Mark]      body  :u::Op.Mark        check=0
```

**A keyword literal's TYPE depends on whether the registry knows the name.** `:not-a-variant` is
`:wat::core::keyword`; `:u::Op.Mark` is a **0-ARY FUNCTION** — the unit variant's constructor.
Identical syntax, two meanings, decided by a lookup.

⚠ **The type system is NOT lying here.** A declared `keyword` return is correctly refused when the
body produces the ctor fn. The dishonesty is upstream, in what the *surface* lets you write.

## The two consequences already on the record

- **The short form is not a narrower spelling — it is a different thing.** `(:u::Box :op :Mark)`
  binds `O` to `:wat::core::keyword`, not to `:u::Op`. Builder: *"use fqdn — this short form is
  illegal."* It is not refused today; it type-checks as something else.
- **`defservice` and user code disagree.** In a `:arms` vector, bare `:-mark` DOES resolve to
  `Op.-Mark` (that narrowing is what produced `Alarm<Op.-Mark>` and the six arc278 reds). In general
  user code, a bare `:Mark` stays a keyword. Two contexts, one syntax, two rules.
- The only spelling that means what a reader expects is the **map form**: `(:u::Op.Mark {})`
  → check 0, and it widens into a `Box<Op>` slot via the lattice. Positional `(:u::Op.Mark)` is
  retired and says so.

## ⛔ WHY THIS IS NOT A STONE TODAY

The conflation exists **because both meanings are spelled as keywords**. The clojure flip —
symbol heads instead of keyword heads — is exactly the separation:

```
u/Op.Mark    a SYMBOL    resolves to the constructor    coherent
:mark        a KEYWORD   is data                        coherent
```

A rule written now would be a rule about the spelling being retired. It would have to be unwritten,
or it would silently constrain the flip's design — `[[feedback_a_rejected_option_returns_in_new_clothes]]`.

## ★ THE ACCEPTANCE ROW THE FLIP OWES

When symbol heads land, these must hold — and each must be PROVEN, not assumed:

```
1. a KEYWORD literal types as :wat::core::keyword ALWAYS — registry membership must not change it
2. a SYMBOL naming a unit variant resolves to that variant's constructor
3. the two contexts agree: defservice's :arms and user code read a bare keyword IDENTICALLY
4. whatever (:u::Op.Mark {}) means today has ONE post-flip spelling, and the retired ones are REFUSED
```

Row 1 is the load-bearing one. It is also the row that makes this finding falsifiable: if, after the
flip, a keyword's type still depends on a lookup, the flip did not separate the two meanings — it
only moved them.

## Adjacent, deliberately not opened

Whether a unit variant should be a 0-ary CONSTRUCTOR at all, or a VALUE (Clojure's
keyword-as-enum shape), is a design question downstream of the flip. Today it is a constructor and
`{}` is how you call it. Named here so the flip can decide it on purpose rather than inherit it.

`[[feedback_a_rulings_premise_expires_but_the_ruling_stands]]`
