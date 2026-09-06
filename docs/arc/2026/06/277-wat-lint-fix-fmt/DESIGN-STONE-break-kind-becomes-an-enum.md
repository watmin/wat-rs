# DESIGN — STONE: Break.kind becomes an enum (and a wall becomes unrepresentable)

> **Builder, 2026-09-06:** *"we are using free form strings for kind... not an enum... we should
> change that"* · *"it feels quite silly that we are using the same string values all over"* ·
> *"break first, then node"*

`:wat::fmt::Break.kind` is a `String` carrying `"block"` or `"align"` across **23 assert sites in 8
rule files**, and the emitter defends the invariant with a runtime raise. It becomes an enum.

## THE BLOCKER COMMENT IS FALSE — measured, with a negative control

`wat/fmt.wat:9-10` says:

> *"kind is a String, not a keyword: rete RHS may insert a string literal but refuses a keyword
> literal (`RhsUnresolvableOperand`). `"block"` | `"align"`."*

True of a **keyword**; the conclusion does not follow. `validate.rs:1060` reads `:name` in a RHS as a
**FIELD REFERENCE**, which is why a bare `:Block` is ambiguous — but an enum variant is a **call
form**, and arc 278 Stone B removed `List` from the never-resolves set.

`wat-scripts/scratch-pad/277-can-a-rete-rhs-carry-an-enum.wat`:

```
:then [(:user::Broken :id ?i :kind (:user::BreakKind::Block))]   →  RULES=1 BROKEN-FACTS=1
NEGATIVE CONTROL — the same slot with a bare :Block             →  RhsUnresolvableOperand
```

## AND THE `:where` SIDE — the one the migration actually lives on

`wat-scripts/scratch-pad/277-can-a-rete-where-compare-an-enum.wat`:

```
:when [(:user::EnumNode (?i <- :id) (?k <- :kind))
       (:wat::rete::where (:wat::rete::core::enum::= ?k (:user::NodeKind::List)))]

three EnumNodes inserted — :List, :Vector, :Keyword  →  ARM-A-enum=1
                                                        ARM-C-string-control=1
```

⚠ **A first draft of this probe named the generic `:wat::rete::core::=` and got
`"is not pure"`, which I read as "rete has no enum comparator."** It does:
**`:wat::rete::core::enum::=` / `::not=` were minted by arc 278 #57** (`vocabulary.rs:1079`) for
exactly this, `rete_type_segment_of` returns `Some("enum")` as a first-class segment
(`validate.rs:868`), and `clause.rs:184` lists `enum` in the `Eq`/`NotEq` families. **The error named
where my instrument gave up, not what the substrate lacks.**
`[[feedback_an_error_names_where_it_gave_up_not_what_is_missing]]`

## THE EMITTER SIDE COMPOSES TOO — proven before the brief

`scratchpad/enum-map.wat`:

```
map-roundtrip-eq=true     cross-variant-eq=false      core::= discriminates on enum VALUES
pad-block=12  pad-align=100                           exhaustive match, NO else arm, dispatches
```

## ★ THE POINT — a WALL BECOMES UNREPRESENTABLE

`wat/fmt.wat:298-312`, `pad-break`, today:

```wat
(:wat::core::if (:wat::core::or (:wat::core::= bk "block") (:wat::core::= bk "align"))
  …
  (:wat::kernel::assertion-failed! "fmt: Break.kind must be block or align" …))
```

A **runtime check standing in for a type** — the CHECK rung of the ladder. With an enum it is a
two-arm `match` with **no else arm**, and the third state has no way to be written down:

```
NEGATIVE CONTROL — drop the :Align arm
  → "non-exhaustive: enum :user::BreakKind missing arm(s) for variant(s): Align"
```

That is the top rung. `extirpare`: convention → check → **a shape the mistake cannot be expressed in.**

⚠ **The OTHER wall is a different invariant and MUST SURVIVE.** `breaks-map` (`fmt.wat:986-999`)
raises `"fmt: conflicting Breaks for node {n} — {a} vs {b}"` when two rules disagree about one node.
That is not the unknown-kind check; it is the ownership wall, it has a sabotage test, and the
migration must keep it firing — with two variants instead of two strings.

## THE SITES — a census, not an estimate

```
23  :then  (:wat::fmt::Break :id ?x :kind "block" | "align")
     defn.wat 3 · defrecord.wat 3 · siblings.wat 4 · kwargs.wat 7 · let.wat 2
     match.wat 1 · defn-args.wat 1 · defrecord-fields.wat 1 · let-bindings.wat 1
 1  read   fmt.wat:986  (:wat::fmt::Break/kind b)
 1  type   fmt.wat:13   kind <- :wat::core::String
 1  map    breaks-map -> HashMap<i64, String>   becomes  HashMap<i64, BreakKind>
 2  wall   fmt.wat:304-312 pad-break (RETIRED into exhaustiveness)
           fmt.wat:995 conflicting-Breaks    (KEPT)
```

★ **23 sites across 8 files is a `.wat` corpus migration, so it is a wat-fix codemod — never hand
edits, never sed.** R21. `wat-scripts/fixes/` holds the recorded shape.

## THE CONTRACT DECISION

**`BreakKind` lives in `wat/fmt.wat` beside `Break`, and is `:wat::enum::Pure` with two nullary
variants `:Block` and `:Align`.** Not a third `:Unknown` — the whole point is that the third state
has no constructor.

## WHAT THIS STONE IS NOT

- **NOT `Node.kind`.** That is the sequel the builder sequenced second: **62** `kind` comparisons
  (`"vector"` 21 · `"list"` 17 · `"keyword"` 12 · `"map"` 7 · `"set"` 5) across 12 rule files, plus
  the producer at `wat/grep.wat:194` and `open-of`/`close-of`. `Break` is the small proving case;
  Node's brief is drawn **after** this lands, against what this measures, not sketched now.
  `[[feedback_a_plan_sketched_n_stones_ahead_names_an_unmeasured_shape]]`
- **NOT `if` / `cond`.** Both ruled, both new rule files, both still queued.
- **NOT the `RhsUnresolvableOperand` diagnostic.** Its `accepted` list omits the call form and is what
  taught the String in the first place — the CLASS behind this bug. It is Rust, it is a separate
  stone, and it is named here so it is not lost.

## FILES

```
wat/fmt.wat                        the BreakKind enum · Break.kind's type · breaks-map's
                                   value type · pad-break becomes an exhaustive match
wat-scripts/fixes/<name>.wat       the recorded codemod that rewrites the 23 assert sites
wat-scripts/fmt/rules/*.wat        8 files, rewritten BY the codemod
```

⚠ **`wat/*.wat` is FROZEN into the release binary** — an emitter edit proves nothing without
`cargo build --release`. Rule files under `wat-scripts/` are read from disk.

⚠ **The codemod runs against a checker that still accepts the old form**, so no stash-dance is
needed here: a String field and an enum field are different TYPES, so the type checker is the
cascade. Expect the build to go red across the 23 sites and waterfall to zero as the codemod lands —
that is `SUBSTRATE-AS-TEACHER`, not a crisis.
