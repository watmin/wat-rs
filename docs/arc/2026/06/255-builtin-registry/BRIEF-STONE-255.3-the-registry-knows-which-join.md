# BRIEF — STONE 255.3: the REGISTRY decides which join — `::` or `/`

**Drawn 2026-09-21 against `main` @ `82e4cc836`** (floor 5936/5936, clippy 0, census `no STOP-8`).
Predecessors: 255.1 (identity is the pair) · 255.2 (the arrow door). Delta **104 → 97**.
⛔ **251.8d is still blocked. The dominant residue is NOT what the last two briefs assumed.**

## ⛔⛔ THE LAST BRIEF AIMED AT THE WRONG CLASS — measured, corrected here

255.2's SCORE found that gap 3a needs a **position grammar**, and the orchestrator was about to draw
this stone as exactly that. ⭐ **Then it measured the 68 `unresolved reference` regressions and the
split inverted the plan:**

| class | files |
|---|---|
| a `:wat::` builtin head | **42** |
| both classes in one file | 14 |
| a name the file declares (the position class) | **12** |

**The position grammar addresses the SMALLER class.** ⛔ **This is the ninth correction to an
orchestrator brief — predicted in the last one — and again it is an assumption that survived because
nobody split the number.**

## ⭐ THE DOMINANT DEFECT — the `Type/method` leaf does not round-trip

**222 of the unresolved paths are `Type::member`.** Traced on a real file
(`tests/macros/probe_arc209_macro_span_fidelity.wat`, original `rc=0`, converted `rc=1`):

```
original :  (:wat::core::Option/expect …)      ← a SLASH in the leaf: Type/method
converted:  (wat.core.Option/expect …)         ← the codemod's output
normalizes back to: :wat::core::Option::expect  ← a DOUBLE COLON. THE SLASH IS LOST.
```

**And the slash is load-bearing — measured:**

| head | verdict |
|---|---|
| `:wat::core::Option/expect` | ✅ **resolves** |
| `:wat::core::Option::expect` | ⛔ `UnresolvedReference` |

⛔ **`ns_to_wat_path(ns, name)` joins with `::` unconditionally.** It cannot tell
*namespace-qualified function* (`wat.core` + `map` → `:wat::core::map`) from *Type/method*
(`wat.core.Option` + `expect` → `:wat::core::Option/expect`). **The two cases are the same shape to
the joiner and different to the resolver.**

⭐⭐ **AND THE DISCRIMINATOR IS THE REGISTRY — which is this arc's whole subject.** The last segment
of the namespace is a **TYPE** (`Option`, `Failure`, `LociDiedError`). `wat.type` now has members
(255.1), so *"is the last segment a type?"* is now an **answerable question**. Before 255.1 it was
not. ⭐ **This stone is why the registry had to come first.**

⚠ **Do NOT discriminate on capitalisation.** A case rule is a naming convention, not a fact, and
this campaign has been bitten by exactly that (`wat.type/string` vs `String`). **Ask the registry.**

### The second class — a name in NAME position walked as a REFERENCE

**`:wat::WatAST` — 65 occurrences, the largest single path.** Measured:

```
[n :- wat/WatAST]    (annotation, converted)  → UnresolvedReference
[n <- :wat::WatAST]  (annotation, colon)      → OK
```

A **type name in an annotation** is being resolved as a reference once it is a Symbol. ⭐ **Same
family as 255.2's finding** — and 255.2 proved by measurement that a *predicate* cannot fix it:

| attempt | regressions |
|---|---|
| skip all top-level namespaced symbols on declare-role lists | **116** |
| …plus rewrite every `is_known_type` symbol | **108** |
| `quote_boundary` + skip `:restricted-to` values | **114** |

> *"Function-as-value (`wat.core/+` as an argument) and a declaration name (`u/x`) are the same node
> kind. **A position grammar — items[1] of the declare form only, not every leaf — is the next
> peel.**"*

**Believe that. It is measured, and three cheaper cures were tried and failed.**

## The work — LARGEST FIRST, and re-measure between

1. ⭐ **The join.** Make the reconstruction ask the registry whether the namespace's last segment is
   a type, and join with `/` when it is. ⛔ **One door** — `ns_to_wat_path` and every caller that
   reassembles a path. ⚠ Nothing else in this stone until the delta is re-run.
2. **RE-RUN THE DELTA** (baseline **97**) **and the classification table.** State both.
3. **The position grammar** for the name/annotation class — items[1] of the declare form, and the
   annotation slot. ⛔ **Not a predicate over leaves. 255.2 measured that dead end three times.**
4. **Re-run the delta again.** Report what each step bought, separately.

⚠ **If step 1 alone moves the number substantially, STOP and report** rather than continuing — the
builder sequences what comes next, and a big single-cause drop changes what 255.4 should be.

## The gate

- ⭐ **THE DELTA (baseline 97) + THE CLASSIFICATION TABLE.** ⛔ **Measure it on ONE tree.** 255.2's
  SCORE reported `97 → 101` by comparing its converted corpus against a number taken from the
  orchestrator's — two different trees (`[[feedback_diff_like_against_like]]`). **Name the tree you
  used and use the same one for before and after.**
- `scripts/floor.sh` green; clippy `-D warnings --all-targets --workspace` **0**;
  `census.sh --diff` → `no STOP-8`.
- ⛔ Run crate clippy **and the lint suite** yourself before scoring.
- Predict the test delta; confirm with `cargo nextest list`.
- ⛔ **NOT ONE `.wat` CONVERTED.** New fixtures expected; a converted corpus file is a STOP.

## Doctrine

- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Capture whole the first time; never re-run first.
- ⛔ **`wat/*.wat` is `include_str!`'d** — an on-disk edit is invisible until `cargo build --release`.
- ⛔ **REPORT WHAT A DOOR CANNOT EXPRESS.** 255.1's `Infer` answer and 255.2's three measured dead
  ends are the two most valuable things this arc has produced. **Do it again.**
- ⛔ **If this brief contradicts the code or the runner, THEY WIN.** **Nine corrections across seven
  stones**, including this brief's own predecessor aiming at the smaller class. **Assume a tenth.**
- Work only in `/home/john/work/holon/wat-rs`. **Do not push.**

## Out of scope

- **8d-ii / 8d-iii** — they wait on the delta.
- **The `:restricted-to` validation pass** — its *resolution* half is the position class; say whether
  it closed.
- **The `wat.type` NAME SET** and **capacity `:panic`** — both deferred by the builder.
