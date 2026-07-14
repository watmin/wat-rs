;; tests/rete/probe_arc300_2_fix_defrule.wat — arc 300.2: the fix conversion as PURE rete defrules.
;;
;; rete is ALWAYS pure in wat: rules DEDUCE, the deductions are QUERIED OUT and ACTIONED
;; outside rete. A :then NEVER transforms a value — it inserts a classification fact whose
;; fields are only ?var bindings (offset/len/name), which the v1 RHS resolver already handles.
;;
;; The transformation (keyword/to-symbol, keyword/to-type-form, ":-") and the I/O live in the
;; DRIVE (wat-scripts/fixes/to-faithful-clojure-rete.wat), OUTSIDE rete — the consumer's job.
;;
;; This fixture is the rete-firing unit test: assert :fix::Node facts, fire, and confirm the
;; three PURE classification facts (:fix::HeadConv / :fix::ArrowConv / :fix::TypeConv) deduce.

;; ── fact model ──────────────────────────────────────────────────────────────
;; Node — one per leaf AST node the walk visits (position-aware: post-arrow tracked).
(:wat::core::defrecord :fix::Node
  [kind       <- :wat::core::String
   name       <- :wat::core::String
   offset     <- :wat::core::i64
   len        <- :wat::core::i64
   post-arrow <- :wat::core::bool])

;; The three PURE classification facts — offset/len (+ name where the drive needs it to
;; reconstruct the keyword). No transformed text: the drive does keyword/to-symbol etc.
(:wat::core::defrecord :fix::HeadConv
  [offset <- :wat::core::i64
   len    <- :wat::core::i64
   name   <- :wat::core::String])

(:wat::core::defrecord :fix::ArrowConv
  [offset <- :wat::core::i64
   len    <- :wat::core::i64])

(:wat::core::defrecord :fix::TypeConv
  [offset <- :wat::core::i64
   len    <- :wat::core::i64
   name   <- :wat::core::String])

;; ── pure string predicates (used in :where guards) ──────────────────────────
;; head-keyword-str? — name string is ::-namespaced.
(:wat::core::defn :fix::head-keyword-str?
  [name <- :wat::core::String] -> :wat::core::bool
  (:wat::core::string::contains? name "::"))

;; type-shaped-keyword-str? — name has matching "<" + ">" OR "(" + ")".
(:wat::core::defn :fix::type-shaped-keyword-str?
  [name <- :wat::core::String] -> :wat::core::bool
  (:wat::core::if (:wat::core::if (:wat::core::string::contains? name "<")
                    (:wat::core::string::contains? name ">")
                    false)
    true
    (:wat::core::if (:wat::core::string::contains? name "(")
      (:wat::core::string::contains? name ")")
      false)))

;; ── the rules: each :then is PURE (bindings only, no transform) ──────────────

;; head-keyword→conv: kind=keyword ∧ contains "::" ∧ ¬post-arrow ∧ ¬type-shaped
;;   → deduce HeadConv(offset, len, name). The drive turns name into (keyword/to-symbol name).
(:wat::rete::defrule :fix::head-keyword->conv
  :when
  [(:fix::Node
     (?offset     <- :offset)
     (?len        <- :len)
     (?kind       <- :kind)
     (?name       <- :name)
     (?post-arrow <- :post-arrow)
     (:wat::core::= ?kind "keyword"))
   (:wat::rete::where (:fix::head-keyword-str? ?name))
   (:wat::rete::where (:wat::core::not ?post-arrow))
   (:wat::rete::where (:wat::core::not (:fix::type-shaped-keyword-str? ?name)))]
  :then
  (:wat::rete::insert (:fix::HeadConv :offset ?offset :len ?len :name ?name)))

;; arrow→conv: kind=symbol ∧ (name="<-" ∨ name="->") → deduce ArrowConv(offset, len).
;;   The drive emits the literal ":-".
(:wat::rete::defrule :fix::arrow->conv
  :when
  [(:fix::Node
     (?offset <- :offset)
     (?len    <- :len)
     (?kind   <- :kind)
     (?name   <- :name)
     (:wat::core::= ?kind "symbol"))
   (:wat::rete::where (:wat::core::or
                        (:wat::core::= ?name "<-")
                        (:wat::core::= ?name "->")))]
  :then
  (:wat::rete::insert (:fix::ArrowConv :offset ?offset :len ?len)))

;; type-keyword→conv: kind=keyword ∧ (post-arrow ∨ type-shaped)
;;   → deduce TypeConv(offset, len, name). The drive turns name into (keyword/to-type-form name).
(:wat::rete::defrule :fix::type-keyword->conv
  :when
  [(:fix::Node
     (?offset     <- :offset)
     (?len        <- :len)
     (?kind       <- :kind)
     (?name       <- :name)
     (?post-arrow <- :post-arrow)
     (:wat::core::= ?kind "keyword"))
   (:wat::rete::where (:wat::core::or
                        ?post-arrow
                        (:fix::type-shaped-keyword-str? ?name)))]
  :then
  (:wat::rete::insert (:fix::TypeConv :offset ?offset :len ?len :name ?name)))

