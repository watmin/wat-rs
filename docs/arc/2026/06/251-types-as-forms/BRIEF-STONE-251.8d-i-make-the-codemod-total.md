# BRIEF — STONE 251.8d-i: make the codemod TOTAL (zero corpus writes)

**Drawn 2026-09-20 against `main` @ `817c85213`** (floor 5924/5924, clippy 0, census `no STOP-8`).
Parent: `BRIEF-STONE-251.8d-the-corpus-flip.md`. Design: `DESIGN-STONE-251.8-symbol-proper.md` §251.8d.

## ⛔ THE CONSTRAINT THAT SHAPES THIS STONE — the builder's, verbatim

> *"make sure we have codemods for all of this as we have many more large scale branch merges to
> contend with -- none of them are merge ready and we'll need to use these tools to fix them once we
> begin their merges"*

⭐ **The codemods ARE the deliverable, as much as the converted corpus.** Every divergent branch we
merge from here will be in the old dialect and will carry these same classes.

⛔ **THEREFORE: NOT ONE FILE MAY BE FIXED BY HAND.** The parent brief allowed *"relocating or runing
them is a legitimate answer"* for class C's 20 files. **That permission is REVOKED.** A hand-fixed
file helps this tree once and helps a future merge never. If a class cannot be toolable, that is a
**STOP and a report**, not a hand edit.

## What this stone does, and does NOT

| | |
|---|---|
| ✅ cure the codemod's three failure classes | A (proven), B (cure derived below), C (needs a new rule) |
| ✅ land class C's **parser** half — `:restricted-to` accepts symbols | purely additive; nothing spells them yet |
| ✅ new `.wat` **fixtures** for the new spellings | these are new files, not conversions |
| ⛔ **write ZERO tracked `.wat` files as CONVERSIONS** | the corpus does not move in this stone |
| ⛔ start the flip | that is 8d-iii |
| ⛔ touch `wat/` as a bootstrap | that is 8d-ii |

**The gate is the census going to 0 failures — on COPIES.**

---

## CLASS A — `ast-name` partiality. ✅ CURE PROVEN, just land it.

`wat/fix.wat:74`, `annotated-if?` calls `(ast-name head)` with **no kind guard**, two lines above
where it correctly guards `c2` with `(= (ast-kind c2) "symbol")`.

**Mechanism, confirmed with a negative control:** `((wat.core/fn [] 1) 2 3)` — a list in head
position with ≥3 children — raises; the same shape with **2** children passes, because
`(< (length ch) 3)` short-circuits first. ⭐ That short-circuit is why the class hits ~7.5% of a
skewed sample and looked mysterious for an entire arc.

**Measured result of the cure:** guard `head`'s kind as `c2`'s already is → corpus census class A
goes **2/80 → 0/2140**. `wat.core/if` is lazy (verified: an untaken `(:wat::i64::/ 1 0)` never ran),
so a kind guard genuinely prevents the call.

⚠ The orchestrator's test patch was written to *measure* the class, not to be the shape. **Derive the
final form yourself.**

## CLASS B — reader-synthesized spans. ⭐ THE CURE IS GENERAL, NOT A `~` SPECIAL-CASE.

**Measured** (`wat-scripts/scratch-pad/8d-probe-reader-macro-nodes.wat`, committed `817c85213`):

```
src=~x      | top=list | head-kind=keyword | head-name=:wat::core::unquote
src=(a ~b)  | top=list | head-kind=symbol  | head-name=a
```

`~x` reads as a **list** whose head is a keyword named `:wat::core::unquote` — **19 chars** — while
its source span covers `~`, **1 char**. `head-keyword?` sees the `::`, calls it a call head, takes
`old-text` from the NAME, and `fix-text-apply` refuses: *"the rule's belief and the source disagree;
refusing to splice."*

⛔ **THE GUARD IS CORRECT. DO NOT WEAKEN IT.** `fix.wat:225-227` documents this hazard in its own
words *before it ever fired* — a synthesized node's span and its canonical name disagree, and filling
`old-text` from the span would make the check *"compare a slice against itself — vacuous, catching
nothing."* A cure that widens the tolerance, silences the refusal, or fills `old-text` from the span
is the defect the guard exists to catch.

⭐ **THE CURE — one rule, no `~` anywhere in it:**

