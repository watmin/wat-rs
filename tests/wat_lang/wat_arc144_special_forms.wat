;; tests/wat_lang/wat_arc144_special_forms.wat — co-located fixture.
;; Arc 144 slice 2 — special-form registry reflection.
;; Each :t:: function probes lookup-define / signature-of-defn / body-of.
;; Pattern: :t::def-X → String (rendered), :t::sig-X → String, :t::body-X → bool (None→true).

;; ─── :wat::core::if ─────────────────────────────────────────────────────────
(:wat::core::defn :t::def-if [] -> :wat::core::String
  (:wat::edn::write (:wat::runtime::lookup-define :wat::core::if)))
(:wat::core::defn :t::sig-if [] -> :wat::core::String
  (:wat::edn::write (:wat::runtime::signature-of-defn :wat::core::if)))
(:wat::core::defn :t::body-if [] -> :wat::core::bool
  (:wat::core::match (:wat::runtime::body-of :wat::core::if) 
    ((:wat::core::Some _) false) (:wat::core::None true)))

;; ─── :wat::core::let ────────────────────────────────────────────────────────
(:wat::core::defn :t::def-let [] -> :wat::core::String
  (:wat::edn::write (:wat::runtime::lookup-define :wat::core::let)))
(:wat::core::defn :t::sig-let [] -> :wat::core::String
  (:wat::edn::write (:wat::runtime::signature-of-defn :wat::core::let)))
(:wat::core::defn :t::body-let [] -> :wat::core::bool
  (:wat::core::match (:wat::runtime::body-of :wat::core::let) 
    ((:wat::core::Some _) false) (:wat::core::None true)))

;; ─── :wat::core::fn ─────────────────────────────────────────────────────────
(:wat::core::defn :t::def-fn [] -> :wat::core::String
  (:wat::edn::write (:wat::runtime::lookup-define :wat::core::fn)))
(:wat::core::defn :t::sig-fn [] -> :wat::core::String
  (:wat::edn::write (:wat::runtime::signature-of-defn :wat::core::fn)))
(:wat::core::defn :t::body-fn [] -> :wat::core::bool
  (:wat::core::match (:wat::runtime::body-of :wat::core::fn) 
    ((:wat::core::Some _) false) (:wat::core::None true)))

;; ─── :wat::core::match ──────────────────────────────────────────────────────
(:wat::core::defn :t::def-match [] -> :wat::core::String
  (:wat::edn::write (:wat::runtime::lookup-define :wat::core::match)))
(:wat::core::defn :t::sig-match [] -> :wat::core::String
  (:wat::edn::write (:wat::runtime::signature-of-defn :wat::core::match)))
(:wat::core::defn :t::body-match [] -> :wat::core::bool
  (:wat::core::match (:wat::runtime::body-of :wat::core::match) 
    ((:wat::core::Some _) false) (:wat::core::None true)))

;; ─── :wat::core::quasiquote ─────────────────────────────────────────────────
(:wat::core::defn :t::def-quasiquote [] -> :wat::core::String
  (:wat::edn::write (:wat::runtime::lookup-define :wat::core::quasiquote)))
(:wat::core::defn :t::sig-quasiquote [] -> :wat::core::String
  (:wat::edn::write (:wat::runtime::signature-of-defn :wat::core::quasiquote)))
(:wat::core::defn :t::body-quasiquote [] -> :wat::core::bool
  (:wat::core::match (:wat::runtime::body-of :wat::core::quasiquote) 
    ((:wat::core::Some _) false) (:wat::core::None true)))

;; ─── :wat::core::defstruct ──────────────────────────────────────────────────
(:wat::core::defn :t::def-defstruct [] -> :wat::core::String
  (:wat::edn::write (:wat::runtime::lookup-define :wat::core::defstruct)))

;; ─── :wat::kernel::spawn ────────────────────────────────────────────────────
(:wat::core::defn :t::def-spawn [] -> :wat::core::String
  (:wat::edn::write (:wat::runtime::lookup-define :wat::kernel::spawn)))
(:wat::core::defn :t::sig-spawn [] -> :wat::core::String
  (:wat::edn::write (:wat::runtime::signature-of-defn :wat::kernel::spawn)))
(:wat::core::defn :t::body-spawn [] -> :wat::core::bool
  (:wat::core::match (:wat::runtime::body-of :wat::kernel::spawn) 
    ((:wat::core::Some _) false) (:wat::core::None true)))

;; ─── Unknown: :wat::core::not-a-special-form — all three return None ─────────
(:wat::core::defn :t::all-none-not-a-sf [] -> :wat::core::bool
  (:wat::core::let
    [d-opt (:wat::runtime::lookup-define :wat::core::not-a-special-form)
     s-opt (:wat::runtime::signature-of-defn :wat::core::not-a-special-form)
     b-opt (:wat::runtime::body-of :wat::core::not-a-special-form)]
    (:wat::core::match d-opt 
      ((:wat::core::Some _) false)
      (:wat::core::None
        (:wat::core::match s-opt 
          ((:wat::core::Some _) false)
          (:wat::core::None
            (:wat::core::match b-opt 
              ((:wat::core::Some _) false)
              (:wat::core::None true))))))))
