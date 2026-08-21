# DESIGN STONE — 255.1c-kernel-error · HOME #6: a HOME and a CATEGORY are DIFFERENT AXES

## The four verbs

```
runtime.rs:6754  LociDiedError/message     → eval_died_error_message      body 27716
runtime.rs:6763  Failure/message           → eval_failure_message         body 27423
runtime.rs:6764  Failure/location          → eval_failure_location        body 27452
runtime.rs:6765  LociDiedError/to-failure  → eval_died_error_to_failure   body 27812
```

## ⛔ FIRST — this stone REFINES the principle home #4 established, and the refinement matters

Home #4 ruled: **the carve boundary is the CATEGORY, not the decomposition table's row** — because
the table filed `stopped?` under *misc* while `:Ambient`'s prose named it. Home #5 reused it.

That principle was about **which verbs belong together**: a mis-drawn table row must not split a
category. **It was never "a module must be single-category,"** and reading it that way here would be
the mistake it exists to prevent — asserting a law from three incidental observations (homes #3/#4/#5
each happened to be one category).

**A HOME is a code-organization unit. A CATEGORY is a per-row semantic label.** Conflating them is
the axis-mix this taxonomy keeps refusing. This home holds four verbs that are one *subject* — the
surface of the two error types — carrying **two categories**:

| verb | `@Category` | why |
|---|---|---|
| `LociDiedError/message` | **Projection** | returns a component that was already there |
| `Failure/message` | **Projection** | ditto — one hop deeper, see below |
| `Failure/location` | **Projection** | ditto |
| `LociDiedError/to-failure` | **Transform** | matches the variant and CONSTRUCTS a Failure |

`:Projection`'s prose names the first three outright. It does **not** name `to-failure`, and the body
says why: `eval_died_error_to_failure` (`runtime.rs:27812`) matches `ev.variant_name` (`"Panic"` →
extract the message) and builds a `Failure` — a **different-kind** value. That is neither `:Projection`
(a part that already existed) nor `:Combine` (a larger value of the *same* kind). It is a
representation transform.

**So the table's `errors` row is TWO categories.** Third table row proven wrong in three stones —
which is the finding, and it is why the row-vs-category distinction had to be stated before this
carve rather than after it.

## ★ The gate is LIVE again — unlike home #5

All four have registered `TypeScheme`s (`check.rs:18101, 18121, 18130, 18147`), so
`doc_arg_ret_types_match_checker_scheme` **will** check every `@arg`/`@ret` here. Home #5's five were
skipped by `None => continue`; these are not. Measured shapes the doc must match exactly:

```
LociDiedError/message     params [:wat::kernel::LociDiedError]  ret :wat::core::String
Failure/message           params [:wat::core::Record]           ret :wat::core::String
Failure/location          params [:wat::core::Record]           ret Option<:wat::kernel::Location>
LociDiedError/to-failure  params [:wat::kernel::LociDiedError]  ret :wat::kernel::Failure
```

⚠ **`Failure/*` take `:wat::core::Record`, not a `Failure` path.** A rider writing the obvious
`@arg failure :wat::kernel::Failure` turns the gate red. That is the gate doing its job, and it is
the reason this home is worth carving right after one where nothing could be checked.

## ★ The `Failure/*` projection is one hop deeper — and the code says so

From the dispatch comment (`runtime.rs:6757`):

> *"Arc 278 the string-wrap annihilation — `Failure/message` / `Failure/location` are DERIVED
> accessors (the stored fields were REMOVED; Failure carries the raised `:wat::core::Error`
> structurally). They read `error.message` / `error.location` off the mandatory `error` field."*

They project `failure.error.message`, not `failure.message`. Still `:Projection` — a part that already
existed — but through a hop, and the rider derives that from the body rather than the name.

## The one contract decision, pinned

**Each row's `@Category` is derived from its own body, and the home does not homogenize them.** If a
rider's reading makes all four one category, that is a finding to report — not a tidiness to apply.

## The taxonomy's own residue, now in front of real rows

