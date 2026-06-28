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
      (:wat::core::match step-result -> :wat::core::String
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
