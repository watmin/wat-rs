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
  (:wat::rete::core::or true (:wat::core::i64::> (:wat::core::i64::/ 1 0) 0)))

;; The NON-VACUITY CONTROL for the entry above: the identical operand, REACHED, does raise. Without
;; this the short-circuit test could pass on an operand that was simply harmless.
(:wat::core::defn :user::form-or-control-raises [] -> :wat::core::bool
  (:wat::rete::core::or false (:wat::core::i64::> (:wat::core::i64::/ 1 0) 0)))

;; ── rows 3-5: THE ADMISSION TEST, in BOTH directions ────────────────────────────────────────
;; row 3 — a rete-module head IS admitted.
(:wat::core::defn :user::admit-rete-module? [] -> :wat::core::bool
  (:wat::rete::vocabulary-admitted? (:wat::core::quote :wat::rete::i64::>)))
;; row 4 — the bare rete ENGINE API (not a vocabulary sub-namespace) is refused.
(:wat::core::defn :user::refuse-engine-api? [] -> :wat::core::bool
  (:wat::rete::vocabulary-admitted? (:wat::core::quote :wat::rete::fire-rules)))
;; row 5 — a `:wat::core::` head is refused (never rete-namespaced at all).
(:wat::core::defn :user::refuse-core-head? [] -> :wat::core::bool
  (:wat::rete::vocabulary-admitted? (:wat::core::quote :wat::core::i64::+)))