> **Edit a leaf only when the source text at its span EQUALS its `ast-name`.**
> Where they disagree the node is reader-synthesized: **skip it, edit nothing.**

Skipping is *correct* here, not a dodge: `~` is already faithful Clojure and needs no conversion.
This also covers `~@`, `` ` ``, `'` and anything else the reader synthesizes, **without enumerating
them** — which is what makes it survive a future branch that uses a reader macro we have not seen.

⚠ **A note so you do not think this door is forbidden:** `fix-text-span-text`'s doc comment warns
against filling `old-text` from the span. **This is the opposite use** — comparing span text against
the name as a *guard*, then declining. Asking the same question `fix-text-apply` asks, and skipping
instead of dying. Say in the SCORE which door you used and why it is not the forbidden one.

**Size:** 99 files, concentrated where the syntax lives — `tests/macros` 38, `wat-scripts/scratch-pad`
20, `wat/holon` 11, `wat` 8. ⛔ **8 are in `wat/`**, so they are also 8d-ii's problem.

## CLASS C — namespace-prefix markers. ⭐ THE BUILDER HAS RULED; IT IS A CONVERSION, NOT AN EXCLUSION.

`fix.wat:281` hands `:my::issuer::` to `keyword::to-symbol`, which refuses: *"not a convertible
call-head/reference keyword (bare data keyword or namespace-prefix marker)."* The refusal is right —
the marker is **data**, not a call head.

**The builder's ruling:**

> *"yes, `{:restricted-to [my.kernel]}` is the ruling"*

⛔ **So class C is NOT "teach `head-keyword?` to skip markers."** Skipping leaves them as `::`
keywords and `::` never retires — which is 8d's entire point. **They must CONVERT.**

| today | becomes | why |
|---|---|---|
| `:my::kernel::` | `my.kernel` | a **symbol**, no `/` ⇒ a namespace |
| `:my::kernel::specific-caller` | `my.kernel/specific-caller` | has `/` ⇒ an exact FQDN |

⭐ **The trailing-`::` marker dies and the first-slash ruling becomes the discriminator.**

### Measured, so you can size the rule

- **32 markers, 12 distinct spellings.** Top: `:my::issuer::` ×7, `:my::kernel::` ×5, `:my::` ×5,
  `:wat::test::` ×4, `:wat::kernel::` ×3.
- ⭐ **EVERY trailing-`::` keyword in the tracked corpus sits inside a `:restricted-to` vector.**
  Verified by sweeping all `.wat` for a trailing-`::` keyword and subtracting the `:restricted-to`
  ones — **the remainder is empty.** ⇒ **the codemod rule can be CONTEXT-FREE**: a keyword whose
  name ends in `::` converts, full stop. ⚠ Re-derive this before relying on it; if a counter-example
  exists the rule must become context-sensitive and you must say so.
- **Exact-FQDN whitelisting EXISTS and is DELIBERATE.** `caller_matches_prefix_list`
  (`src/check.rs:1498`): `entry.ends_with("::")` → `starts_with`, else `==`. It has exactly **one**
  corpus use — `:my::kernel::specific-caller` — and that use is **its own paired fixtures**:
  `wat_arc198_def_restricted_ok_exact_fqdn_allowed.wat` and `..._bad_exact_fqdn_denied.wat`
  (*"a sibling, not the exact name"*). A positive and a negative control, arc 198.

### The parser half — lands HERE, and it must

`src/check.rs:1373` builds the whitelist with a `filter_map` to `WatAST::Keyword` and **silently
drops everything else**:

```rust
.filter_map(|n| if let WatAST::Keyword(k, _) = n { Some(k.clone()) } else { None })
```

⛔ **After the flip every entry is a `Symbol`, so every whitelist in the tree would silently become
`[]`.** It fails **closed** (an empty list denies all callers — verified: the caller is refused with
`whitelist []`), so it reds loudly rather than admitting quietly. But it would red ~20 files for a
reason that looks nothing like the cause.

**Required:**
1. Accept `Symbol` entries alongside `Keyword`. Additive — nothing spells them yet, so this is safe
   to land before the corpus moves.
2. ⛔ **Hard-error on an entry that is neither.** A silent drop is how this hazard was built.
3. Discriminate on **`/`**, not on a trailing `::` — `caller_matches_prefix_list`'s `ends_with("::")`
   becomes "contains no `/`".
