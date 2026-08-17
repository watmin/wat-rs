;; tests/value/wat_arc221b_keyword_dispatcher_completeness.wat — co-located fixture.
;; Slurped via startup_beside(file!()). Each function covers one probe.
;; Functions return String (the EDN output) or bool so Rust can assert without stdout capture.

;; ─── Probe 1 — watast_to_holon Keyword arm ───────────────────────────────────

(:wat::core::defn :t::probe-1 [] -> :wat::core::String
  (:wat::core::let
    [h   (:wat::holon::from-wat (:wat::core::quote :foo))
     edn (:wat::edn::write h)]
    edn))

;; ─── Probe 2 — :wat::holon::leaf Keyword arm ─────────────────────────────────

(:wat::core::defn :t::probe-2 [] -> :wat::core::String
  (:wat::core::let
    [h   (:wat::holon::leaf :user::foo)
     edn (:wat::edn::write h)]
    edn))

;; ─── Probe 3a — eval-step! AlreadyTerminal Keyword ───────────────────────────

(:wat::core::defn :t::probe-3a [] -> :wat::core::String
  (:wat::core::let
    [step-result
      (:wat::eval-step! (:wat::core::quote :outcome))
     rendered
      (:wat::core::match step-result 
        ((:wat::core::Ok r) (:wat::core::show r))
        ((:wat::core::Err e) (:wat::core::show e)))]
    rendered))

;; ─── Probe 3b — from-wat(quote :outcome) identity equality ───────────────────

(:wat::core::defn :t::probe-3b [] -> :wat::core::String
  (:wat::core::let
    [h1  (:wat::holon::from-wat (:wat::core::quote :outcome))
     h2  (:wat::holon::from-wat (:wat::core::quote :outcome))
     eq  (:wat::core::= h1 h2)]
    (:wat::edn::write eq)))

;; ─── Probe 4 — EDN keyword wire format ───────────────────────────────────────

(:wat::core::defn :t::probe-4 [] -> :wat::core::String
  (:wat::core::let
    [h   (:wat::holon::leaf :bar)
     edn (:wat::edn::write h)]
    edn))

;; ─── Probe 5 — Value::Unit consistency / nil leaf ────────────────────────────

(:wat::core::defn :t::probe-5 [] -> :wat::core::String
  (:wat::core::let
    [h   (:wat::holon::leaf nil)
     edn (:wat::edn::write h)]
    edn))

;; ─── Probe 6 — watast_to_holon keyword distinct identities ───────────────────

(:wat::core::defn :t::probe-6 [] -> :wat::core::String
  (:wat::core::let
    [h1  (:wat::holon::from-wat (:wat::core::quote :foo))
     h2  (:wat::holon::from-wat (:wat::core::quote :bar))
     eq  (:wat::core::= h1 h2)]
    (:wat::edn::write (:wat::core::not eq))))

;; ─── Arc 294.j — VARIANT DISCRIMINATORS (the instrument, repaired) ────────────
;;
;; ⛔ THE WIRE CAN NO LONGER TELL Keyword FROM Symbol. Measured 2026-08-16: both
;; `(:wat::holon::leaf :foo)` and `(:wat::holon::from-wat (:wat::core::quote foo))`
;; render `#wat/holon :foo`, because `from_holon_item`'s Symbol arm maps a Symbol
;; composition to a keyword Value (runtime.rs:16646 — a comment describing the
;; PRE-arc-221 world, where Symbol carried colon-prefixed keywords).
;;
;; That is RULED NOT NOW (task #103): wat has no proper symbols yet and the grind
;; is toward them; patching the symbol arm would muck with a surface about to be
;; rebuilt, and the target classifier shape is different anyway
;; (`wat.type/keyword` / `wat.type/symbol`, per the builder).
;;
;; ★ SO THE PROBES' GOLDENS WERE NOT REGENERATED IN PLACE. Probes 1/2/5 exist to
;; prove a claim about the HolonAST VARIANT ("emits Keyword, NOT Symbol" — arc 221
;; doctrine); wire text was only ever the INSTRUMENT for that claim, and the
;; instrument went blind. Regenerating alone would have left four tests passing
;; while proving nothing. These fns assert the CLAIM directly; the wire goldens
;; stay alongside as encoding regression guards.
;; `[[feedback_ask_what_a_test_measures_before_fixing_how_it_measures]]`

(:wat::core::defn :t::probe-1-is-keyword [] -> :wat::core::String
  (:wat::edn::write
    (:wat::holon::is-Keyword? (:wat::holon::from-wat (:wat::core::quote :foo)))))

(:wat::core::defn :t::probe-1-is-symbol [] -> :wat::core::String
  (:wat::edn::write
    (:wat::holon::is-Symbol? (:wat::holon::from-wat (:wat::core::quote :foo)))))

(:wat::core::defn :t::probe-2-is-keyword [] -> :wat::core::String
  (:wat::edn::write (:wat::holon::is-Keyword? (:wat::holon::leaf :user::foo))))

(:wat::core::defn :t::probe-5-is-nil [] -> :wat::core::String
  (:wat::edn::write (:wat::holon::is-Nil? (:wat::holon::leaf nil))))
