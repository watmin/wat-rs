;; tests/rete/probe_arc278_55_slice_one_vocabulary.wat — co-located fixture for the sibling probe
;; (.rs), slurped via call_beside_value(file!(), entry). Arc 278 #55 (S3b+S4) slice one: THE ONE
;; TABLE (`src/rete/vocabulary.rs`), its four demonstration ops, and the module-set admission test.

;; ── the four ops dispatch correctly (EXPECTATIONS row 7) ────────────────────────────────────
(:wat::core::defn :user::alias-gt [] -> :wat::core::bool
  (:wat::rete::i64::> 5 3))

(:wat::core::defn :user::fallback-no-overflow [] -> :wat::core::i64
  (:wat::rete::i64::+ 2 3 :undefined -1))

(:wat::core::defn :user::form-and [] -> :wat::core::bool
  (:wat::rete::core::and true (:wat::rete::i64::> 5 3)))

;; ── row 9: the fallback FIRES on overflow — no raise, `-1` substituted ──────────────────────
(:wat::core::defn :user::fallback-overflow [] -> :wat::core::i64
  (:wat::rete::i64::+ 9223372036854775807 1 :undefined -1))

;; ── row 6: COMPOSITION, proven by a run — a user defn built from all four ops ───────────────
(:wat::core::defn :test::rete-combo [a <- :wat::core::i64  b <- :wat::core::i64] -> :wat::core::bool
  (:wat::rete::core::and
    (:wat::rete::i64::> (:wat::rete::i64::+ a b :undefined -1) 0)
    (:wat::rete::i64::> a 0)))

(:wat::core::defn :user::combo-is-pure? [] -> :wat::core::bool
  (:wat::rete::pure? (:wat::core::quote (:test::rete-combo 3 4))))
(:wat::core::defn :user::combo-is-deterministic? [] -> :wat::core::bool
  (:wat::rete::deterministic? (:wat::core::quote (:test::rete-combo 3 4))))

;; ── #56: the two head-table form mirrors ────────────────────────────────────────────────────
;; `not` is an ALIAS (a plain strict fn), NOT a form — the design stone's corrected class table.
(:wat::core::defn :user::alias-not [] -> :wat::core::bool
  (:wat::rete::core::not false))

;; `or` is a FORM, and the ONLY thing distinguishing that from an Alias is LAZINESS. So the gate
;; is the short-circuit, not the answer: were `or` strict, the second operand raises
;; DivisionByZero and this entry never returns.
(:wat::core::defn :user::form-or-short-circuits [] -> :wat::core::bool
  (:wat::rete::core::or true (:wat::i64::> (:wat::i64::/ 1 0) 0)))

;; The NON-VACUITY CONTROL for the entry above: the identical operand, REACHED, does raise. Without
;; this the short-circuit test could pass on an operand that was simply harmless.
(:wat::core::defn :user::form-or-control-raises [] -> :wat::core::bool
  (:wat::rete::core::or false (:wat::i64::> (:wat::i64::/ 1 0) 0)))

;; ── #56 phase 1: the head-table pair (`if`/`let`) ───────────────────────────────────────────
;; EXPECTATIONS row 3 — `if` routes to `infer_if`, NOT the bool short-circuit arm: non-bool
;; branches (i64) must unify and type-check clean. Pre-phase-1 this would have been routed to
;; `infer_boolean_shortcircuit`, which requires every arg to be `:bool` — this entry would have
;; FAILED TO LOAD (a type error on the branches) if that bug were still present.
(:wat::core::defn :user::rete-if-non-bool-branches [] -> :wat::core::i64
  (:wat::rete::core::if true 1 2))

;; row 4 — `if` does not evaluate the untaken branch: the untaken (else) branch raises.
(:wat::core::defn :user::rete-if-short-circuits [] -> :wat::core::i64
  (:wat::rete::core::if true 1 (:wat::i64::/ 1 0)))

