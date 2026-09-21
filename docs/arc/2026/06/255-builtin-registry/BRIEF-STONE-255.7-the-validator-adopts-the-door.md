# BRIEF — STONE 255.7: the rete VALIDATOR adopts the door

**Drawn 2026-09-21 against `main` @ `f9914bf70`** (floor 5953/5953, clippy 0, census `no STOP-8`).
Delta baseline **61**. Predecessor: **255.6 LANDED** — rete's `:when` classifier is through the door.

## THE FINDING THIS STONE ANSWERS — the executor's, from 255.6

> *"The five closed files failed on `:when`. The remaining **16** fail on `:then` insert heads:
> `validate_then_form: fact_items[0] must be WatAST::Keyword` — converted
> `(weather/ColdAndWindy :location ?loc)` → `MalformedClause`."*

⭐ **And it checked the generator class unprompted:** `wat/query.wat` mints a fact-**bind** (already
`:-`), and **no constructor of `(Type/Name …)` clause heads exists.** ⇒ **`:then` heads are SOURCE
LITERALS.** The codemod already converts them; the validator is behind the corpus.

## ⛔ IT IS NOT ONE SLOT. THE ORCHESTRATOR ALMOST BRIEFED IT AS ONE.

The obvious brief is *"fix `validate/mod.rs:1289`."* ⛔ **Measured: that file has 14 `WatAST::Keyword`
gates**, at lines **135 · 152 · 172 · 560 · 782 · 784 · 1009 · 1059 · 1065 · 1084 · 1151 · 1289 ·
1336 · 1581**.

⛔ **DO NOT TAKE THAT LIST AS THE WORK.** It is a `grep` for one node variant and it is **exactly the
shape that produced the eighth and ninth corrections** (`defsurface` and `newtype`, missed by
enumeration). **DERIVE the set**, and say how.

⚠ **They are NOT all the same kind** — measured, three distinct shapes at least:

| shape | example | needs |
|---|---|---|
| extract a **type** from a head | `:1289` `fact_items[0]` → `fact_type` | canonicalize, then the existing `::`/lookup path |
| compare against a **literal name** | `:152` `k == ":wat::rete::make-rule"` · `:172` `make-query` | ⛔ canonicalize **both sides** — a converted `wat.rete/make-rule` will never equal that literal |
| **list Keyword among node kinds** | `:560` a `match` arm | ⚠ **may be correct as-is.** A widening here could admit a Symbol somewhere a Symbol is genuinely wrong. |

⭐ **Classify every gate before changing any.** 255.6 is the model: it **measured** that
`expr_is_provably_boolean` was not yet reached, and found `check_fence_interior` **fail-opening** —
silently skipping checks. ⛔ **A validator that widens without classification converts a red into a
skipped check**, which is the worse failure.

## The work

1. **Derive the gate set** in the rete validator — not from this brief's line numbers.
2. **Classify each**: type-extraction · literal-name comparison · legitimate-Keyword-only.
   **State the classification.** ⛔ A gate changed without a stated class is unreviewable.
3. **Route the first two classes through `canonical_identity`** — the door 255.6 already wired into
   `clause.rs`. ⛔ **One door. Not a second Symbol arm**, which is the defect behind this arc's
   32-red.
4. **RE-RUN THE DELTA** (baseline **61**) **+ classification**, one named tree, **re-converted with
   the current codemod.** ⚠ `ReteCheckErrors` **16** should collapse. **If a residue survives, name
   it** — you have done that four stones running and it has been right every time.

## The gate

- ⭐ **THE DELTA (baseline 61) + classification table**, one named tree.
- ⛔ **NON-VACUITY BOTH WAYS, and this stone needs it most:**
  - a converted `:then` head `(weather/ColdAndWindy :location ?loc)` **validates**;
  - a genuinely bad `:then` head **still fails** — ⭐ **and prove the failure is still DIAGNOSED, not
    SKIPPED.** A widened validator that stops checking is `check_fence_interior`'s fail-open all over
    again, and it would pass this gate silently unless you assert the error.
  - the retired `(?k <- :k)` **stays `MalformedClause`**.
- `scripts/floor.sh` green; clippy `-D warnings --all-targets --workspace` 0; census `no STOP-8`.
  Run crate clippy **and the lint suite** yourself.
- Predict the test delta; confirm with `cargo nextest list`.
- ⛔ **NOT ONE `.wat` CONVERTED.** Parser/validator stone. Fixtures expected.

## Doctrine

- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Capture whole the first time.
- ⛔ **ONE DOOR** — `canonical_identity`. This arc's most expensive red came from a second consult.
- ⛔ **A WIDENED CHECK THAT STOPS CHECKING IS WORSE THAN A RED.** 255.6 found exactly that
  (`check_fence_interior` fail-opening). **Assert what each widened gate still refuses.**
- ⛔ **REPORT WHAT A DOOR CANNOT EXPRESS.** Seven stones running — and in 255.6 the executor was
  **right against the brief** (`is_known_type` would have broken the classifier's documented
  TypeEnv-independence). **Do that again.**
- ⛔ **If this brief contradicts the code or the runner, THEY WIN.** **Twelve corrections across ten
  stones.** This brief openly hands you a 14-line `grep` it tells you not to trust. **Assume a
  thirteenth.**
- Work only in `/home/john/work/holon/wat-rs`. **Do not push.**

## Out of scope

- **8d-ii / 8d-iii** — they wait on the delta.
- **The registry residue** — `UnresolvedReference` 23 (declaration names, `mem-store::start`, and
  `:wat::core::not-a-special-form` ×3 which is a **deliberate negative-test name that stays**),
  `defsurface :messages` 8, gap-2 tail 4.
- **`fn` param annotations** — 7,028 sites, correctly still `<-`; they move with 8d-iii.
