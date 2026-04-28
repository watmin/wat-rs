;; wat-tests/holon/term.wat — tests for arc 073's term decomposition
;; primitives.
;;
;; Three substrate functions decompose a HolonAST into the Prolog/
;; population-code form: template (cell type), slots (tuning values),
;; ranges (receptive fields). The lab cache slice (umbrella 059) and
;; future query / recall consumers compose these directly.
;;
;;   template :HolonAST -> :HolonAST            ;; replace Thermometer values
;;                                              ;; with SlotMarker (min, max)
;;   slots    :HolonAST -> :Vec<f64>            ;; pre-order Thermometer values
;;   ranges   :HolonAST -> :Vec<(f64,f64)>      ;; pre-order Thermometer ranges
;;
;; Templates compare exactly (HashMap-keyable). Slots and ranges are
;; parallel in length and order; the TermStore::get path uses them
;; together to score per-slot tolerance.

;; ─── Template collapses thoughts with different tuning ─────────────
;;
;; Two RSI thoughts at different values produce IDENTICAL templates.
;; Same cell type; different tuning. The whole point of the
;; decomposition.

(:wat::test::deftest :wat-tests::holon::term::test-template-collapses-tuning
  ()
  (:wat::core::let*
    (((rsi-70 :wat::holon::HolonAST)
      (:wat::holon::Bind
        (:wat::holon::leaf :rsi-thought)
        (:wat::holon::Thermometer 70.0 0.0 100.0)))
     ((rsi-30 :wat::holon::HolonAST)
      (:wat::holon::Bind
        (:wat::holon::leaf :rsi-thought)
        (:wat::holon::Thermometer 30.0 0.0 100.0)))
     ((tpl-70 :wat::holon::HolonAST) (:wat::holon::term::template rsi-70))
     ((tpl-30 :wat::holon::HolonAST) (:wat::holon::term::template rsi-30)))
    ;; Templates are structural; templates can't go through `encode`
    ;; (SlotMarker is unencodable), so we compare via :wat::core::=
    ;; which uses HolonAST's PartialEq impl directly.
    (:wat::test::assert-eq tpl-70 tpl-30)))

;; ─── Template distinguishes different ranges ──────────────────────
;;
;; Same shape, same value, different (min, max) → distinct templates.
;; Different cell type; the receptive field is part of the template.

(:wat::test::deftest :wat-tests::holon::term::test-template-distinguishes-ranges
  ()
  (:wat::core::let*
    (((a :wat::holon::HolonAST)
      (:wat::holon::Bind
        (:wat::holon::leaf :x)
        (:wat::holon::Thermometer 0.5 0.0 1.0)))
     ((b :wat::holon::HolonAST)
      (:wat::holon::Bind
        (:wat::holon::leaf :x)
        (:wat::holon::Thermometer 0.5 -1.0 1.0)))
     ((tpl-a :wat::holon::HolonAST) (:wat::holon::term::template a))
     ((tpl-b :wat::holon::HolonAST) (:wat::holon::term::template b)))
    (:wat::test::assert-eq
      (:wat::core::= tpl-a tpl-b)
      false)))

;; ─── Template distinguishes different atom heads ──────────────────
;;
;; Same range, same value, different keyword → distinct templates.
;; Different cell type; the surrounding structure is part of the template.

(:wat::test::deftest :wat-tests::holon::term::test-template-distinguishes-atoms
  ()
  (:wat::core::let*
    (((rsi :wat::holon::HolonAST)
      (:wat::holon::Bind
        (:wat::holon::leaf :rsi-thought)
        (:wat::holon::Thermometer 70.0 0.0 100.0)))
     ((macd :wat::holon::HolonAST)
      (:wat::holon::Bind
        (:wat::holon::leaf :macd-thought)
        (:wat::holon::Thermometer 70.0 0.0 100.0)))
     ((tpl-rsi :wat::holon::HolonAST) (:wat::holon::term::template rsi))
     ((tpl-macd :wat::holon::HolonAST) (:wat::holon::term::template macd)))
    (:wat::test::assert-eq
      (:wat::core::= tpl-rsi tpl-macd)
      false)))

;; ─── Slots: pre-order extraction of Thermometer values ────────────

(:wat::test::deftest :wat-tests::holon::term::test-slots-pre-order
  ()
  (:wat::core::let*
    (((bundled :wat::holon::BundleResult)
      (:wat::holon::Bundle
        (:wat::core::vec :wat::holon::HolonAST
          (:wat::holon::Thermometer 70.0 0.0 100.0)
          (:wat::holon::Thermometer 0.25 -1.0 1.0))))
     ((form :wat::holon::HolonAST)
      (:wat::core::match bundled -> :wat::holon::HolonAST
        ((Ok h)  h)
        ((Err _) (:wat::holon::Atom "unreachable"))))
     ((slots :Vec<f64>) (:wat::holon::term::slots form))
     ((n :i64) (:wat::core::length slots)))
    (:wat::test::assert-eq n 2)))

;; ─── Slots and ranges parallel in length ──────────────────────────

(:wat::test::deftest :wat-tests::holon::term::test-slots-ranges-parallel
  ()
  (:wat::core::let*
    (((bundled :wat::holon::BundleResult)
      (:wat::holon::Bundle
        (:wat::core::vec :wat::holon::HolonAST
          (:wat::holon::Thermometer 70.0 0.0 100.0)
          (:wat::holon::Thermometer 0.25 -1.0 1.0))))
     ((form :wat::holon::HolonAST)
      (:wat::core::match bundled -> :wat::holon::HolonAST
        ((Ok h)  h)
        ((Err _) (:wat::holon::Atom "unreachable"))))
     ((slot-count :i64)
      (:wat::core::length (:wat::holon::term::slots form)))
     ((range-count :i64)
      (:wat::core::length (:wat::holon::term::ranges form))))
    (:wat::test::assert-eq slot-count range-count)))

