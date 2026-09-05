# DESIGN — STONE: the rete vocabulary enters the registry, all 37, and the two orphans with it

> **Builder, 2026-09-04:** *"reduce is an alias on foldl ... so we need aliases to just delegate to
> their target's properties... cond should be labeled the same as if?... and reduce is an alias to
> foldl?.... and the rete flavors have further restrictions than the core... we forced all of rete
> into totality... core isn't there (yet)"*
>
> Then: *"you've named our next steps... so ... we step forward... and we measure where we land"*

## THE GROUND — every number asked of the registry, not grepped

```
RETE_OPS                    74 rows · 37 registered · 37 NOT
of the 37 unregistered:
  core_name not registered .. 2   :wat::rete::core::cond · :wat::rete::core::reduce
  core_name is itself an ALIAS  0   (nothing would CHAIN)
  CLEAR, registerable today . 35
rows with NO native handler  51   — a row can be a pure DECLARATION
```

Re-derive: `wat-scripts/scratch-pad/255-b0-what-actually-gates-the-rete-rows.wat` (alias rows as
`name -> target`), `…/255-b0-name-and-totality.wat` (`name|totality` for all 556),
`…/255-b0-rows-without-handlers.wat`. Join against `RETE_OPS` in `src/rete/vocabulary.rs`.

## ⛔ THE FORK THIS STONE CLOSES — alias vs RESTRICTION, resolved AGAINST restriction

The SEAM has carried this open: *"the 8 blocked rete equality rows point at the GENERIC core_name.
An alias means IS; these are RESTRICTED TO. The registry cannot say that."*

**Measured, they are not restricted-to.** The totality demand is **contextual, not a property of
the name**: `compile-condition` (`wat/rete/compile.wat`) consults `:Total` for a `where` clause,
and `:wat::rete::string::concat` is a registered row carrying `Partial` that is legitimately
called OUTSIDE a `where` — in `def` bindings and value positions across `wat-scripts/`.

```
the registry answers   "what are THIS NAME's properties?"   → the delegate's. INHERITED.
the fence answers      "may this expression appear HERE?"   → a CONTEXT predicate.
```

Two questions, two authorities, no overlap — so the RULING's one-authority rule is not violated,
and a row claiming `Total` because rete usually demands it would **over-claim at every non-`where`
call site**. Three rows already inherit non-Total poles faithfully and are correct:
`:wat::rete::core::List` and `:wat::rete::map::contains-key?` (Unreviewed),
`:wat::rete::string::concat` (Partial) — each exactly its target's.

★ And the contract is already written down, at the home these rows go in
(`src/intrinsic/special/rete_alias.rs:88`): *"an alias's axes ARE the target's, resolved by the
registry after every submission has folded, not restated here where they could disagree."*
The builder's *"aliases just delegate to their target's properties"* is the shipped design.

**What the 35 will inherit, measured:** `Partial 14 · Unreviewed 13 · Total 7 · Preserving 1`.
The 13 Unreviewed are **ungraded, not conflicting** — the 270 both-axes batch clears them and
gates nothing. The 14 Partial are mostly `=`/`not=` (deliberately Partial since this session — their
domain admits `Fn`) plus `first`/`Vector`/`PersistentMap`.

## THE THREE PARTS

### 1 · `:wat::core::cond` — a declaration row labelled as `if`

`cond` is a stdlib `defmacro` (`wat/core.wat:1455`) and expands to chained `if`. `rete/purity.rs`
already classifies it clause-aware *as* chained `if`. So its axes are `if`'s, measured at
`src/intrinsic/special/control_flow.rs:25-29`:

```
@Purity Preserving · @Determinism Preserving · @Totality Preserving
@ExpandTime Legal  · @Category ControlFlow
```

`Preserving` is the correct pole three times over: a form whose properties ARE its operands'.
The row carries no handler — 51 precedents, `defclause`/`defsurface`/`load-file!` among them.

### 2 · `:wat::core::reduce` — the alias MOVES, it is not duplicated

`reduce` is a wat-side `defalias` for `foldl` (`wat/seq.wat`, Stone 1c-f, this session). Minting a
registry alias row **alongside** it would create two authorities for one name — precisely what the
RULING forbids. So the wat `defalias` is **deleted** in the same stone that mints the row.

`foldl` is `@Purity/@Determinism/@Totality Preserving`; the row declares NO axes and inherits, per
the `rete_alias.rs` contract. Long-term the builder intends `foldl → reduce`; this puts `reduce`
in the registry ahead of that rename.

⚠ **This part carries the stone's only real risk.** A wat `defalias` writes to `sym.functions` —
door 3 of `head_ok`'s order. Phase 3a (*resolve asks the registry*) has NOT shipped, so removing
the wat alias could leave `:wat::core::reduce` unresolvable at its call sites even though the row
exists. `rete_alias.rs:83` says a registry alias DOES dispatch (*"dispatches through the intrinsic
registry's `alias_of` field directly"*), so the path exists — but that is a claim about rete rows,
and it must be PROVEN for a `:wat::core::` name before this part ships. See STOP-2.

### 3 · the 37 rete rows — plain alias rows in `rete_alias.rs`

35 are clear today; parts 1 and 2 clear the last two. Each declares `@alias <core_name>`, no axes,
following `:wat::rete::i64::>`'s shape verbatim. Both ledgers
(`REGISTRY_MEMBERSHIP_GAP_A`/`GAP_B`) shrink by exactly the names registered — **let the ratchets
name the edit; do not pre-compute their lists.**

## THE FOUR QUESTIONS — flat YES/NO

| | Obvious? | Simple? | Honest? | Good UX? |
|---|:---:|:---:|:---:|:---:|
| **cond labelled as `if`** | YES | YES | YES | YES |
| **reduce's alias MOVES to the registry** | YES | YES | YES | YES |
| **the 37 as plain aliases, inheriting** | YES | YES | YES | YES |

- **Obvious?** `cond` IS chained `if` and the checker already says so; `reduce` IS `foldl`; a rete
  flavour IS its core verb. Each row states an identity the substrate already relies on.
- **Simple?** One row shape, already in use 37 times at the destination file.
- **Honest?** This is the load-bearing one, and it is why restriction was rejected: an inherited
  `Partial` is TRUE of the name at every call site, while a stamped `Total` would be true only
  inside a `where`. The registry says what the verb IS; the fence says where it may appear.
- **Good UX?** `(:wat::runtime::metadata-of :wat::rete::i64::+)` starts answering instead of
  failing, for 37 more names.

## Scope

**In:** a `cond` declaration row · a `reduce` alias row + deletion of the `wat/seq.wat` `defalias`
· 37 rete alias rows in `src/intrinsic/special/rete_alias.rs` · both frozen ledgers shrinking by
the registered names · whatever the compiler and the gates name.

**Out, affirmatively:** grading the 13 Unreviewed inheritances (the 270 both-axes batch — parallel,
gates nothing) · the FOURTH registry (`registry()` reading `sym.binding_metadata`; this stone
proves it is NOT a prerequisite) · Phase 3a itself · any change to `wat/rete/compile.wat`'s fence,
which this stone establishes is a separate and correct authority.
