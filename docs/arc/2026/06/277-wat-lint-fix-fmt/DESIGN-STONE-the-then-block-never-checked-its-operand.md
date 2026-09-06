# DESIGN — STONE: a rete `:then` never checked its operand against the field it lands in

> **Builder, 2026-09-06:** *"so... we found a flaw in rete's `:then` block?... those take priority
> to address...."*

Surfaced by `[[SCORE-STONE-node-kind-becomes-an-enum]]`'s honest delta: `Capture.value` printed
`#wat.grep.NodeKind/Symbol []` while `wat/grep.wat:98` declares it `:wat::core::String`. **That is
not a quirk of that migration.**

## THE DEFECT — two doors into one record, and only one of them checks

```
ARM A   rete :then    (:user::Box :label ?n)   ?n is i64, label is String   →  BOXES=1
ARM B   constructor   (:user::Box :label 42)                                →  REFUSED
        ":user::Box: parameter #1 expects :wat::core::String; got :wat::core::i64"
```

`check_rhs_operands` (`src/rete/validate.rs:1144`) is **purely syntactic**. Via
`rhs_operand_can_never_resolve` it asks only *"could this operand ever resolve at fire time?"* —
`IntLit | FloatLit | BoolLit | StringLit | List | ?var`. It never asks *"does it match the field it
lands in?"* `[[feedback_a_slot_with_two_implementations_is_two_slots]]`

## ★ WHAT IT COSTS — a compile error becomes a runtime error, at a distance

```
consumer:  (:wat::string::length (:user::Box/label b))

wat --check   CLEAN            ← the checker believes the RECORD's declaration
runtime       DIES             ← ":wat::string::length: expected String, got wat::core::i64 `42`"
```

The blame lands on the **innocent consumer**, never on the `:then` that lied. That is the whole cost:
not a missing convenience, a **misattributed failure**.

## ⚠ AND THE SEGMENT TRAP — the obvious fix is the wrong tool

`resolve_operand_type` (`validate.rs:1051`) already exists and looks like the answer. **It is not**:
it returns a rete *segment* — `"i64"`, `"string"`, `"enum"` — which is coarser than the declared
type. Two different enums both segment to `"enum"`:

```
(:wat::core::defenum :user::Alpha … )   (:wat::core::defenum :user::Beta … )
:then [(:user::Box :k ?k)]   ?k is an Alpha, :k is declared Beta   →  BOXES=1
```

**Measured. The hole admits enum-into-a-different-enum today, and a segment-based check would keep
admitting it.** The `:where` question (*can rete COMPARE these?*) and the `:then` question (*does
this value FIT this field?*) are different questions; the RHS needs declared-type identity.

## THE MATERIAL IS ALREADY THERE

```
validate.rs:829   lookup_field_types      the destination's declared types
validate.rs:984   collect_rule_bind_types the rule-wide ?var → type map, already built for :when
validate.rs:1404  the kwargs call site    already holds field_names, kv_pairs, types
validate.rs:1416  the positional call site
```

`validate_then_form` receives `types` but not `binds`, and looks up field NAMES but not TYPES.

## ★★ THE SECOND DEFECT, SAME FILE, SAME FAMILY — the message under-reports the language

```
"a RHS operand must be  a ?var bound by this rule's :when,
                        or an integer / float / boolean / string literal"
```

`rhs_operand_can_never_resolve` accepts **`List` too** — arc 278 Stone B removed it from the
never-resolves set. The message never says so.

★ **This is the message that taught the String.** `wat/fmt.wat:9` justified a free-form `kind` with
*"rete RHS … refuses a keyword literal"* — true — and concluded a String was the ceiling, because the
`accepted` list offered nothing else. Both enum stones this session existed to undo that. **Fix the
message and the wrong choice stops being the obvious one.**

## ⛔ THE BLAST RADIUS IS UNKNOWN, AND THAT IS THE FIRST DELIVERABLE

**319 `:then` insert sites** across the corpus. How many store a mismatched type is **not knowable
without imposing the check** — so the stone's first act is to impose it and read the screams, not to
predict them. `[[feedback_impose_the_check_and_read_the_screams]]`

**One scream is known in advance, and it is the previous stone's own defect:**
`tests/cli/wat_grep__g7_rule.wat:26` stores a `NodeKind` into `Capture.value <- String`.

## ⚠ AND ITS RESOLUTION IS NOT THE OBVIOUS ONE — measured

A `:then` **cannot call a user conversion fn**:

```
:then [(:user::Box :label (:user::k-name ?k))]
  →  "then expr is not a rete primitive — ':user::k-name' is not a rete primitive;
      a then admits only :wat::rete:: ops"
```

And **no rete op renders an enum as a string** — the vocabulary has `core::enum::=` and
`core::enum::not=`, nothing else.

So `g7`'s options are narrow, and the DESIGN does not pretend otherwise:

- **capture the literal** — `g7::match-arrow` requires `IsArrow`, which `g7::arrow` asserts only for
  `NodeKind::Symbol`, so `"symbol"` is the truth and not a hard-coding of new knowledge;
- **stop capturing the kind**;
- **mint a `:wat::rete::core::enum::name` vocabulary row** — general, in the same family as
  `enum::=`, and **out of scope here**: minting an op for one caller is speculative until the screams
  say how many callers there are.

**The stone chooses the first, and the screams decide whether the third earns its own stone.**

## THE CONTRACT DECISION

**The check compares DECLARED TYPES, not rete segments, and it reports every mismatch rather than the
first.** Batching is this validator's existing contract (`validate_rete_rules` returns them all).

## WHAT THIS STONE IS NOT

- **NOT a change to the `then-item-fence`.** That a `:then` admits only `:wat::rete::` ops is a
  separate ruling with its own reasons; this stone measures against it, it does not relax it.
- **NOT minting `enum::name`.** Named, scoped out, and gated on what the screams show.
- **NOT `if` / `cond`**, both still ruled and queued.

## FILES

```
src/rete/validate.rs    check_rhs_operands gains the field types + binds; the accepted list
                        gains the call form
tests/rete/…            the negative controls — both defects, both directions
<whatever the screams name>
```
