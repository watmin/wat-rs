# BRIEF — STONE 251.8d-i-b: `<-` is NOT always an annotation arrow

**Drawn 2026-09-21 against `main` @ `89dcbe584`** (floor 5941/5941, clippy 0, census `no STOP-8`).
Amends **8d-i** (the codemod), which is LANDED. ⛔ **This is a CODEMOD defect, not a registry gap** —
255 has nothing to look up here.

## ⭐ THE FALSE PREMISE IS WRITTEN IN THE CODEMOD'S OWN COMMENT

`wat/fix.wat:125-126`:

> *"arrow? — a bare binder/return annotation arrow SYMBOL (`<-` or `->`). NOTE: the threading macro
> head is the KEYWORD `:wat::core::->` ; **a bare `->` SYMBOL is always an annotation arrow.**"*

⛔ **It is not.** Inside a rete `:when` clause, `<-` is a **FIELD BINDING**, and the keyword after it
is a **field NAME**, not a type.

```
ORIGINAL : :when [(:vrm::F (?k <- :k))]   :then [(:vrm::Seen :k ?k)]
CONVERTED: :when [(vrm/F  (?k :- k))]     :then [(vrm/Seen  :k ?k)]
                        ↑        ↑
                   binder→ascription   field NAME → bare symbol
```

⚠ **Note `:k` SURVIVED in `:then` and was destroyed in `:when`.** The rule fires off the arrow, and
`:then` has none. That asymmetry is the proof the arrow is the trigger.

⭐ **ONE FIX CURES BOTH SYMPTOMS.** `fix.wat`'s header: *"`fix-seq` carries `prev-arrow?` so
post-arrow keywords are converted as types."* If `arrow?` declines here, `prev-arrow?` never sets and
the `:k` → `k` conversion cannot fire. **Do not write two fixes.**

## THE SCALE — measured, and far larger than the sample showed

| | |
|---|---|
| files with `(?x <- ` field bindings | **326** |
| converted files sampled | 40 |
| of those, failing `ReteCheckErrors` | **36 — 90%** |
| rete failures in the 179-file delta | 21 |

⇒ **~290 corpus files**, against 21 visible in the delta sample (which sees ~1 in 12).
⛔ **This is now the largest single class blocking 8d.**

## ⭐ THE DISCRIMINATOR — measured, and it is LOCAL

Character immediately preceding the binder name, across every `x <- ` in the corpus:

| char | n | meaning |
|---|---|---|
| `[` | 7,143 | annotation, first in a vector |
| ` ` | 4,060 | annotation, later in a vector |
| **`(`** | **3,853** | ⛔ **rete field binding** |
| `~` | 42 | annotation with an **unquoted name** in a macro body |
| `"` | 3 | inside a string literal |

And by prefix: **every `?`-prefixed binder in CODE sits inside a `(`** — 3,853 of them.
⚠ The 2 apparent `[?x <- ` cases are **COMMENTS** (`;; Tally(n) :- [?n <- (acc::count) …]`), not code.

⇒ **A `?`-prefixed left side is a rete variable, never an annotation binder.**

⚠ **The 42 `~`-prefixed cases MUST KEEP CONVERTING** — they are real annotations whose *name* is an
unquote (`[& ~call-args-sym <- (:wat::core::Vector :- […])]`, `wat/Record.wat:198`). ⛔ **A cure that
keys on "the name looks unusual" breaks these. Key on `?`, or on structure — not on strangeness.**

## The work

1. **Narrow `arrow?` (or its caller) so the annotation rules do not fire on a rete field binding.**
   ⛔ **Prefer a rule that states its own precondition** over one that enumerates rete. 8d-i's class-B
   cure is the model: *"edit a leaf only when the source text at its span equals its `ast-name`"* —
   general, and it survived a grammar nobody had looked at.
   ⚠ **`?`-prefix is local and measured; a structural rule (`(` vs `[`, or "inside a `:when` vector")
   may be truer. Choose, and say why.** If the structural one needs context the walk does not carry,
   **that is a finding** — `fix-seq` already carries `prev-arrow?`, so it carries *some* state.
2. ⛔ **Do NOT write a second fix for the `:k` → `k` symptom.** It is downstream of `prev-arrow?`.
   **If it still fires after the arrow fix, that is a finding** — say so rather than patching it.
3. **Extend the recorded replay fixture** — `wat-scripts/fixes/replay/…` — with a rete clause
   **and** a `~`-named annotation. ⭐ **The near-miss is the test**, as 255.4's fixture showed.

## The gate

- ⭐ **THE DELTA (baseline 64, orchestrator's tree) + the classification table**, one named tree,
  before and after. **`ReteCheckErrors` — 21 — should collapse.** If it does not, that is the finding.
- ⛔ **A NON-VACUITY CONTROL BOTH WAYS:** a rete `(?k <- :k)` is **left alone**, AND a real annotation
  `[x <- :wat::core::i64]` — including a `~`-named one — **still converts**. A cure that stops
  converting annotations would "fix" this class by breaking everything else.
- **Idempotence** on the converted copies; dry-run on `/tmp`, `diff`.
- `scripts/floor.sh` green; clippy `-D warnings --all-targets --workspace` **0**; census `no STOP-8`.
  Run crate clippy **and the lint suite** yourself first.
- ⛔ **NOT ONE corpus `.wat` CONVERTED.** This amends the TOOL. Fixtures are expected.
- ⚠ **`wat/fix.wat` is `include_str!`'d — REBUILD before concluding a cure does not work.**
  Three stones have lost time to this.

## Doctrine

- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Capture whole the first time.
- ⛔ **R21 — the corpus moves BY THE TOOL**, and these codemods are the builder's **merge
  infrastructure** for branches that are not merge-ready. A rule that is right only for this tree is
  worth little.
- ⛔ **REPORT WHAT A RULE CANNOT EXPRESS.** Six stones running, the best output of this campaign.
- ⛔ **If this brief contradicts the code or the runner, THEY WIN.** **Ten corrections across eight
  stones. Assume an eleventh.**
- Work only in `/home/john/work/holon/wat-rs`. **Do not push.**

## ⚠ WHY 8d-i's GREEN CENSUS MISSED THIS

8d-i gated **2145/2145 conversion** — *"the codemod can rewrite every file"*. These 326 files rewrite
**cleanly** and then **do not load**. ⛔ **That is the same blind spot that produced
`FINDING-8d-the-premise-is-FALSE`**, recurring inside the arc that found it.
**A conversion gate is not a loading gate**, and 8d-iii's acceptance must be the delta, not the census.

## Out of scope

- **The registry residue** — declaration names, `mem-store::start`, `defsurface :messages`. That is
  255's, and `:wat::core::not-a-special-form` ×3 is a **deliberate negative-test name that stays**.
- **8d-ii / 8d-iii** — still blocked on the delta.