;; row 5 — the NON-VACUITY CONTROL for row 4: the identical raising operand, actually REACHED
;; (condition now false, so the else branch fires), DOES raise.
(:wat::core::defn :user::rete-if-control-raises [] -> :wat::core::i64
  (:wat::rete::core::if false 1 (:wat::i64::/ 1 0)))

;; row 6 — `let` actually scopes a binding: bind, then read it back.
(:wat::core::defn :user::rete-let-scopes [] -> :wat::core::i64
  (:wat::rete::core::let [x 42] x))

;; row 7/8 — THE TCO GATE. A rete `if` in TAIL POSITION must reach `eval_if_tail` exactly as its
;; core twin does, or this recursion trades TCO for a native stack frame per call and SIGSEGVs
;; long before 200000 (proven: `wat-scripts/scratch-pad/probe-s5-tail-position-is-load-bearing.wat`,
;; whose `and`-tailed sibling — a Form `eval_tail` does NOT intercept — segfaults at this exact
;; depth). Depth chosen to match that proof exactly, not a smaller number that TCO doesn't need.
(:wat::core::defn :probe::rete-countdown-if [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::rete::core::if (:wat::i64::<= n 0)
    0
    (:probe::rete-countdown-if (:wat::i64::- n 1))))

(:wat::core::defn :user::rete-if-tail-tco-survives-depth [] -> :wat::core::i64
  (:probe::rete-countdown-if 200000))

;; ── #56 phase 2: `match`, the first of the structural-guard pair ────────────────────────────
;; row 10 — a rete `match` whose PATTERN would fail as an expression classifies clean. A tagged
;; enum variant (user-declared, so `constructor_meta` recognizes it — deliberately NOT
;; `:wat::core::Option`'s `Some`/`None`, which are natively-typed and hit an orthogonal,
;; pre-existing purity gap unrelated to this stone): `(:test::S5Shape::Circle r)`'s pattern head
;; is a LIST (not a Keyword/Symbol). If `classify_expr` ever walked this arm generically instead
;; of skipping the pattern structurally, the "General list" arm's own head-shape check rejects a
;; non-keyword/non-symbol head outright, so this would be misclassified as an axis violation
;; despite the match itself being perfectly pure.
(:wat::core::defenum :test::S5Shape :wat::enum::Pure
  :Circle [r <- :wat::core::i64]
  :Square)

(:wat::core::defn :test::rete-match-shape-area [s <- :test::S5Shape] -> :wat::core::i64
  (:wat::rete::core::match s
    ((:test::S5Shape::Circle r) (:wat::i64::* r r))
    (:test::S5Shape::Square    0)))

(:wat::core::defn :user::rete-match-pattern-not-classified-as-expr-pure [] -> :wat::core::bool
  (:wat::rete::pure? (:wat::core::quote (:test::rete-match-shape-area (:test::S5Shape::Circle 5)))))
(:wat::core::defn :user::rete-match-pattern-not-classified-as-expr-det [] -> :wat::core::bool
  (:wat::rete::deterministic? (:wat::core::quote (:test::rete-match-shape-area (:test::S5Shape::Circle 5)))))

;; ── S5's last form (closing #56's leftover): `fn`, the second of the structural-guard pair ─────
;; The builder's target form (BRIEF-s5-fn-mirror.md) — an anonymous rete `fn` type-checks (routes
;; through `infer_rete_form` to `infer_fn`, same as `check.rs`'s own `":wat::core::fn"` arm) and
;; EVALUATES as a value: applied via `:wat::core::apply` to a real argument. `0 + 5` never
;; overflows, so the `:undefined -1` fallback is dead here — a plain positive-path proof, not the
;; fallback mechanism (already covered by `:user::fallback-no-overflow` above).
(:wat::core::defn :user::rete-fn-target-form [] -> :wat::core::i64
  (:wat::core::apply
    (:wat::rete::core::fn [x <- :wat::core::i64] -> :wat::core::i64
      (:wat::rete::i64::+ 0 x :undefined -1))
    [5]))

