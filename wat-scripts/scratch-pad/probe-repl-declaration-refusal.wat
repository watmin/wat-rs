;; probe-repl-declaration-refusal.wat — DISCONFIRMING PROBE (arc 170, the REPL stone).
;;
;; THE ONE QUESTION: when the REPL's `E` phase evaluates a line, can it tell a
;; DECLARATION ("this form joins the def set") apart from a GENUINE ERROR ("this form
;; is wrong") — using only the substrate's own refusal, with no second list of
;; declaration heads to drift from `dispatch_keyword_head`'s?
;;
;; Why it decides the design: `:repl::turn` wants exactly one classifier. The substrate
;; already owns one — `#wat.runtime/DeclarationInExpressionPosition` (`runtime.rs:4129`
;; for `def`, `:4137` for `defclause`). If EVERY declaration head refuses distinguishably,
;; the REPL consults that authority and never copies it. If some heads refuse the same way
;; a typo does, the classifier is a lie and the design needs another answer.
;;
;; The suspicion this probe exists to test: `def`/`defclause` are refused at RUNTIME
;; dispatch, but `defn`/`defrecord`/`defenum` never reach it — they are FREEZE-time
;; declarations, so an `eval-ast!` of one may fail as an ordinary unknown-form error,
;; indistinguishable from a misspelling.
;;
;; RUN: target/release/wat wat-scripts/scratch-pad/probe-repl-declaration-refusal.wat
;; Read the EDN on stdout: each line is one case's `Result`. Compare the DECLARATION
;; cases against the CONTROL cases — if their error shapes are the same, the classifier
;; is not available and `:repl::turn`'s match arm as drafted is dishonest.

(:wat::core::defn :probe::try [label <- :wat::core::String  src <- :wat::core::String] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println label)
    (:wat::kernel::println
      (:wat::eval-ast! (:wat::core::first (:wat::core::read-string src))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    ;; ── DECLARATIONS — each should refuse in a way the REPL can recognize ──
    (:probe::try "A-def"       "(:wat::core::def :usr::x 1)")
    (:probe::try "B-defn"      "(:wat::core::defn :usr::f [] -> :wat::core::i64 1)")
    (:probe::try "C-defrecord" "(:wat::core::defrecord :usr::R [a <- :wat::core::i64])")
    (:probe::try "D-defenum"   "(:wat::core::defenum :usr::E :wat::enum::Pure :A [])")

    ;; ── CONTROLS — genuine errors; the REPL must NOT mistake these for declarations ──
    (:probe::try "E-unknown-fn" "(:usr::no-such-fn 1)")
    (:probe::try "F-typo-head"  "(:wat::core::deffn :usr::g [] 1)")

    ;; ── CONTROL — a plain expression; must simply evaluate ──
    (:probe::try "G-expr"       "(:wat::core::+ 1 2)")))
