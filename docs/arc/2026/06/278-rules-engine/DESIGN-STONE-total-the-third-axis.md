# DESIGN-STONE — `total?`, the third fence axis

> **Status: DESIGNED, blocked on `BRIEF-the-fence-names-the-head.md`.** Builder-ruled 2026-08-02:
> *"draw the fence naming strike, then total? behind it."*

## The hole, measured

A rete `where` predicate must be pure ∧ deterministic. Both hold for verbs that are **partial** — defined
on some inputs and undefined on others — so the fence admits them:

| verb | in `purity.rs`'s pure∧det list? |
|---|---|
| `first` / `second` / `third` | **yes — admitted** |
| `i64::/` `i64::mod` `i64::rem` | **yes — admitted** |

`first`-on-empty is the dangerous shape: it compiles, fires correctly for as long as no rule meets an
empty vector, and then **one empty vector kills the entire fire** — a raising predicate aborts the whole
fire on both engines (measured; recorded in `SEAM-2026-08-01`, a dated seam since pruned — the breadcrumb is now the single `SEAM.md`, and this measurement's home is this stone). No amount of green testing surfaces it.

## Not a fourth purity flavour — a third axis

Arc 299's thesis is that `Impure` **fused effect and entropy**, and 299.3 splits it to
`Pure | Effectful | Entropic`. That trio is the three inhabited cells of purity.rs's existing 2×2
(pure × deterministic). **Totality is not a cell in that grid** — `first` is `Pure` by every measure the
trio can take. The grid asks *what does this touch?*; totality asks *is this defined on all its inputs?*

There are **two failure sources**, and the builder's own contribution to 299 (welding entropic to
*cannot-world-fault*) is what makes the split exact:

| | question | fails how | remedy |
|---|---|---|---|
| effect | touches the world? | **world-fault** | an outcome enum |
| entropy | same input → same output? | cannot fault | — |
| **domain** | **defined on all inputs?** | **domain-fault** | **an `:else`** |
| termination | does it halt? | doesn't return | — (see below) |

**And "total" was already a fusion of two of these.** The `total?` designed a month ago
(`NOTE-overlay-read-path` Part 5) was about **recursion/termination** — the seam records that it *would
not have caught* `first`-on-empty, because `first` halts fine and is simply undefined on an input. This
stone is the **domain** axis. Termination is a fourth, separate, and not in scope.

## Why no redesign is needed first

`purity.rs` is **already a record of named axes** — `OpMeta { pure, deterministic }`, one shared walk,
independent predicates. It has never been a flattened flavour. `total` is a third named field.

The flattening question belongs to `types.rs::Purity` (the 2→3-variant enum 299.3 widens, guarding **277**
`defenum` wire-purity markers). **Different population, different question, not a blocker here.**

And the hand-managed map is *already* the declared interim: its own doc says it is *"the explicit v1
projection of the queryable registry that arc 255 will eventually own… when 255 lands, delete this map."*
Adding an axis to it is using the scaffolding as designed, not erecting new scaffolding.

Measured cost: **7 `OpMeta` construction sites** (`purity.rs:101 :105 :113 :298 :317 :327 :347`) — the 110
verbs live in a single `matches!` arm at `:116` feeding one of them.

## The strike

1. `Axis::Total` beside `Pure` / `Deterministic`; `OpMeta.total`; `is_total_expr`;
   `eval_total_predicate` — each a mirror of the two that exist.
2. Register `:wat::rete::total?` beside its siblings (`check.rs:19227-19245`).
3. Split the `matches!` list: the partial verbs (`first`, `second`, `third`, `nth`, `i64::/`, `mod`,
   `rem`, `quot`, `Option/expect`, `Result/expect`) leave the total group.
4. A third conjunct at the fence (`rete.wat:563`), and — because of the strike ahead of this one — a
   message that **names the offending head**.

**Default-deny is the whole method.** Do NOT mass-assert `total: true` across the vetted block: those 110
were vetted for a *different* property, and carrying the claim over is the hand-audit stem the file's own
doc condemns. Everything unproven; run the gate; **the corpus enumerates itself**; classify only what a
live row demands. (Builder, on the namespacing wall: *"they self identify on enforcement"* — and there it
out-enumerated a grep that had been wrong four times.)

## ⛔ Enumerate first, arm last

A refused `first` with nowhere to go locks a user out of arithmetic. The builder's ruled remedy is
rete-namespaced **total variants with a mandatory fallback** — `(rete.i64// n 0 :else -1)`,
`(rete/first n :else …)` — with the partial forms disallowed in a `where`.

