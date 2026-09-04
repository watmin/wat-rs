# DESIGN — STONE: `:wat::holon::literal` is a special form and says it is an intrinsic

> **Builder, 2026-09-04:** ruled option **A** after the four questions — *"A has been reasoned."*
>
> Governed by `[[RULING-the-registry-is-the-sole-authority]]` item **1** (every name — and what
> KIND of thing it is). This stone does not add a row or a slot: it corrects a row that **lies
> about its own kind**, and deletes the hand-list entry that lie forced into existence.

## How this was found — a hand-list came back from the inside

The previous stone replaced `eval_apply`'s local `const SPECIAL_FORMS` with a registry query
(`entry.kind == Kind::SpecialForm`). The rider measured each of the 11 names individually and
found that gating on kind alone would **silently stop rejecting `:wat::holon::literal`** — no test
catches it. The repair was a hand-written exception.

★★★ **That is the campaign's own failure mode reappearing from the inside.** The hand-list did not
come back because the registry lacked a slot; it came back because **a row's answer is wrong.**
Every future consumer that correctly asks `Kind::SpecialForm` will need the identical exception,
and each will look like a legitimate special case in isolation.

## The evidence that the row is misdeclared — three independent sources

```
1  ITS OWN CHECK ARM      check.rs:3265-3271 — "The enclosed form is DATA captured without
                          evaluation (exactly as `:wat::core::quote`)."
2  ITS OWN DOC            "quotes `form` without evaluating it"; @arg reads
                          "the unevaluated form, alone". It shares `eval_quote` with
                          `:wat::core::quote`.
3  ITS SIBLINGS           :wat::core::quote    #[wat_special_form]   @syntax (… <expr>)
                          :wat::stream::lazy   #[wat_special_form]   @syntax (… <body>)
                          :wat::holon::literal #[wat_intrinsic]      — none —
```

Not evaluating what you are handed is this substrate's own definition of a special form —
`quasiquote`'s registered ground states it: a form *"is a special form BECAUSE some of what it is
handed never runs."*

## ⛔ AND `@syntax` IS NOT THE ISSUE — the earlier framing was wrong

An intrinsic cannot carry `@syntax`, and that is **correct by design**, not a gap. `wat-doc`'s
`DocError::MissingShape` states the rule: *"`@arg` for positional forms (grammar is derived),
`@syntax` for structural forms."* An intrinsic is positional by definition — its shape IS its arg
list. The defect is not a missing directive; it is the **kind**.

## The three moves, each with a worked precedent on an adjacent line

```
role = eval    annotate the existing `eval_holon_literal` (src/intrinsic/holon/atom.rs).
               `Kind::SpecialForm` DOES carry a handler — `intrinsic/mod.rs:418`: "Some when a
               `role = eval` implementation registered a pointer". 19 such impls exist.
role = check   extract the inline arm at `check.rs:3271` to a named fn.
               ⭐ THE ARM IMMEDIATELY BELOW IT — `:wat::core::forms` at check.rs:3282 — is
               ALREADY this exact delegation, done by Stone 1a-γ-i for six verbs.
the doc        satisfy `DocSpecialForm`. Its `@arg` prose already describes the shape.
```

⚠ **The completeness gate makes both mandatory**, not optional: `intrinsic/mod.rs:2802` — *"every
registered `Kind::SpecialForm` entry must carry at least a `check` and an `eval` impl."* A
reclassification without both goes red there, by design.

## What changes, measured

```
eval_apply         the `:wat::holon::literal` exception DELETED (`defn` stays — a stdlib macro,
                   the FOURTH-registry fork).
reflection         the sentinel head moves `:wat::core::__internal/registered` →
                   `:wat::core::__internal/special-form` (lookup.rs:418, verbs.rs:270).
                   ★ A change TOWARD truth: the verb IS a special form.
check.rs:5650      loses the `Kind::Intrinsic` arity fallback — and this costs NOTHING,
                   MEASURED: literal's own check arm enforces `args.len() != 1` and RETURNS,
                   so it never reaches that fallback. Not assumed; read at check.rs:3272-3278.
the DEBT census    `intrinsic/mod.rs:2267` splits no-scheme rows by kind — one row moves from
                   `Kind::Intrinsic, no scheme` to `Kind::SpecialForm, no scheme`. The DEBT
                   TOTAL is unchanged; only the split moves.
```

**Blast radius: 42 call sites across 10 files.** ⛔ Note the spelling trap that nearly hid it: the
corpus writes this verb as the **`#holon` reader tag**, not as `:wat::holon::literal`. A census on
the FQDN alone returns *zero test files* and is wrong. `tests/types/probe_arc294b_holon_literal.rs`
is the dedicated test.

## ★ A SECOND STALE CLAIM, found on the way

`wat/runtime-meta.wat`'s `Kind` enum documents `:SpecialForm` as *"a substrate special form — **no
NativeHandler**; dispatched by the runtime."* Measured false: 19 `role = eval` impls register
handlers on `Kind::SpecialForm` rows, and `intrinsic/mod.rs:418` says so explicitly. Correct that
comment in this stone — it is the definition every reader of the enum meets first, and this stone
is precisely about a row whose kind was chosen wrongly.

## THE FOUR QUESTIONS — as debated in-chat, both options, flat YES/NO

| | Obvious? | Simple? | Honest? | Good UX? |
|---|:---:|:---:|:---:|:---:|
| **A — reclassify** | YES | YES | YES | YES |
| B — keep `Intrinsic` + the exception | **NO** | YES | **NO** | **NO** |

B's `Simple = YES` is real — doing nothing is simpler *today*. It is overridden by the campaign's
own precedent (arc 233, quoted in `[[DESIGN-CAMPAIGN-the-registry-becomes-the-sole-authority]]`):
**Simple measures the SHIPPED state, not the transition.** B's ongoing cost is unbounded — one
exception per future kind-test, each legitimate-looking alone; A's is bounded and enumerated above.

## Scope

**In:** the reclassification and its two `role =` pointers · the `eval_apply` exception deleted ·
the `Kind` enum's stale "no NativeHandler" comment corrected · whatever the 10 files need.

**Out of scope, affirmatively:**
- **`@Purity Pure` on a verb that never evaluates its argument.** This arc minted
  `Purity::Unevaluated` for exactly that shape — but `quote` and `stream::lazy` both declare `Pure`
  too. That is a **three-row question**, and settling it by assertion inside a reclassification is
  how a behaviour change hides in a refactor. Named here; not touched.
- **`:wat::core::defn`'s exception.** It is a stdlib macro with no registration site at all; the
  FOURTH-registry fork owns it.
- **Auditing other rows for the same mis-kinding.** This stone fixes the one the substrate caught.
  A census of `#[wat_intrinsic]` rows that do not evaluate their arguments is a separate stone and
  is named as a fork, not started here.
