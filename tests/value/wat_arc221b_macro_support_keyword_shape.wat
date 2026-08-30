;; tests/value/wat_arc221b_macro_support_keyword_shape.wat — co-located fixture.
;; Slurped via startup_beside(file!()). Each function covers one probe.
;; No :user::main needed — startup_beside loads defns; tests call each fn via eval_in_frozen.

;; ─── Probe 1 — rename-callable-name accepts Keyword first child ──────────────

(:wat::core::defn :t::probe-1-fn [x <- :wat::core::i64] -> :wat::core::i64 x)

(:wat::core::defn :t::probe-1 [] -> :wat::core::String
  (:wat::core::let
    [sig
      (:wat::core::Option/expect
        (:wat::runtime::signature-of-defn :t::probe-1-fn)
        "expected Some for probe-1-fn")
     renamed
      (:wat::runtime::rename-callable-name
        sig
        :t::probe-1-fn
        :t::probe-1-renamed)
     rendered
      (:wat::edn::write renamed)]
    rendered))

;; ─── Probe 2 — rename-callable-name from-mismatch errors ────────────────────
;; This function intentionally errors at runtime (wrong from-name).
;; The Rust test uses eval_in_frozen expecting Err.

(:wat::core::defn :t::probe-2-my-fn [x <- :wat::core::i64] -> :wat::core::i64 x)

(:wat::core::defn :t::probe-2 [] -> :wat::core::nil
  (:wat::core::let
    [sig
      (:wat::core::Option/expect
        (:wat::runtime::signature-of-defn :t::probe-2-my-fn)
        "expected Some")
     _
      (:wat::runtime::rename-callable-name
        sig
        :t::probe-2-wrong-name
        :t::probe-2-alias)]
    nil))

;; ─── Probe 3 — defalias end-to-end (substrate target, Stone 241.12) ─────────

(:wat::core::defalias :t::my-length :wat::core::length)

(:wat::core::defn :t::probe-3 [] -> :wat::core::String
  (:wat::core::let
    [v   (:wat::core::Vector :- [:wat::core::i64] 1 2 3)
     r1  (:wat::core::length v)
     r2  (:t::my-length v)]
    (:wat::string::concat
      (:wat::edn::write r1)
      " "
      (:wat::edn::write r2))))