So the order is: **enumerate on a branch → mint the `:else` variants → migrate → arm.** Do not ship the
refusal before the destination exists.

## What this buys beyond the hole

R62 records that the corpus's **absolute** column — STOP-1 rejections, the half a peer cannot bound — is
**empty**: 98 rows, zero. Every green row says *we agree with Clara*; only a refusal is a fact about our
substrate alone. This axis is the first thing that would put entries in that column, filled by the
substrate's own answers rather than by hand.

## Owed before it lands

- The fence-names-the-head strike (`BRIEF-the-fence-names-the-head.md`) — hard prerequisite.
- **intueri on the `:else` variant names** — the builder ruled the shape, explicitly not the names.
- The mint list: which verbs get an `:else` sibling. **Allow-list, not deny-list.** Some totals already
  exist (`PersistentVector/get` + `match`, `foldl` with a seed).

---

# ✦ STATUS UPDATE — 2026-08-02, `a787cd25`

*The body above is the original stone, kept unedited. Four things changed under it today.*

## 1. The prerequisite is PAID

`BRIEF-the-fence-names-the-head` landed (`a787cd25`). The fence now names the offending head and the
axis; `find_axis_violation` / `eval_axis_violation` expose the violating leaf the walk always held.
Step 4 of "The strike" above therefore costs nothing extra: the message machinery already exists and
a third axis inherits it.

## 2. The axis is an ENUM, not a keyword — and that is where the payoff sits

`(:wat::core::defenum :wat::rete::Axis :Pure :Deterministic)` now exists (`wat/rete.wat:556`),
builder-ruled under the CLOSED-SET RULE (*"a closed set is an enum... verbose exhaustive match — that's
our form"*). `axis_from_keyword` / `axis_keyword` were deleted, not made careful.

**Minting `:Total` therefore BREAKS `axis-violation-message`'s exhaustive match, by design.** That is
the mechanism working — the checker enumerating its own consumers — and it is verified, not assumed:
`check.rs:5700` really does reject a non-exhaustive enum match.

> ### ⛔ STOP — the launder that would un-arm this
>
> `check.rs:5700`'s message ends *"(or include `_` wildcard)"*. The `_`-arm-on-an-enum ban is
> **doctrine** (`109/NOTE-full-enum-match-mandatory-no-wildcard-arm.md`) whose **checker rule is still
> deferred and unbuilt**. So when `:Total` breaks the match, adding `_` is a one-keystroke way to make
> the break disappear — and it silently un-arms the exact mechanism this design rests on.
>
> **Name every variant. A `_` arm here is a rejected strike, not a shortcut.** Three
> designed-but-deferred walls were walked into in one session earlier in this arc; this is the fourth
> waiting to happen.

## 3. The fallback keyword is RULED: `:undefined`

Builder-ruled after an intueri cast. Not `:else` — that is `cond`'s word for a *branch* chosen when
tests fail, whereas this substitutes a *value* when an input falls outside a function's domain. Two
concepts, and the ward found they aren't even enforced by the same mechanism (`cond`'s mandatory-ness
is a macro scanning a variadic clause list, `wat/core.wat:1240-1246`; a required kwarg's is ordinary
arity checking, `wat/core.wat:632-633`). `:or` was specifically rejected — `:wat::core::or` is the
boolean intrinsic in this same purity table, so `:or -1` beside `(or a b)` in one `where` invites a
real misread.

**Mandatory-ness comes free.** `kwargs-lower` already raises `"kwargs-lower: missing argument :<field>"`
at expansion for a required field with no default. Constraint "the fallback can never be omitted" needs
no new machinery — declare it required.

### The call shape — RULED 2026-08-02: positional operands, `:undefined` the only kwarg

```clojure
(:wat::rete::i64::+ a b       :undefined -1)
(:wat::rete::i64::/ n d       :undefined -1)
(:wat::rete::first  xs        :undefined 0)
```

The intueri cast argued **full** kwargs (`:numerator`/`:denominator`) from `/`'s non-commutativity, and
the orchestrator relayed it. The builder killed it with one form:

> *"`(+ :1 0 :2 2 :3 massive-int :undefined -1)` …. wtf is a kwarg for `+`?"*

**`i64::+` is variadic and commutative.** There is no name for the third addend, nothing for keywords to
disambiguate, and no spelling that isn't `:1 :2 :3`. The cast reasoned from `/` — binary, ordered — and
generalized to all eight without writing the form out for the most common member of its own worklist
(`i64::+`, 11 refused rows). Materializing the form is what catches this, and it wasn't done.

