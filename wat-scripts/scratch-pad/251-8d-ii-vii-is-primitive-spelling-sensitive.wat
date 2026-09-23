;; ─── Arc 251 Stone 8d-ii (SEVENTH DRAW) — IS `:wat::rete::primitive?` SPELLING-SENSITIVE? ────
;;
;; The seventh brief records that the orchestrator probed this directly and got `false` for all
;; four inputs, including one it expected to pass, and concluded its own probe shape was wrong.
;; This is the re-probe, with the ARGUMENT SHAPE the intrinsic actually declares:
;;
;;     (:wat::rete::primitive? <expr :- :wat::WatAST>)
;;
;; i.e. the expression must arrive QUOTED. An unquoted form is EVALUATED and the predicate then
;; classifies the value, not the form — which is how a probe returns `false` for everything.
;;
;; Each row is printed as `<row> <verdict>`; the pairs are keyword-spelling vs symbol-spelling of
;; the SAME form, so a row pair that disagrees is a spelling-sensitivity, and a pair that agrees
;; is not — whatever the verdict.
;;
;; ⛔ MEASUREMENT, never a ratchet.

(:wat::core::defrecord :p7::R [x <- :wat::core::i64])

(:wat::core::defn :p7::show [label <- :wat::core::String  v <- :wat::core::bool] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::string::concat label " = " (:wat::core::bool::to-string v))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    ;; ── a rete-namespaced structural guard ──
    (:p7::show "A rete-cond   KEYWORD" (:wat::rete::primitive? (:wat::core::quote (:wat::rete::core::cond (true 1) (:else 2)))))
    (:p7::show "B rete-cond   SYMBOL " (:wat::rete::primitive? (:wat::core::quote (wat.rete.core/cond (true 1) (:else 2)))))
    ;; ── a rete-namespaced primitive op ──
    (:p7::show "C rete-i64+   KEYWORD" (:wat::rete::primitive? (:wat::core::quote (:wat::rete::i64::+ 1 2))))
    (:p7::show "D rete-i64+   SYMBOL " (:wat::rete::primitive? (:wat::core::quote (wat.rete.i64/+ 1 2))))
    ;; ── core-spelled COMPUTATION: law A must refuse BOTH ──
    (:p7::show "E core-i64+   KEYWORD" (:wat::rete::primitive? (:wat::core::quote (:wat::i64::+ 1 2))))
    (:p7::show "F core-i64+   SYMBOL " (:wat::rete::primitive? (:wat::core::quote (wat.i64/+ 1 2))))
    ;; ── core-spelled STRUCTURAL GUARD: law A must refuse BOTH ──
    (:p7::show "G core-cond   KEYWORD" (:wat::rete::primitive? (:wat::core::quote (:wat::core::cond (true 1) (:else 2)))))
    (:p7::show "H core-cond   SYMBOL " (:wat::rete::primitive? (:wat::core::quote (wat.core/cond (true 1) (:else 2)))))
    ;; ── the DECLARATION-DERIVED construction door, post-lowering (the type is argument 0) ──
    (:p7::show "I kwargs-ctor KEYWORD" (:wat::rete::primitive? (:wat::core::quote (:wat::core::kwargs-construct :p7::R :x 1))))
    (:p7::show "J kwargs-ctor SYM-HEAD" (:wat::rete::primitive? (:wat::core::quote (wat.core/kwargs-construct :p7::R :x 1))))
    (:p7::show "K kwargs-ctor SYM-TYPE" (:wat::rete::primitive? (:wat::core::quote (:wat::core::kwargs-construct p7/R :x 1))))
    ;; ── the same door, PRE-lowering (the type is the head) ──
    (:p7::show "L surface-ctor KEYWORD" (:wat::rete::primitive? (:wat::core::quote (:p7::R :x 1))))
    (:p7::show "M surface-ctor SYMBOL " (:wat::rete::primitive? (:wat::core::quote (p7/R :x 1))))
    ;; ── TIGHTNESS CONTROL: the verb alone must NOT be enough ──
    (:p7::show "N kwargs-undeclared   " (:wat::rete::primitive? (:wat::core::quote (:wat::core::kwargs-construct :p7::NotAType :x 1))))
    (:wat::kernel::println "")))
