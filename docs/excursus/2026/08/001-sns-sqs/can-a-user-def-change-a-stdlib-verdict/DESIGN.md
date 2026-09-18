# DESIGN — can a user def change a stdlib verdict? (the Tier B spike)

**Drawn 2026-09-17.** The gate on Tier B, per the ruling in
`the-boot-cache-is-possible-or-it-is-not/FINDING.md`: *Tier A is built; Tier B follows; **the spike runs
before Tier B is drawn***. **NOT STRUCK, and Tier B must not be built here.**

## The question, stated so it can be answered yes or no

Tier A elides parse + expand + registration (**done**, `359446c6b`: boot 0.448 → 0.220 s, floor 2.2×).
Tier B would elide the four `ALL fns` check sweeps — **126 ms** — by running them only over *un-cached*
functions.

> **Is it true that, for every stdlib function `F`, the verdict of `check(F)` is the same whether or not
> user definitions are present?**

**If yes**, the sweeps may be restricted to user functions and Tier B is sound.
**If no**, Tier B as conceived is wrong, and the finding is the counter-example.

## The four sweeps in question

`check:retired-syntax(ALL fns)` 13.32 ms · `check:restricted-call(ALL fns)` 4.16 · `check:def-position(ALL fns)` 2.46 ·
**`check:body-infer(ALL fns)` 111.10** — the last is 88 % of the prize.

## The mechanism already located — start here, do not re-find it

`src/check.rs:817` mutates `env.defined_values` from **user top-level `def`s**, and body-infer runs at
`:844` — *after*. So a user `def` is in scope while stdlib bodies are inferred. `redef_allowed` exists.
That is the shape of an attack; whether it is a real one is the spike.

## ⭑ METHOD: ADVERSARIAL FIRST, AND MIND THE ASYMMETRY

**Try to BUILD a user program that changes a stdlib function's verdict.** A single witness settles it —
**UNSOUND**, definitively, with a reproduction.

⛔ **Failing to find one proves much less**, and must not be written as if it proved the opposite. If no
witness is found, bound the claim by reading the path: *which* state does body-infer consult that user
code can reach, and is every route from `defined_values` to a stdlib body's verdict closed? **Report
SOUND / UNSOUND / UNKNOWN, and if UNKNOWN say what would settle it.**

⭐ **The prior is NOT neutral.** Tier A asked the sibling question — *can a user macro affect stdlib
expansion?* — and answered it with a witness rather than an argument. The tidy answer ("the stdlib only
calls `:wat::` heads") was **false for this corpus**: `:repl::eval-and-loop`, `:repl::eval-form`,
`:repl::turn`. **Expect this one to be false too, and be surprised if it is not.**

## Obvious attacks to try, and say which you tried

1. **Redefinition** — a user `def` of a name a stdlib body resolves. `redef_allowed` is the switch; find
   what it permits.
2. **Type-level shadowing** — a user type or record whose name a stdlib signature mentions.
3. **A user `extend-type`** adding an impl a stdlib body's dispatch could select.
4. **Arity/among-many resolution** — a user definition that makes a previously unique resolution
   ambiguous.
5. **`defclause` / acronym / macro-adjacent registration** reachable from `defined_values`.

## What a witness must show

Not merely *"a user def is visible during stdlib inference"* — that is already known from `:817` vs
`:844`. A witness must show **a stdlib function whose check VERDICT differs** (passes without the user
def, fails with it — or the reverse). Anything less is evidence of reach, not of consequence, and the
SCORE must say which it has.

## Trap-doors

1. ⛔ **Do not build Tier B**, even if the answer is SOUND. That is the next stone.
2. ⛔ **Do not "fix" anything you find.** A witness is a finding; changing the checker to make Tier B
   possible is a separate ruling with its own blast radius.
3. **A green floor proves nothing here** — this is a question about a path, not about today's corpus.
4. **`redef_allowed` may be off by default.** If it is, say whether the answer changes when it is on —
   a soundness argument that holds only in the default configuration must say so.
5. **Boot is now cached.** Use `WAT_BOOT_CACHE=off` when the question concerns derivation order, or the
   experiment measures the cache instead of the checker.

## Out of scope

Tier B itself · the two sharpening follow-ons · the queue promotion (now unblocked by Tier A and waiting
on its own re-weigh).
