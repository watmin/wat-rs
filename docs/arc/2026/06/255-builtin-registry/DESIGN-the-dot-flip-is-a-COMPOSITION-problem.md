# DESIGN — the dot flip is a COMPOSITION problem, not an acceptance problem

> Builder: *"`:user::something::Enum.Variant` (`user.something/Enum.Variant`)"*

The blanket is dead (`c3fefc5ab`), so the flip is unblocked. Measured first, before drawing any of
it — and the shape is not what "teach the reader to accept a dot" suggests.

## What a user enum does TODAY

```
(:wat::core::defenum :user::app::Box :wat::enum::Pure :Full [inside <- :wat::core::i64] :Empty [])

INPUT   (:user::app::Box::Full {:inside 42})   →  runs
OUTPUT  #user.app/Box.Full {:inside 42}        →  ALREADY the target notation
INPUT   (:user::app::Box.Full  {:inside 42})   →  refused at resolve, exit 3
INPUT   (user.app/Box.Full     {:inside 42})   →  refused, exit 3
```

★ **The runtime already WRITES the spelling the reader will not ACCEPT.** A program cannot read back
what it just printed.

★★ And the two refusals converge on ONE candidate: `user.app/Box.Full` normalizes to
`:user::app::Box.Full`, exactly as it should. **The reader and the normalizer are already correct.**
Nothing needs teaching about dots. What is missing is that `Enum.Variant` and `Enum::Variant` are
built in different places by different code and nothing makes them agree.

⚠ Both refusals are LOUD only because the blanket died. Before `c3fefc5ab`,
`(:wat::core::Option.Some {:value 7})` type-checked clean and silently answered
`#wat.core/Option.None {}`. That is the seam's ordering caveat, discharged.

## ★★★ THE MEASUREMENT — fifteen hand-rolled compositions, two spellings, zero doors

```
13 sites   format!("{}::{}", <enum path>, <variant name>)     the LOOKUP spelling
 2 sites   format!("{enum_leaf}.{variant_name}")              the RENDER spelling
 0         composition functions in the one-name grammar
```

Every one of the thirteen was read: all thirteen join `(enum_path, variant_name)`. No false
positives.

```
src/runtime.rs:4108   ⚠ "{}::{}", e.type_path.trim_start_matches(':'), e.variant_name
src/runtime.rs:8628 · 8828 · 8968      "{}::{}", ev.type_path, ev.variant_name
src/closure_extract.rs:2251            "{}::{}", ev.type_path, ev.variant_name
src/rete/expr_ir.rs:1320               "{}::{}", e.type_path, e.variant_name
src/declare/register.rs:1341 · 1370    "{}::{}", enum_def.name, variant_name
src/declare/preregister.rs:314         "{}::{}", type_name, variant_name
src/value/observe.rs:363               "{}::{}", ev.type_path, ev.variant_name
src/types.rs:709 · 1028                "{}::{}", name, variant_name
src/check.rs:1870                      "{}::{}", enum_path, variant_name
src/edn/render.rs:3918 · 4628          "{enum_leaf}.{variant_name}"
```

⚠ **THE DRIFT HAS ALREADY STARTED.** `runtime.rs:4108` strips a leading `:` before joining; the
other twelve do not. One of fifteen already disagrees about what it is composing, and nothing can
tell you which is right, because there is no definition to be right about.

## The grammar has ten readers and no writer

`crates/wat-reader/src/identifier.rs` — the one-name grammar — exposes
`leaf` · `path` · `receiver` · `method` · `prime` · `deprimed` · `namespace` · `is_reference`,
and exactly one constructor, `bare`. **Every accessor DECOMPOSES a name. Nothing COMPOSES one.**

So a name that must be split is split by the grammar, once, consistently — and a name that must be
BUILT is built by whoever needs it, fifteen times, two ways.
`[[feedback_a_derived_structure_makes_every_reader_guess]]`

## The one contract decision

> **A variant's name has ONE composition function, and every site calls it. The separator is that
> function's decision, made once.**

This is the campaign's own sentence — ONE QUESTION, ONE ANSWER — applied to name composition
instead of name membership. It is the same move that closed the blanket: not "make the fifteen
agree by discipline," but "make it impossible for them to disagree."

★ **Then the flip is a one-line change with a compiler-checked blast radius.** Today, flipping the
separator means finding fifteen `format!`s by grep and hoping the grep was a census —
`[[feedback_reach_for_the_compiler_before_the_fourth_grep]]`. After the door, it means changing one
function body, and every consumer follows by construction.

★★ **And the two spellings can coexist safely during migration**, because one function decides. The
reader accepting `Enum.Variant` and the renderer emitting `Enum.Variant` stop being two
independently-maintained facts.

## The order

```
1  MINT THE DOOR         one composition fn in the one-name grammar, beside its decomposers
2  IMPOSE IT             point all 15 sites at it — the compiler produces the worklist, and
                         `runtime.rs:4108`'s trim is either right for everyone or wrong for it
3  THEN FLIP             the separator becomes one decision in one body
```

⛔ **Step 2 before step 3, always.** Flipping first means fifteen edits, a grep for a census, and a
silent wrong answer for anything missed. This is the same ordering argument the blanket's death
just discharged, one layer down.

## Out of scope = REJECTED

- **Teaching the reader about dots.** It already reads them; `user.app/Box.Full` normalizes to
  exactly the right keyword today.
- **`:wat::core::Option.Some`** and the stdlib's own variants. Same mechanism; they follow the door,
  they do not need their own.
- **Retiring the `::` spelling.** A separate, later decision. This design makes it a one-line one.
