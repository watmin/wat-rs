# ⛔ NOTE (arc 109) — the Tuple arm is mode-blind. It BLOCKS ②-iii.

**Filed 2026-08-20. MEASURED.** `②-i` (`0422b67ff`) gave `type_expr_to_clojure_form` a head-spelling
mode and bracketed the `Parametric` arm. Its rider flagged one thing as a deliberate judgment call:

> *"`TypeExpr::Tuple`'s head (`wat.type/Tuple`) I left OUT OF SCOPE for `mode` — it's not part of the
> 4-way ladder Room 2 scopes to, and nothing in the acceptance criteria or the 8-fixture contract suite
> exercises a COLON-mode Tuple."*

That was accurate and correctly reported. **②-ii then walked straight into it**, and the consequence is
larger than "one arm is unparameterized". Measured, at HEAD:

```
:wat::core::nil                      →  (wat.type/Tuple)
:wat::core::Result<nil,String>       →  (:wat::core::Result [(wat.type/Tuple) :wat::core::String])
:(wat::core::i64,wat::core::i64,…)   →  (wat.type/Tuple :wat::core::i64 :wat::core::i64 …)
```

**Three distinct faults, not one:**
1. **Wrong head spelling in COLON mode** — `wat.type/Tuple` where `:wat::core::Tuple` is required.
2. **MIXED spelling inside one otherwise-correct form** — the `Result` case. Half migrated, half not.
3. ★ **The Tuple arm does not BRACKET either** — its args are still spliced flat. So `Tuple` is
   un-migrated on *both* axes this campaign is about, and #3 was not in the ②-i rider's report.

★ **`nil` is the reason this is load-bearing.** `keyword/to-type-form-colon` canonicalizes
`:wat::core::nil` to `TypeExpr::Tuple(vec![])` (`src/types.rs:4728`), so **every `-> :wat::core::nil`
return annotation routes through the Tuple arm** — roughly 1,126 sites.

## How ②-ii routed around it, and what that costs

`wat-scripts/fixes/parametrics-take-a-type-vector.wat` does two things about it:

1. **Dropped the post-arrow rule.** `wat/fix.wat`'s `fix-text-leaf-edits` rewrites ANY post-arrow
   keyword, type-shaped or not. In CLOJURE mode that is harmless; in COLON mode it drags every
   `-> :wat::core::nil` through the Tuple arm. Measured redundant for every real non-`nil` site — a
   post-arrow type-shaped keyword is already caught by the type-shaped rule regardless of position —
   so it was removed, losing nothing.
2. **A rendered-output guard.** After computing a replacement via `ast->source`, if the result contains
   `wat.type/`, the edit is REFUSED and the original text left untouched.

★ The guard is the interesting part: rule-selection alone could not fix this, because the fault
reappears one level down, *inside* a legitimate parametric (`Result<nil,Error>`). Only inspecting the
**rendered output** — not the input keyword's shape — catches it. The codemod therefore **skips rather
than corrupts**, which is the right failure direction.

**The cost, measured — and three different numbers were quoted before one was earned:**

```
30     standalone tuple keywords  :(A,B)          ⚠ a first pattern said 2 — it required LOWERCASE
                                                    members and could not see `:(A,B,C)` or
                                                    `:(String, Vector<String>)`
66     `nil` nested inside a parametric
1031   bare `-> :wat::core::nil` return annotations
```

The 30 + 66 are what the guard SKIPS. The 1,031 are what the **dropped post-arrow rule** would have
corrupted — the larger number, and the reason dropping that rule mattered more than the guard did.

⚠ ②-ii's report estimated "~220 standalone tuple, ~36 nil-in-parametric"; my first counter-measurement
said 2 and 66. Neither was right, both were heuristic greps, and **I wrote a fabricated "~247" into an
earlier draft of this note by combining them.** The numbers above are the re-measured ones and the
patterns are stated so the next reader can falsify them.
`[[feedback_validate_a_search_pattern_before_trusting_its_count]]`

## ⛔ Consequence: ②-iii must NOT run until this is closed

Applying the codemod today migrates most of the corpus and silently leaves a Tuple-shaped remainder in
the old spelling. Since ③ makes the angle form illegal, that remainder becomes a hard error with no
codemod able to fix it.

**Close it first:** give `TypeExpr::Tuple` the same treatment `Parametric` got in ②-i — bracket its
args, and honour the head-spelling mode. Then re-run ②-ii's proofs; the guard should stop firing, and
the skipped ~283 sites should convert.

⚠ **One design question this exposes and does not answer:** if `nil` canonicalizes to the empty tuple,
what is `-> :wat::core::nil` *supposed* to become? `(:wat::core::Tuple [])` is the mechanical answer
and reads badly for 1,126 return annotations. `nil` may deserve to stay a scalar keyword rather than
round-tripping through `Tuple`. **That is a builder decision, not a rider's**, and it should be made
before the Tuple arm is written — the answer changes what the arm should emit.
