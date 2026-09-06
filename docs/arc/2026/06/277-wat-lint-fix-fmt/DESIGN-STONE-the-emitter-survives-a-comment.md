# DESIGN — STONE: the emitter survives a comment

> **Builder:** *"we need to work on the comments stuff now ..... not later.... the comments declared
> above... they need to align with the code lines after them."*
> ```
> (wat.core/defn c/go
>   [x :- wat.type/i64]
>   :- wat.type/i64
>   ;; no allowed new lines after ret-spec.. body must begin....
>   (wat.i64/+ x 1))
>
> ;; comments (or any other forms) must have a line after a top level form....
> ```

## THE DEFECTS — measured on a minimal input, exact columns

```
INPUT                                        OUTPUT
;; a leading comment above the form          ';; a leading comment above the form'   ✓
(:wat::core::defn :c::go […] -> i64  ;; A    '(:wat::core::defn :c::go'              ✓
  ;; B                                       '  [x <- :wat::core::i64]'              ✓
  (:wat::i64::+ x 1))  ;; C                  '  -> :wat::core::i64'                  ✓
                                             '  '                    ⛔ D — blank line, 2 spaces
                                             '  ;; A'                ⛔ E — was TRAILING
                                             '  ;; B'
                                             '(:wat::i64::+ x 1))'   ⛔ F — indent LOST, 0 not 2
                                             ';; trailing on the body'
```

**Everything is correct until a comment is emitted.** The leading comment at column 0 is already
right; it is the **code after it** that breaks — exactly as the builder said.

### The four, and their causes

```
D  a blank line carrying trailing whitespace       the emitter writes indent, then a newline
E  a TRAILING comment is lifted onto its own line  every comment is emitted as a LEADING comment
F  the node after a comment loses its indent       the emitter's column state does not survive
G  (on wat/spawn.wat) tag padding lost across a comment   same cause as F
```

★ **F and G are one bug.** Fix the column state and both go.

## WHAT IS RULED

```
1  a comment above a form takes THAT FORM's indent            ← the builder, directly
2  NO blank line between the ret-spec and the body            ← "no allowed new lines after ret-spec"
3  a blank line REQUIRED after every top-level form           ← "must have a line after a top level form"
4  a blank line carries NO trailing whitespace                ← R9, drafted since the first table
```

⭐ **Rule 3 ratchets what the corpus already does.** Measured across `wat/`: **178 top-level forms
have a blank line before them, 15 do not.** The rule fixes fifteen sites and codifies 92%.

⚠ **Rule 2 has a wider blast radius: 869 blank lines currently sit inside a started top-level form.**
Most are deliberate paragraph breaks inside long bodies. **Rule 2 as ruled forbids one specific
position — between ret-spec and body — NOT every blank line inside a form.** Reading it wider would
delete 869 author-placed separations, and this DESIGN does not.

## ⛔ WHAT IS **NOT** RULED — and must not be inferred

**Defect E: where does a TRAILING comment go?**

```
(:wat::core::defn :c::go [x <- i64] -> i64  ;; documents the RET-SPEC
```

Today it is lifted onto its own line above the *next* node, so it silently re-attaches to something
it was not written about. The builder's example shows a comment **already on its own line** above the
body — which does not answer what should happen to one that starts trailing.

★ And there is a case with no obvious answer, which is why the reader stone called this **policy**:
**a trailing comment on a form the formatter EXPLODES has no single line left to belong to.**
`wat/spawn.wat`'s `:Shutdown  ;; owner dropped the handle…` survives because a variant is one line;
a trailing comment on a `defn`'s head line does not.

> **This stone fixes D, F and G — the mechanical ones — and leaves E untouched and reported.**
> A comment that is trailing in the input stays trailing where its line survives, and where the line
> does not survive the stone must SAY SO rather than choose.

## THE ACCEPTANCE

```
1  ★★★ the node after a comment KEEPS its indent          (defect F — the builder's ask)
2  ★★ wat/spawn.wat's variants stay at indent 2 across their trailing comments, tags still padded
3  ★★ blank lines carry NO trailing whitespace            (defect D)
4  ★ no blank line between ret-spec and body
5  ★ exactly one blank line after each top-level form
6  ★★ a blank line INSIDE a body is PRESERVED — rule 2 is one position, not a purge
7  idempotent — the required blank must not accumulate on pass 2
8  ★★ wat/io.wat still COMMENTS=28, and its 85-comment sibling wat/deporder.wat unchanged in count
9  ★★★ REPORTED, not fixed: what became of each TRAILING comment
```

⛔ **Row 6 is the row that stops rule 2 becoming a massacre.** 869 blank lines sit inside top-level
forms; a strike that reads rule 2 as "no blank lines in a form" passes rows 4 and 5 and destroys
them all.

⚠ **Row 7 is where the required-blank rule characteristically fails**: pass 1 inserts the blank,
pass 2 must see it as already present. Same class as `EmptyVecAfter`'s `[] []`.

## OUT OF SCOPE

- **Defect E** — above; the builder's policy call.
- **R8 aligned trailing comments** — needs E settled first.
- **The `:examples` fat arrow** — deferred.