;; ─── Empty slots for forms with no Thermometer leaves ─────────────

(:wat::test::deftest :wat-tests::holon::term::test-slots-empty-for-thermometer-free
  ()
  (:wat::core::let*
    (((form :wat::holon::HolonAST)
      (:wat::holon::Bind
        (:wat::holon::leaf :x)
        (:wat::holon::leaf 42)))
     ((slots :Vec<f64>) (:wat::holon::term::slots form))
     ((n :i64) (:wat::core::length slots)))
    (:wat::test::assert-eq n 0)))

;; ─── Decomposing a template yields no slots ───────────────────────
;;
;; A template (which already contains SlotMarker in place of every
;; Thermometer) carries no extractable values — SlotMarker is a
;; placeholder, not a tuning point.

(:wat::test::deftest :wat-tests::holon::term::test-template-has-no-slots
  ()
  (:wat::core::let*
    (((form :wat::holon::HolonAST)
      (:wat::holon::Bind
        (:wat::holon::leaf :rsi-thought)
        (:wat::holon::Thermometer 70.0 0.0 100.0)))
     ((tpl :wat::holon::HolonAST) (:wat::holon::term::template form))
     ((slots :Vec<f64>) (:wat::holon::term::slots tpl))
     ((n :i64) (:wat::core::length slots)))
    (:wat::test::assert-eq n 0)))

;; ─── matches? — same form against itself ─────────────────────────

(:wat::test::deftest :wat-tests::holon::term::test-matches-self
  ()
  (:wat::core::let*
    (((form :wat::holon::HolonAST)
      (:wat::holon::Bind
        (:wat::holon::leaf :rsi-thought)
        (:wat::holon::Thermometer 70.0 0.0 100.0))))
    (:wat::test::assert-eq
      (:wat::holon::term::matches? form form)
      true)))

;; ─── matches? — close-but-not-identical slots within tolerance ────
;;
;; At the default coincident floor (sigma=1, sqrt(d)=100 for d=10000),
;; the per-slot tolerance window is ~1% of the receptive field. A
;; thought at 70.0 against one at 70.5 (0.5% delta on a 100-wide
;; range) sits well inside that.

(:wat::test::deftest :wat-tests::holon::term::test-matches-close-slot
  ()
  (:wat::core::let*
    (((q :wat::holon::HolonAST)
      (:wat::holon::Bind
        (:wat::holon::leaf :rsi-thought)
        (:wat::holon::Thermometer 70.0 0.0 100.0)))
     ((s :wat::holon::HolonAST)
      (:wat::holon::Bind
        (:wat::holon::leaf :rsi-thought)
        (:wat::holon::Thermometer 70.5 0.0 100.0))))
    (:wat::test::assert-eq
      (:wat::holon::term::matches? q s)
      true)))

;; ─── matches? — distant slot exceeds tolerance ───────────────────

(:wat::test::deftest :wat-tests::holon::term::test-matches-distant-slot
  ()
  (:wat::core::let*
    (((q :wat::holon::HolonAST)
      (:wat::holon::Bind
        (:wat::holon::leaf :rsi-thought)
        (:wat::holon::Thermometer 70.0 0.0 100.0)))
     ((s :wat::holon::HolonAST)
      (:wat::holon::Bind
        (:wat::holon::leaf :rsi-thought)
        (:wat::holon::Thermometer 30.0 0.0 100.0))))
    (:wat::test::assert-eq
      (:wat::holon::term::matches? q s)
      false)))

;; ─── matches? — different templates never match ──────────────────
;;
;; Same value, same range, different keyword head → distinct templates;
;; matches? short-circuits to false without even reaching the slot loop.

(:wat::test::deftest :wat-tests::holon::term::test-matches-different-template
  ()
  (:wat::core::let*
    (((q :wat::holon::HolonAST)
      (:wat::holon::Bind
        (:wat::holon::leaf :rsi-thought)
        (:wat::holon::Thermometer 70.0 0.0 100.0)))
     ((s :wat::holon::HolonAST)
      (:wat::holon::Bind
        (:wat::holon::leaf :macd-thought)
        (:wat::holon::Thermometer 70.0 0.0 100.0))))
    (:wat::test::assert-eq
      (:wat::holon::term::matches? q s)
      false)))

;; ─── matches? — template-only forms (no Thermometer) match exactly ──
;;
;; A form with no Thermometer leaves degenerates to a single-template
;; bucket; matches? reduces to "templates equal", which equals
;; structural equality. Two structurally-identical forms with no
;; Thermometers always match.

(:wat::test::deftest :wat-tests::holon::term::test-matches-thermometer-free
  ()
  (:wat::core::let*
    (((q :wat::holon::HolonAST)
      (:wat::holon::Bind
        (:wat::holon::leaf :x)
        (:wat::holon::leaf 42)))
     ((s :wat::holon::HolonAST)
      (:wat::holon::Bind
        (:wat::holon::leaf :x)
        (:wat::holon::leaf 42))))
    (:wat::test::assert-eq
      (:wat::holon::term::matches? q s)
      true)))
