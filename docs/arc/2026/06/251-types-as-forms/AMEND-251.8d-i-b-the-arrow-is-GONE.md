# AMEND — STONE 251.8d-i-b: the arrow is GONE. `<-` and `->` BOTH become `:-`.

**Builder's ruling, 2026-09-21.** ⛔ **This INVERTS the stone's direction. The orchestrator drew it
wrong.**

> *"in my mind the arrow syntax is gone … `<-` and `->` both become `:-`. **this is what becoming a
> clojure means.**"*

## ⛔ THE ORCHESTRATOR MADE A DESIGN DECISION IT HAD NO STANDING TO MAKE

The brief said *"the annotation rules must not fire on a rete field binding"* — which **preserves
`<-`** at 3,853 sites. That was never ruled. It was decided inside a brief about a codemod bug, and
the builder caught it:

> *"why are we re-introducing the arrow? … I don't recall agreeing to this."*

⚠ **And it sits badly beside 255.2**, where the executor *routed* `is_binder_marker` into one door
precisely so wat would not carry two parameterization recognisers. The orchestrator then preserved
two arrow operators without weighing it. **Eleventh correction.**

⭐ **The strike itself was right and is NOT wasted** — see *What survives* below.

## THE RULING, AS THE CONVERSION MUST NOW READ

| input | output | what moves |
|---|---|---|
| `[x <- :wat::core::i64]` | `[x :- wat.type/i64]` | arrow **and** type |
| `(:vrm::F (?k <- :k))` | `(vrm/F (?k :- :k))` | ⭐ **arrow converts; the FIELD NAME `:k` is PRESERVED** |
| `(:vrm::F ?f)` `-> :T` | `… :- …` | every arrow, everywhere |

⛔ **THE BUG THE STRIKE FIXED IS STILL A BUG.** The old codemod produced `(?k :- k)` — destroying
**both** the arrow *and* the field name. Under this ruling the arrow **must** convert; the field
name **must not**. ⭐ **So `prev-rete-var?` is still exactly the right bit — it just gates the WRONG
RULE.**

**Move it:**

| rule | old gate (wrong) | new gate |
|---|---|---|
| arrow → `:-` | `annotation-arrow?` = `arrow?` AND NOT `prev-rete-var?` | ⭐ **`arrow?` — unconditional. Every arrow converts.** |
| post-arrow keyword → type form | `prev-arrow?` | ⭐ **`prev-arrow?` AND NOT `prev-rete-var?`** |

## THE SECOND HALF — rete must LEARN `:-`

Measured: **rete has never seen `:-`.** Four hard-coded sites compare against `"<-"`:

```
src/rete/clause.rs:365        items[1] … s.as_str() == "<-"
src/rete/clause.rs:390        items[1] … s.as_str() == "<-"
src/rete/expr_ir/mod.rs:1035  ps[i]    … s.as_str() == "<-"
src/rete/kernel/stratify.rs:717  arrow.as_str() == "<-"
```

⛔ **DO NOT add `|| == ":-"` at four sites.** That is four hand-rolled recognisers of the one
parameterization marker, and `tests/lint/one_param_spec.rs` exists to refuse exactly that —
*"`:-` is wat's ONE parameterization operator."* ⭐ **255.2's cure is the model:** `is_binder_marker`
already lives in `wat-reader` as the shared door. **Route all four through it.**

### ⭐ RULED 2026-09-21 — NO TRANSITIONAL DUAL-ACCEPT

> *"rete moves to `:-` — it is not exception."*

⛔ **Rete accepts `:-` and NOT `<-`.** There is no grace period and no second spelling. The four
sites stop recognising `<-` entirely; `is_binder_marker` is the only recogniser.

⚠ **CONSEQUENCE, and it must be faced in this stone:** the moment rete stops accepting `<-`, every
unconverted rete clause in the corpus is **illegal**. The tool change and the corpus conversion
become **one landing**, exactly the shape `wat/fix.wat`'s header calls the **STASH-DANCE**
(*"a codemod that ships ALONGSIDE a checker change that makes the OLD form illegal"*).

⛔ **This is the FIRST stone in the campaign where that applies** — every prior one kept both
spellings legal. **Read `wat/fix.wat` lines 22-53 before you build.** If the dance is needed, say so
and follow it; if it is not, **say why not** — do not discover it at the rebuild.

⚠ **`wat/` is `include_str!`'d and contains rete clauses**, so the stdlib is inside the blast radius:
the binary that must convert the corpus is built from a stdlib that the new checker would reject.
**That is precisely the chicken-and-egg the header documents.** ⛔ **If this cannot be done without
breaking the build, STOP AND REPORT** — that is a finding, not a thing to force.

## What SURVIVES from the strike — do not rewrite it

| | |
|---|---|
| `prev-rete-var?` plumbed through `fix-seq` / `fix-text-seq-edits` | ⭐ **keep** — it gates the type rule now |
| `rete-var?` (`ast-name` starts with `?`) | ⭐ **keep** — measured: every `?`-prefixed binder in code sits in a `(`; the 2 apparent `[?x` are comments |
| the replay fixture with **near-misses** | ⭐ **keep, and INVERT the rete row**: `(?k <- :k)` → `(?k :- :k)` is now a **convert**, not an identity |
| the `~`-named annotation control | ⭐ **keep** — `[& ~call-args-sym <- …]` must still convert |
| ⛔ `annotation-arrow?` | **delete** — the arrow is unconditional now |

## The gate

- ⭐ **THE DELTA (baseline 64, orchestrator's tree) + classification**, one named tree.
  ⚠ **`ReteCheckErrors` 21 should now collapse** — the stone's own finding was that they fail on
  *correctly converted* clauses because **rete's `:when` parser does not accept a symbol-headed fact
  pattern**. ⛔ **That is a SEPARATE cause from the arrow and may survive this stone. If it does, say
  so and do not force it** — it is a checker question, and the executor was right to decline it once.
- ⛔ **ZERO `<-` and ZERO `->` remain as annotation/binding arrows** in any converted output — and a
  **gate** that keeps it so.
- ⛔ **NON-VACUITY BOTH WAYS:** `(?k :- :k)` keeps its **field name**, and `[x :- wat.type/i64]` gets
  its **type converted**. A cure that stops converting types would "fix" the field-name class by
  breaking the annotation class.
- `scripts/floor.sh` green; clippy `-D warnings --all-targets --workspace` **0**; census
  `no STOP-8`. Run crate clippy **and the lint suite** — especially `one_param_spec` — yourself.
- Idempotence; dry-run on `/tmp` copies; ⛔ **NOT ONE corpus `.wat` converted.**
- ⚠ **`wat/fix.wat` is `include_str!`'d — REBUILD before concluding a cure does not work.**

## Fold or redraw

⛔ **Fold into `120c67a4a`.** The machinery is right and the gate moves; this is not a new stone.
**Nothing is pushed.**