`255.1c-taxonomy` recorded: *"Whether DERIVED record accessors should carry `:Projection`. They have no
`Category` today by design (`accessor_meta` derives from the frozen `TypeEnv`). Widening the registry
to cover derived rows is a registry-shape question, not a taxonomy one."*

These four are the **hand-written** accessors, so they are in scope and the residue is not. But this
home puts three `:Projection` rows on disk beside a generated-accessor population that has none — which
is the concrete form of that open question. **Named, not answered.**

## Blast radius

```
NEW   src/intrinsic/kernel_error.rs
EDIT  src/intrinsic/mod.rs   one `mod kernel_error;` line
EDIT  src/runtime.rs         delete 4 literal arms (+ replacement comment); widen 4 delegates
```

No `check.rs`. No `wat/runtime-meta.wat`. No new types.

## ⚠ STANDING ORCHESTRATOR STEP — the goldens, every carve from here

Five `.edn` diagnostics fixtures pin an exact `src/runtime.rs` line. **Every kernel carve shifts it,
so every carve breaks them** — twice already today (homes #4 and #5), and it will fire on every
remaining home. This is now a scheduled step, not a discovery:

1. after the strike, `git diff --numstat src/runtime.rs` → net delta **D**
2. confirm every structural hunk precedes the pinned site, and that the 1-for-1 `fn`→`pub(crate)`
   hunks have zero delta with new positions exactly old − D
3. confirm `:col` is unchanged and **only** `:line` moved
4. bump the five, and confirm the diff is exactly five `:line` lines and nothing else

**The rider cannot see this** — a scoped `test(/intrinsic::tests::/)` filter does not reach
`tests/diagnostics/`, which is why both prior riders reported green against a red floor. The brief
tells the rider its filter is blind here so it stops claiming a green it cannot observe.

The deeper question — whether a golden should pin a line number inside a file this arc is actively
dismantling — is **open and the builder's**; this step is the conservative half, taken so the cost is
scheduled rather than rediscovered.

## Progress meter

69 → 73 registered production names. Four arms leave `runtime.rs`, and the registry gains its first
home whose rows do not share a category.

---

## ⊘ RENAMED MID-STRIKE 2026-08-19 — `:Project` → `:Projection`, on the builder's call

The variant shipped as `:Project` in `255.1c-taxonomy` and was renamed before this home's rows ever
reached a commit. Builder: *"is it Project or Projection?.... hrm...."* → *"Projection it is"*.

**The deciding argument is ambiguity in THIS repo's idiom.** Our whole vocabulary is arcs, stones and
projects; a `Category` column reading `Project` invites a double-take on every read. `Projection` has
no such collision and is the exact word — taking a component out of a product IS a projection. It
also sits naturally beside `Reflection`, its closest sibling in kind.

**It does NOT re-trip the rejection that killed `:Accessor`.** That was refused as an AGENT noun —
the thing that does it rather than the doing. `Projection` is the act; the agent noun would be
`Projector`. Checked deliberately, because "a rejected option returns in new clothes" is a mistake
made earlier the same day. `[[feedback_a_rejected_option_returns_in_new_clothes]]`

**The counter, recorded rather than buried:** it breaks the bare-stem quartet `Transform` / `Probe` /
`Combine` / `Project`, and `:Projection`'s own prose calls it *"the inverse of `:Combine`"*. Measured
before deciding: **`Combine` has ZERO tenants** (as do `Probe`, `Declaration`, `Binding`), so the
pairing was a prose claim, not a shipped constraint. If the family matters later the coherent end
state is `Projection`/`Combination`, and aligning `Combine` costs nothing while nothing carries it.

**Cost of doing it now vs later:** three rows, uncommitted, plus the four mirrors — the same sites the
`:Clock`→`:Entropic` rename touched hours earlier. Renaming after the rows shipped would have cost
strictly more, which is why it was folded into this stone rather than filed.

Done by the orchestrator by hand, not delegated: a six-site rename layered on a released rider's
uncommitted work is more coordination risk than the calibration is worth. Recorded because the
default is to delegate.