**The principle, stated so it generalizes:** positional confusion is a defect when there is **no
established order convention** — `assertion-failed!`'s two bare `:None`s (same type, no convention,
genuinely unreadable) is the case the kwargs doctrine was ruled for. Division has a universally-known
operand order; naming it buys nothing and costs ceremony on a form written constantly. Operands keep
core's order, which callers already know; only the fallback — which has no positional convention and
must be impossible to omit — is keyword-marked.

### ✂ CUT: the "query DSL" — a parallel rete vocabulary. Considered, rejected, recorded so it is not re-derived.

After T1 measured 8 partial verbs (not 4), the orchestrator proposed a full parallel op vocabulary —
every op usable in a `where` getting a `:wat::rete::` name — arguing that under partial coverage *"the
user must know which core ops are secretly partial to know which need the rete form."*

**That argument is dead, and what killed it is the stone landed one commit earlier.** The fence now
NAMES the offending head (`a787cd25`): `where expr is not total — ':wat::core::i64::+' is not total`.
The user never needs to know in advance; the checker teaches at the point of failure, which is R29
`RVINA ERVDIT` working exactly as designed. The orchestrator made the argument having just built the
thing that refutes it.

Re-run with the diagnostic accounted for, the smaller design wins outright: **Simple** (one conjunct on
an existing fence vs a second namespace kept in agreement with core), **Honest** (no second
hand-maintained list, so no drift class needing a derive mechanism to engineer around), **Obvious**
(*a `where` admits only total ops, and the checker names any that aren't*). Builder: *"the 'dsl' here
(poorly used) is imposing a stricter purity check on where clauses in rete."*

Eight siblings adjacent to core. Not forty. Do not re-open this.

> ### ⟲ REVERSED, same day — and the reversal is the record
>
> It WAS re-opened, hours later, and the wider design is now ratified:
> **`DESIGN-STONE-where-admits-only-rete-ops.md`** (a full rete expression language, ~40–60 names).
> This section stays because the reversal is worth more than a tidy file.
>
> **What changed is the JUSTIFICATION, not the scope.** The cut above was correct against the argument
> then on the table — *"users must know which core ops are secretly partial"* — which the
> fence-names-the-head stone dissolved an hour later. The reopened case rests on two different legs
> the orchestrator never made:
>
> 1. **Compilability** (the builder's) — a closed head-space is what turns `where`-compilation from
>    *"compile a large fraction of wat"* into a finite jump table. `compiled_cond.rs` and
>    `compiled_rhs.rs` exist; `compiled_where.rs` does not, and the open op set is why. Clara admits
>    arbitrary Clojure in `:test` and therefore cannot compile it at all — the reduction is the weapon.
> 2. **The corpus cannot size this** — `[[feedback_optimize_for_the_expressivity_surface_not_the_corpus]]`:
>    *the corpus is a record of what happened to COMPILE, so it is structurally blind to the fence.*
>    T1's 8 verbs are a FLOOR, not a target. Sizing the vocabulary to 98 rows written under the old
>    rules is designing to survivorship — which is what "eight, not forty" did.
>
> Kept visible rather than deleted: an argument can be right, be correctly cut, and later be right
> again for a reason nobody had yet. Deleting this would hide that the *premise* moved, not the answer.

## 4. The strike is SPLIT — this stone's first rider does NOT arm the fence

"Enumerate first, arm last" is the whole discipline, so the work is at least two strikes:

| strike | scope | ships |
|---|---|---|
| **T1 — the axis, unarmed** | `Axis::Total`, `OpMeta.total`, `is_total_expr`, `eval_total_predicate`, register `:wat::rete::total?`, the arm that unbreaks the exhaustive match. **The fence does NOT consult it.** Then MEASURE: run `total?` over every `where` expr in the 98-row corpus and report exactly which verbs a live row demands. | the axis + **the worklist** |
| **T2 — the mint** | the `:undefined` variants for whatever T1's enumeration actually named (allow-list, not deny-list), operand-spelling ruled first | the destination |
| **T3 — migrate, then ARM** | corpus onto the total variants; only then the third conjunct at the fence | the wall |

**T1 is the deliverable that makes T2 honest.** Do not guess the mint list — the corpus names it. And
`total?` being callable-but-unconsulted means T1 cannot move the accepted-`where` set by one row, which
is its own STOP.

**Default-deny is not softened by the split.** Everything is `total: false` until a live corpus row
demands otherwise. Do NOT mass-assert `total: true` over the 110 verbs in the `matches!` — they were
vetted for a *different* property, and carrying that claim across is exactly the hand-audit stem the
file's own doc condemns. The enumeration exists so the classification is demanded rather than assumed.
