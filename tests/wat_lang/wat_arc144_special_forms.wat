;; tests/wat_lang/wat_arc144_special_forms.wat — co-located fixture.
;; Arc 144 slice 2 — special-form registry reflection.
;; Each :t:: function probes lookup-define / signature-of-defn / body-of.
;; Pattern: :t::def-X → String (rendered), :t::sig-X → String, :t::body-X → bool (None→true).

;; ─── :wat::core::if ─────────────────────────────────────────────────────────
(wat.core/defn t/def-if [] :- wat.type/String
  (wat.edn/write (wat.runtime/lookup-define wat.core/if)))
(wat.core/defn t/sig-if [] :- wat.type/String
  (wat.edn/write (wat.runtime/signature-of-defn wat.core/if)))
(wat.core/defn t/body-if [] :- wat.type/bool
  (wat.core/match (wat.runtime/body-of wat.core/if) 
    [wat.core/Option.Some {:value _} false] [wat.core/Option.None {} true]))

;; ─── :wat::core::let ────────────────────────────────────────────────────────
(wat.core/defn t/def-let [] :- wat.type/String
  (wat.edn/write (wat.runtime/lookup-define wat.core/let)))
(wat.core/defn t/sig-let [] :- wat.type/String
  (wat.edn/write (wat.runtime/signature-of-defn wat.core/let)))
(wat.core/defn t/body-let [] :- wat.type/bool
  (wat.core/match (wat.runtime/body-of wat.core/let) 
    [wat.core/Option.Some {:value _} false] [wat.core/Option.None {} true]))

;; ─── :wat::core::fn ─────────────────────────────────────────────────────────
(wat.core/defn t/def-fn [] :- wat.type/String
  (wat.edn/write (wat.runtime/lookup-define wat.core/fn)))
(wat.core/defn t/sig-fn [] :- wat.type/String
  (wat.edn/write (wat.runtime/signature-of-defn wat.core/fn)))
(wat.core/defn t/body-fn [] :- wat.type/bool
  (wat.core/match (wat.runtime/body-of wat.core/fn) 
    [wat.core/Option.Some {:value _} false] [wat.core/Option.None {} true]))

;; ─── :wat::core::match ──────────────────────────────────────────────────────
(wat.core/defn t/def-match [] :- wat.type/String
  (wat.edn/write (wat.runtime/lookup-define wat.core/match)))
(wat.core/defn t/sig-match [] :- wat.type/String
  (wat.edn/write (wat.runtime/signature-of-defn wat.core/match)))
(wat.core/defn t/body-match [] :- wat.type/bool
  (wat.core/match (wat.runtime/body-of wat.core/match) 
    [wat.core/Option.Some {:value _} false] [wat.core/Option.None {} true]))

;; ─── :wat::core::quasiquote ─────────────────────────────────────────────────
(wat.core/defn t/def-quasiquote [] :- wat.type/String
  (wat.edn/write (wat.runtime/lookup-define wat.core/quasiquote)))
(wat.core/defn t/sig-quasiquote [] :- wat.type/String
  (wat.edn/write (wat.runtime/signature-of-defn wat.core/quasiquote)))
(wat.core/defn t/body-quasiquote [] :- wat.type/bool
  (wat.core/match (wat.runtime/body-of wat.core/quasiquote) 
    [wat.core/Option.Some {:value _} false] [wat.core/Option.None {} true]))

;; ─── :wat::core::defstruct ──────────────────────────────────────────────────
(wat.core/defn t/def-defstruct [] :- wat.type/String
  (wat.edn/write (wat.runtime/lookup-define wat.core/defstruct)))

;; ─── Unknown: :wat::core::not-a-special-form — all three return None ─────────
(wat.core/defn t/all-none-not-a-sf [] :- wat.type/bool
  (wat.core/let
    [d-opt (wat.runtime/lookup-define wat.core/not-a-special-form)
     s-opt (wat.runtime/signature-of-defn wat.core/not-a-special-form)
     b-opt (wat.runtime/body-of wat.core/not-a-special-form)]
    (wat.core/match d-opt 
      [wat.core/Option.Some {:value _} false]
      [wat.core/Option.None {}
        (wat.core/match s-opt 
          [wat.core/Option.Some {:value _} false]
          [wat.core/Option.None {}
            (wat.core/match b-opt 
              [wat.core/Option.Some {:value _} false]
              [wat.core/Option.None {} true])])])))
