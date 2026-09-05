# DESIGN — STONE: the exploded form, and blank lines after a complex binder

Two ruled changes. **Sequenced, not bundled** — part 2's trigger depends on part 1's output, so part 1
must be green before part 2 begins.

## PART 1 · R11 becomes ALWAYS-BREAK, with a leading run of atoms

> **Builder:** *"we add compression rules later... get a proper 'verbose' or 'exploded' form first...
> maybe we only support the exploded form"*

R11 today is **all-or-nothing**: it breaks a form's children only if the author already broke some.
It cannot produce an exploded form from a one-liner, which is why `(:wat::core::do (println "a")
(println "b") n)` survives untouched inside `claim-demo.wat`.

**The rule becomes:**

> **A leading run of ATOM children rides the head line. The first COMPOUND child, and every child
> after it, gets its own line.**

★ **This is one rule that reproduces every shape the builder has ruled**, which is why it is stated
this way rather than as "break everything":

```
(wat.hashmap/assoc m              m is an atom          -> rides
  (wat.fmt.Break/id b)            compound              -> own line
  (wat.fmt.Break/kind b))         after a compound      -> own line

(wat.core/foldl                   arg 1 is compound     -> NOTHING rides
  (wat.core/fn …)
  0                               atoms, but AFTER      -> own lines
  xs)

(:wat::core::do                   arg 1 is compound     -> nothing rides
  (:wat::kernel::println "a")
  (:wat::kernel::println "b")
  n)                              atom, after           -> own line
```

⛔ **"Break every child" would be WRONG** — it puts `m` on its own line and contradicts the builder's
`assoc` shape. The leading-atom run is the whole difference.

⚠ **R11 needs no notion of a SLOT.** Slots (`-> T` as one unit, a binder's name-with-value) are a
property of forms that have a GRAMMAR, and those forms have specific rules. The default rule sees an
unruled form: head, then children. Atoms and compounds, nothing else.

## PART 2 · `BlankBefore` — D2-A, four-questioned 4/0

> **Builder:** *"complex forms demands a blank line before next binder"*

```
(wat.core/let
  [x (wat.core/map
       (wat.core/fn …)
       some-coll)
                              ← blank line, because x's value is complex
   j (wat.core/+ 40 2)]
  …)
```

```wat
(:wat::core::defrecord :wat::fmt::BlankBefore
  [id <- :wat::core::i64])
```

**A separate fact, not a third `Break.kind`** — vertical separation is a different axis from indent
discipline, and a node may carry both without either changing the other's meaning.

### What "complex" means — my reading, stated for confirmation

"The value renders multi-line" is circular: it depends on layout, which is what we are computing.
The **structural** proxy that matches the builder's example exactly:

> **A binder's value is COMPLEX when it is a form containing at least one compound child** — because
> that is precisely when Part 1 will break it.

```
(wat.core/map (wat.core/fn …) some-coll)   contains a compound child   -> COMPLEX
(wat.core/+ 40 2)                          only atoms                  -> simple
```

Decidable from the fact base with no width and no layout pass. ⚠ **If the builder means something
narrower, this is the line to correct.**

## ⚠ THE DRIVER-LOADING TRAP — found this session, and it bites this stone

`collect-rules :fmt` gathers rules by NAMESPACE, but only from files a driver has `load-file!`d.
**Adding a file to `rules/` does not add the rule.** It cost three mis-aimed sabotages to find. Any
new rule file here needs its driver edited too — and it is a real dent in *"a new rule is a new
file"* that the arc should eventually close.

## THE ACCEPTANCE — part 1 provable before part 2 starts

```
PART 1
1  claim-demo.wat's one-line `do` EXPLODES — one child per line — and is idempotent
2  the assoc shape: a leading atom RIDES  (fixture: (assoc m (f b) (g b)))
3  the foldl shape: a leading compound means NOTHING rides
4  every existing fixture keeps its ruled shape and is idempotent

PART 2
5  a let with a complex first binder emits a BLANK LINE before the next binder
6  a let with only simple binders emits NO blank line
7  idempotent — a blank line must not accumulate on pass 2
```

⚠ **Row 7 is the one most likely to fail.** An emitted blank line changes the next pass's input; if
the trigger reads anything about the previous pass's output, blanks stack. It must be derived from
STRUCTURE, exactly like indent was in `[[DESIGN-STONE-indent-is-structural]]`.

⚠ **Row 6 is not decoration.** A blank line after every binder is the failure mode that looks like
success on a one-binder fixture.

## OUT OF SCOPE

- **Compression rules.** *"we add compression rules later"* — nothing collapses anything here.
- **R2 `fn`, `foldl`, `map`, `filter` as rule files.** After this stone they are genuinely just
  files: their shape is the default, and the default finally does the right thing.
- **R15 (120) as a lint** — off the critical path since the exploded ruling; still wants the width
  fact when it comes.