4. **Unify the two paths.** `types/defstruct.rs:103` *errors* on a non-keyword entry while the `defn`
   path *drops* it. One surface, two behaviours, today.

⚠ **The sibling case is the one that matters.** `my.kernel/specific-caller` must still DENY
`my.kernel/other-caller`, while bare `my.kernel` admits both. The two arc-198 fixtures already pin
it — **convert them and they become the codemod's own acceptance test**, which is better evidence
than a fixture written for this stone.

---

## THE CODEMODS ARE THE DELIVERABLE

Every cure lands as a **recorded, re-runnable** artifact under `wat-scripts/fixes/` (or inside
`wat/fix.wat` where it is a rule of the existing engine), per R21 and the builder's merge constraint.

⛔ **Each must be runnable against an ARBITRARY branch's tree**, not just this one. A rule that
hard-codes a path, a file list, or one of our namespaces is not merge infrastructure.

**Prove it:** the SCORE names, for each of A/B/C, the artifact and the command that runs it.

## The gate

- ⭐ **THE CENSUS GOES TO ZERO.** Re-run the corpus-wide class census — all 2,140 `::`-head files, on
  **copies**, one process per file — and it reports **0 A, 0 B, 0 C, 0 other**. State the table.
  ⚠ Budget: ~2.7 s/file single-invoked ⇒ **~96 min**. Time it on one item first
  (`[[feedback_time_a_tool_on_one_item_first]]` — the orchestrator relearned this launching it blind).
- ⛔ **ZERO tracked `.wat` files modified as conversions.** `git status` proves it. New fixtures for
  the parser change are fine and expected; a converted corpus file is a STOP.
- **Idempotence:** dry-run twice on the copies; the second pass changes 0 files.
- `scripts/floor.sh` green; clippy `-D warnings --all-targets --workspace` **0**.
  ⛔ **Run `cargo clippy --release --all-targets -p wat -- -D warnings` YOURSELF before scoring.**
  218.7 landed a clippy error because the brief assigned clippy to the orchestrator; 218.8 then had
  a clippy fix trip a *different* wall (`one-variant-separator`). **Two walls can disagree about one
  line — expect it, and do not trade one red for another.**
- Predict the test-count delta from the diff, confirm with `cargo nextest list`.
- `scripts/replay/census.sh --diff` → `no STOP-8`.

## Doctrine — `wat-rs/CLAUDE.md` does not reach a subagent

- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Capture whole the first time; never re-run first.
- ⛔ **R21 — the corpus moves BY THE TOOL, never by hand.** Reinforced by the builder's merge
  constraint above. An unreachable site is a STOP and a report.
- ⛔ **DRY-RUN ON COPIES AND `diff`.** Every measurement behind this brief was taken on copies under
  `$CLAUDE_JOB_DIR/tmp`; the tree was never the experiment.
- ⛔ **`wat/fix.wat` is `include_str!`'d** into the binary (`src/load/stdlib.rs:353`). **Editing it
  changes nothing until `cargo build --release`.** The orchestrator's first class-A cure "failed"
  for exactly this reason. **If a cure appears not to work, rebuild before theorising.**
- ⛔ **ASK THE TOOL THAT OWNS THE FACT.** Test counts → `cargo nextest list`. Three counts in these
  documents were contaminated by comments and string literals before being caught.
- ⛔ **If this brief contradicts the code or the runner, THEY WIN.** Land what is true and report the
  brief's error. The counterpart has corrected this orchestrator's instructions twice in two stones
  (`a:/b`; the 218.7 REPL cross-reference) — both times correctly.
- Work only in `/home/john/work/holon/wat-rs`. No `git filter-branch`. **Do not push.**

## Out of scope — affirmatively cut

- **The corpus flip** (8d-iii) and the **`wat/` bootstrap** (8d-ii).
- **Duplicate map keys in `.wat` source** — the builder ruled it an error, but it is `wat-reader`'s
  and its blast is unmeasured. Filed: `218-wat-edn-impeccable/FINDING-duplicate-keys-are-legal-in-wat-source.md`.
- **The `T/0` positional accessors** (3 files) — Clojure refuses a name beginning with a digit. A
  ruling, not a tool. Not here.
- **`wat-edn`.** 218.7/218.8 made it a spec-correct EDN reader. It does not read `.wat` source.
