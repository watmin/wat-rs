;; probe-defclause-refusal-restored.wat — CONFIRMING PROBE, arc 255 Stone 1c-d.
;;
;; THE ONE QUESTION: `SEAM:214` recorded that `defclause` lost its named
;; `DeclarationInExpressionPosition` refusal when the hand-rolled `runtime.rs` arms for
;; `def`/`defclause` were retired in favour of the registry-first door's `@Purity Unevaluated`
;; guard — `def` had a registry row and kept its refusal; `defclause` had none and fell through
;; to the generic `UnknownFunction` fallback instead. Stone 1c-d gives `defclause` a registry
;; row (`intrinsic/special/defclause.rs`, `@Purity Unevaluated`). This probe asks: does
;; `dispatch_keyword_head`/`dispatch_keyword_head_value`'s `Unevaluated`-keyed guard
;; (`src/runtime.rs:1951`/`:2086`) now answer `DeclarationInExpressionPosition` for
;; `:wat::core::defclause` again, exactly as it already does for `def`?
;;
;; Modeled on `wat-scripts/scratch-pad/probe-repl-declaration-refusal.wat`'s idiom
;; (`eval-ast!` over freshly `read-string`'d, UNCHECKED source) so a genuine expression-position
;; encounter is exercised the same way that probe already exercises `def`/`defn`/`defrecord`/
;; `defenum` against CONTROL cases (an unknown function, a typo'd head) — the substrate's own
;; refusal must be distinguishable from an ordinary error, or the restoration is not real.
;;
;; RUN: target/release/wat wat-scripts/scratch-pad/probe-defclause-refusal-restored.wat
;; Read the EDN printed for each labelled case. Case A (defclause in expression position) is
;; the one this stone's report must quote in full.

(:wat::core::defn :probe::try [label <- :wat::core::String  src <- :wat::core::String] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println label)
    (:wat::kernel::println
      (:wat::eval-ast! (:wat::core::first (:wat::core::match (:wat::core::read-string src) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None))))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    ;; ── A. THE RESTORED REFUSAL — defclause in expression position ──
    ;; Expect: #wat.runtime/DeclarationInExpressionPosition naming ":wat::core::defclause",
    ;; the same shape case B (def) already gets — NOT UnknownFunction.
    (:probe::try "A-defclause" "(:wat::core::defclause :usr::probe-inner ([n <- :wat::core::i64] -> :wat::core::i64 n))")

    ;; ── B. CONTROL — def, the sibling that never lost its refusal ──
    (:probe::try "B-def"       "(:wat::core::def :usr::x 1)")

    ;; ── C. CONTROL — a genuine unknown function; must NOT be mistaken for a declaration ──
    (:probe::try "C-unknown-fn" "(:usr::no-such-fn 1)")

    ;; ── D. CONTROL — a typo'd declaration head; must ALSO not be mistaken for a declaration ──
    (:probe::try "D-typo-head"  "(:wat::core::defclaus :usr::g ([n <- :wat::core::i64] -> :wat::core::i64 n))")

    ;; ── E. CONTROL — a plain expression; must simply evaluate ──
    (:probe::try "E-expr"       "(:wat::core::+ 1 2)")))
