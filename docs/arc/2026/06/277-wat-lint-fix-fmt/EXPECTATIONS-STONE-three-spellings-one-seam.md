# EXPECTATIONS — STONE: three head spellings, one seam

| # | what | expected — EXACT |
|---|---|---|
| 1 | it builds | clean |
| 2 | ★★★ **all three spellings get the SAME ruled shape** | one fixture per spelling, same form. `let`'s binders paired and aligned in **all three**; `defn`'s args **one per line with `<-` aligned** in all three |
| 3 | ★★★ **and the shapes are identical modulo the head token** | strip the head spelling from each output → the three are **byte-identical**. "All three look right" is satisfiable by three different rights. |
| 4 | ★★★ **a TWO-argument `defn` is the discriminator** | a one-arg `defn` renders the same under the ruled rule and the generic fallthrough — it **cannot** prove this stone. Every spelling fixture carries **≥2 args and ≥2 let-binders**. |
| 5 | ★★★ **the canonical form is the CLOJURE TARGET** | rules compare against `wat.core/…`, **not** `:wat::core::…`. **This is the whole builder requirement**: canonicalising to today's spelling means paying for the migration twice. |
| 6 | ★★★ **DROPPING A FLAVOR IS ONE DELETION** | delete the FQDN arm of the canonicaliser → **only** FQDN fixtures change; the clojure fixture is untouched and **no rule file is edited**. **Demonstrate it, restore it.** This is the acceptance, not a claim. |
| 7 | ★★★ **the three-spelling knowledge does not leak into the rules** | `grep -c ':wat::core::'` over `wat-scripts/fmt/rules/*.wat` = **0** head literals; every comparison is against the canonical form |
| 8 | ★★★ **the canonicaliser is ONE function** | one home, named in the SCORE. **Not a helper per rule file, not a second name parser** — `one_name_grammar` went red on this arc's `:then` stone for exactly that |
| 9 | ★★ **`wat/deporder.wat` et al still format identically** | `deporder c69f0460e9f91672` · `spawn 1c4bbb19bee10093` · `io ba89b65cbd61abc6` · `fmt cae5250235f652ec` — the corpus is FQDN, so canonicalising must be a **no-op** for it |
| 10 | ★★★ **a real doc-row example now gets the RULED shape** | the `step-payload` example, dotted, formats with `let` binders paired and `defrecord`'s name riding. **This is what the stone is FOR** — it was under 120 and mis-shaped before |
| 11 | ★★ the 614 unchanged | `OVER120=0 WORST<=120` |
| 12 | ★★ no comment lost | `28` · `85` · `429` · `157` · `45` both sides |
| 13 | ★★ idempotent | every fixture, all three spellings |
| 14 | ★★ hygiene | `'col'` in rules **0** · `'120'` in rules **0** · `ClaimedUnder` **0** |
| 15 | ★★ wat-scripts load · wat-grep | `1 passed` · `wat_grep::*` **7 passed** |
| 16 | floor (ORCHESTRATOR) | `5189+` run, **0 FAILED** |
| 17 | clippy (ORCHESTRATOR) | `0` |

**Runtime prediction:** 60-100 min. The canonicaliser is small; retargeting 33 comparisons across 12
files is a codemod; row 6's demonstration is the careful part.

## Trap-doors named in advance

- **Row 6 IS the builder's requirement.** Everything else can pass while the three spellings are
  wired as three parallel comparisons — which works today and costs a 12-file sweep tomorrow.
  **Show the deletion.**
- **Row 4 nearly closed this stone as a non-issue.** A one-argument `defn` renders identically under
  the ruled rule and the generic fallthrough; the difference only appears at two. Any fixture that
  cannot discriminate is not a fixture.
- **Row 3 is the anti-"looks right" row.** Three outputs can each look reasonable and differ. Strip
  the head token and diff.
- **Row 9 is the no-op check.** The five real files are all FQDN; if canonicalising moves them, the
  fold is lossy in the direction that matters most.
- **Row 5 decides whether this is done once or twice.** Canonicalising to `:wat::core::` passes rows
  2, 3 and 4 and fails the builder's actual ask.
- **33 comparisons across 12 rule files is a `.wat` corpus migration** — R21, a recorded codemod,
  dry-run and diffed, not hand edits.
- ⚠ **`wat/*.wat` is FROZEN into the release binary** — rebuild between an edit and a measurement.
