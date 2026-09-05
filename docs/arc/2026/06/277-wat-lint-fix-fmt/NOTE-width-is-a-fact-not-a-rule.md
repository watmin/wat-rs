# NOTE — rete CANNOT derive width. The DESIGN's mechanism is wrong, and the repair is small. (2026-09-05)

## What was probed and why

The builder's 120-column budget and tiered `fn` need a form's width **as it would render on one
line** — not its current span width, since the current text may already be broken. That number must
be derived, and a parent's derivation depends on its children's.

`[[DESIGN-wat-fmt-the-rule-set-is-the-product]]` named the mechanism confidently and nothing had ever
driven it:

> *"a node's rendered WIDTH depends on its children's widths → bottom-up derivation to a FIXPOINT"*
> *"does this form fit the budget? → acc::sum over the matched child set"*

**Both halves are individually right. Together they are recursion through an aggregate.**

## ⛔ THE ENGINE REFUSES IT

`wat-scripts/scratch-pad/277-width-fixpoint-probe.wat`, run on one file:

```
stratify: negation cycle detected — rule set is not stratifiable
```

The interior rule computes `Width` by `acc::sum` **over `Width`** — the relation it derives.
Stratified evaluation forbids that, and this engine implements it (`src/rete/kernel/stratify.rs`,
`native_stratify_fix`, mirroring `wat/rete/oracle/stratify.wat`).

**Isolated, not guessed.** The identical rule with one change — aggregating over `Node` instead of
over its own `Width` output — runs and prints. Nothing else differs. The blocker is recursion
through aggregation, not syntax, not arity, not my rule.

★ **This is what a disconfirming probe is for.** Had the tiering been briefed on the DESIGN's
sentence, the rider would have hit this wall with a brief that presumed it impossible-by-
construction — the arc-143 failure mode the recovery doc records at ~2 hours a time.

## ✅ THE REPAIR — width is a FACT, computed by the walk

`wat/grep.wat` already computes `Node` / `Named` / `Span` by **walking the tree in ordinary wat**,
where a post-order fold has no stratification problem whatsoever. Width belongs in that walk:

```
leaf      width = length(ast->source node)
interior  width = 2 delimiters + Σ children + (n-1) separators   ==   Σ + n + 1
```

`wat-scripts/scratch-pad/277-width-as-a-fold.wat` implements it and it works.

⭐ **The DESIGN's acceptance criterion is untouched.** *"A new style rule is a NEW FILE and nothing
else"* — rules consume `Width` exactly as they consume `Span`. Width stops being rule logic and
becomes fact-base logic, which is where the other four facts already live.

## ★★ THE CONTROL, AND IT FOUND SOMETHING

For a form already on ONE LINE, the derived width must EQUAL its actual span width. The probe prints
**CHECKED beside MISMATCH**, always — a bare "0 mismatches" is indistinguishable from "0 forms
examined", and that vacuous green was published once already today
(`[[feedback_a_green_test_can_prove_nothing]]`).

```
wat/io.wat          73 checked      0 mismatch      18 quasiquote chars
wat/rete.wat       721 checked      0 mismatch      48
wat/grep.wat      1015 checked      5 mismatch     196
wat/fix.wat       4189 checked     10 mismatch     291
wat/core.wat      5488 checked    437 mismatch    1216
wat/service.wat   6922 checked   1833 mismatch    2153
```

**18,408 forms checked.** The mismatch count is monotonic in quasiquote density across all six
files, and the cause is documented in the substrate already — `wat/fix.wat`'s own header:

> *"a synthesized keyword's Span covers 2 raw source chars while its ast-name claims an 18-char
> canonical token"*

A reader-synthesized node (`~x`, `` `x ``, `~@x`, `\c`) carries a span that does **not** cover its
rendered text. `wat/service.wat:2949` is literally `~handle-record`: derived 19, actual 1 — the span
covers the `~`.

**So the fold is right and the CONTROL is invalid on synthesized nodes, not the derivation.** The
distinction already has a name in the fact base: `wat/grep.wat`'s `Written` fact exists precisely to
say *"this span holds this node's own text"*, and the control must join it.

## What this changes, concretely

1. **`wat/grep.wat` gains a `Width` fact**, emitted by the same walk that emits `Span`. It is the
   fifth fact, not a new subsystem.
2. **The DESIGN's fixpoint/`acc::sum` sentence is retired** and this NOTE replaces it. `acc` keeps
   every other job it has; it simply cannot close a loop over its own output.
3. **The width control joins `Written`** before it can be a gate.

## ⚠ NOT MEASURED — stated so it is not assumed

- **A multi-line string literal's leaf width** counts the newline characters inside `ast->source`.
  Every form in the control is single-line, so the control cannot see this case. It needs its own
  test before `Width` becomes a gate.
- **Whether the mismatches are ENTIRELY synthesized nodes.** The correlation is monotonic across six
  files and one instance was read by hand; the residue was not enumerated. `Written` makes that
  enumerable — a mismatch on a `Written` node would be a real defect in the fold.
- **Cost.** The fold is O(nodes) on a tree already being walked, but it was never timed.
