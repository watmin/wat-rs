# WEIGH — STONE 255.10: ACCEPTED. ⭐ Delta 18 → 4, and a DEFAULT-DENY SECURITY GATE WAS OPEN.

Weighed against `465728902` by independent re-run, including **building both binaries myself**.
5 `src/` files, +267/−23. `git status --porcelain -- '*.wat'` = **0**. Not pushed.

## ⛔⛔ THE SECURITY FINDING IS REAL, AND WORSE THAN REPORTED

The SCORE says the F5 expand-time purity gate was bypassed by the symbol spelling. **I built the
pre-cure binary (`git checkout f1bef5944 -- src/`) and measured it:**

```
PRE-CURE   (:wat::kernel::println …) in a macro body  → rc=1  "default-deny F5 gate, arc 249"
PRE-CURE   (wat.kernel/println   …) in a macro body  → rc=0  ⛔⛔ CLEAN
POST-CURE  both spellings                             → rc=1, BYTE-IDENTICAL refusal
```

⛔ **The SCORE says the symbol form "stopped only because stdio services were absent." At `--check`
it did not stop at all — it is `rc=0`.** The macro was **DEFINED** with an impure body. Arc 249
stone O's whole property — *"fails at definition, not silently at first use"* — **was
spelling-dependent**, and the corpus flip would have made the bypass the default spelling.

⭐ **This is the most serious defect the arc has found, and the brief never looked at it.** It was
reached by following a failing file, not by the census frame — which is the argument for the
evidence gate over the count.

## Re-derived here

| gate | mine | SCORE | |
|---|---|---|---|
| originals clean (both sides from copies) | **160 / 179** | 160 | ✅ |
| converted clean | **156 / 179** | 156 | ✅ |
| ⭐ **NEW (clean→fail)** | **4** | 4 | ✅ **18 → 4** |
| ⭐ **RECOVERY (fail→clean)** | **0** | 0 | ✅ no forged green |
| floor | **5963/5963**, ×3 runs, **no `ARM.txt`** | same | ✅ |
| clippy / census | exit 0 · `no STOP-8`, **212 = 212**, 0 rc changes | same | ✅ |
| a `defsurface` residue file, spot-checked | `probe_arc272_rs1…` **FAIL → CLEAN** | 8 of 8 | ✅ |

⭐ **The 4 survivors are exactly the 4 the SCORE reported rather than cured** — the two arc-170
probes and `wat/source.wat` + `wat/holon/Ngram.wat`. **Nothing was cured quietly and nothing is
unaccounted for.**

## ⭐⭐ THE TEST IS THE BEST IN THIS ARC

Row 1 does not assert "both parse" — it asserts the two spellings resolve to a **byte-identical
identity path**. Row 2 pins the **full reason** for three markers (the rider **strengthened this
mid-stone** after noticing the kind alone would be satisfied by any malformation). Row 4 pins the
wall naming the **TRUE** undeclared type, byte for byte, **in both spellings** — ⭐ **so the stone
provably cannot trade a false red for a false green**, which was the one way this cure could have
gone wrong. The guard **has failed twice by targeted revert**, and cure 2 exists *because* reverting
it alone made the wall go silent on a genuinely broken surface.

## ⭐ IT CORRECTED THE PREVIOUS STONE, AND I CONFIRMED IT

**255.9's census classified `validate_pure_total` as LOUD. It is SILENT** — my pre-cure `rc=0`
proves it. ⛔ **A false negative in the immediately preceding stone's census**, which is the strongest
possible argument for the rider's re-framing: the census frame both briefs used
(`grep "Some(WatAST::Keyword"`) **cannot see** a `matches!` inside a closure (`is_callable_form`) or
a bare `starts_with(':')` string test (`keyword-node`). **Two of six cures were outside the frame.**

⭐ **The instrument earned itself:** the rider introduced a regression (cure 5 with
`canonical_identity` alone broke `Option/expect`'s member join) and **the committed delta list caught
it** — the file is in the sample. The list we instituted one stone ago paid for itself immediately.

## The brief was wrong three times

1. **"634 keyword read-sites" → 665**, and more importantly **the wrong FRAME** (above).
2. **`surface.rs:746` → the site is `:729`.**
3. **"15 of 18 are three user forms"** — the 8/4/3 split is exact, but one of my "other three"
   **IS** this class (it led to the F5 wall) and two are **not this class at all**. ⭐ **My framing
   would have had the rider cure two files blind.** The evidence gate is what stopped that.

**Eighteen corrections across sixteen stones.**

## Disclosed, and I am not discounting any of it

- **Process errors on floor runs 1 and 2** (rebuilt under one, edited test source under the other).
  ⭐ **Reported and NOT relied on**; run 3 on the frozen tree is the gate, and it is the one I checked.
- ⛔ **The shape-B/D sweep is real remaining exposure** — the rider read shape A plus only the B/D
  sites a failing file led it to, *"the frame followed the evidence, not the reverse."*
- ⛔ **154 of the 156 clean converted files were never RUN.** `--check` clean hides behaviour changes;
  `scan_for_setter` is the proof.
- ⛔ **The step-7 ordering is STILL ungated** — recommended as a stone for the **second** time.
- ⛔ **RECOVERY is still not institutionalized** — this stone printed it; nothing enforces it.

## VERDICT

**ACCEPTED.** Pushing.

⭐ **Sequencing note — the next stone got bigger and better-defined.** The two remaining delta files
are `TypeEnv::register_validated`'s raw `Some(e) if e == &def` where `types::type_exprs_same` already
exists — **the same `wat.type` denotation gap** as the two runtime tables that stopped 8d-ii. **They
are one stone, not two:** *wire type denotation into every type-identity comparison.* It closes the
8d-ii blocker **and** the last 2 delta files. ⚠ It makes a **duplicate-declaration wall** more
permissive — the red→false-green direction — so it needs a non-vacuity row proving a genuine
duplicate is still refused.

Then: the 416-test rete remainder · the shape-B/D sweep · mint `:wat::keyword::canonical-identity`
(no wat-level identity verb exists — measured) · RECOVERY as a standing gate · a gate on step-7
ordering · 255.8's wrong-join hole · then 8d-ii (fourth draw) · then 8d-iii.