;; rows 4+5 — the fence checks the BODY, in BOTH directions (together, not separately — a
;; body-check test that only shows the impure case proves nothing about the pure one). Return type
;; is a plain flat keyword in both, so this pair isolates BODY purity specifically (the
;; return-type-SLOT trick lives in the next section, testing a different thing).
(:wat::core::defn :user::rete-fn-impure-body-is-not-pure [] -> :wat::core::bool
  (:wat::rete::pure? (:wat::core::quote
    (:wat::rete::core::fn [] -> :wat::core::i64
      (:wat::io::IOReader/open-file "x")))))
(:wat::core::defn :user::rete-fn-impure-body-is-not-deterministic [] -> :wat::core::bool
  (:wat::rete::deterministic? (:wat::core::quote
    (:wat::rete::core::fn [] -> :wat::core::i64
      (:wat::io::IOReader/open-file "x")))))
;; the control: the identical shape, pure body, classifies pure/deterministic.
(:wat::core::defn :user::rete-fn-pure-body-is-pure [] -> :wat::core::bool
  (:wat::rete::pure? (:wat::core::quote
    (:wat::rete::core::fn [] -> :wat::core::i64
      (:wat::rete::i64::+ 1 2)))))
(:wat::core::defn :user::rete-fn-pure-body-is-deterministic [] -> :wat::core::bool
  (:wat::rete::deterministic? (:wat::core::quote
    (:wat::rete::core::fn [] -> :wat::core::i64
      (:wat::rete::i64::+ 1 2)))))

;; row 6 — the structural guard fires: the RETURN-TYPE SLOT (never evaluated — `parse_type_node`,
;; `src/types.rs`, accepts a parametric `(Ctor arg…)` List there) holds an impure-LOOKING head.
;; Never reached because this whole fn lives inside `quote` (never type-checked — same "quote
;; never checks its content" property `probe_arc278_59...`'s tests rely on) AND because the
;; structural guard isolates the body (`items.get(i+2..)`), skipping the params vector + the
;; return-type slot entirely. If `classify_expr` ever fell through to the "General list" arm
;; instead (the guard's literal-only pre-widening condition, reverted at row 7's manual gate),
;; that arm recurses into EVERY element after the head — INCLUDING the return-type slot — and
;; `:wat::io::IOReader/open-file` is a real `is_effectful_op` head, so it would deny purity despite
;; the body being trivially pure. The body itself is pure arithmetic either way; this test isolates
;; the SLOT, not the body (rows 4/5 already isolate the body).
(:wat::core::defn :user::rete-fn-return-type-slot-not-classified-as-expr-pure [] -> :wat::core::bool
  (:wat::rete::pure? (:wat::core::quote
    (:wat::rete::core::fn [] -> (:wat::io::IOReader/open-file "unused-return-type-slot")
      (:wat::rete::i64::+ 1 2)))))
(:wat::core::defn :user::rete-fn-return-type-slot-not-classified-as-expr-det [] -> :wat::core::bool
  (:wat::rete::deterministic? (:wat::core::quote
    (:wat::rete::core::fn [] -> (:wat::io::IOReader/open-file "unused-return-type-slot")
      (:wat::rete::i64::+ 1 2)))))

;; ── rows 3-5: THE ADMISSION TEST, in BOTH directions ────────────────────────────────────────
;; row 3 — a rete-module head IS admitted.
(:wat::core::defn :user::admit-rete-module? [] -> :wat::core::bool
  (:wat::rete::vocabulary-admitted? (:wat::core::quote :wat::rete::i64::>)))
;; row 4 — the bare rete ENGINE API (not a vocabulary sub-namespace) is refused.
(:wat::core::defn :user::refuse-engine-api? [] -> :wat::core::bool
  (:wat::rete::vocabulary-admitted? (:wat::core::quote :wat::rete::fire-rules)))
;; row 5 — a `:wat::core::` head is refused (never rete-namespaced at all).
(:wat::core::defn :user::refuse-core-head? [] -> :wat::core::bool
  (:wat::rete::vocabulary-admitted? (:wat::core::quote :wat::i64::+)))
