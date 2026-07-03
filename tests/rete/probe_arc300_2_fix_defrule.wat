;; tests/rete/probe_arc300_2_fix_defrule.wat — arc 300.2 probe: the fix conversion as rete defrules.
;;
;; Fact model: one record per AST node; one edit record per replacement.
;; Explores whether rete defrules can express the faithful-Clojure conversion from fix.wat.
;;
;; FINDINGS (summarised; full analysis in probe_arc300_2_fix_defrule.rs):
;;
;;   ✓ Conditions: rete alpha-match + :where CAN express all required predicates:
;;       - inline (:wat::core::= ?kind "symbol") for type equality
;;       - (:wat::rete::where ...) for string::contains?, not, or — all pure+det
;;   ✓ arrow→:- rule: FULLY EXPRESSIBLE — literal ":-" in the :then RHS.
;;   ✗ head-keyword→symbol + keyword→type-form: STOP-1.
;;       The RHS uses build_insert_fact whose resolve_operand handles ONLY
;;       ?var / :field / literals. Nested expressions like
;;       (:wat::core::keyword/to-symbol ?name) return None → RuntimeError at fire.
;;       Furthermore, keyword/to-symbol requires a WatAST::Keyword node, not a String,
;;       so even if arbitrary exprs were supported the type would mismatch.
;;       With the spec'd Node fact model (name is a String), the converted symbol text
;;       cannot be produced in the :then.  The arrow→:- rule IS the upper bound of what
;;       the v1 engine can express for this conversion.

(:wat::core::defrecord :fix::Node
  [kind       <- :wat::core::String
   name       <- :wat::core::String
   offset     <- :wat::core::i64
   len        <- :wat::core::i64
   post-arrow <- :wat::core::bool])

(:wat::core::defrecord :fix::Edit
  [offset <- :wat::core::i64
   len    <- :wat::core::i64
   text   <- :wat::core::String])

;; head-keyword-str? — true if name string contains "::" (is ::-namespaced).
(:wat::core::defn :fix::head-keyword-str?
  [name <- :wat::core::String] -> :wat::core::bool
  (:wat::core::string::contains? name "::"))

;; type-shaped-keyword-str? — true if name contains "<" + ">" OR "(" + ")".
(:wat::core::defn :fix::type-shaped-keyword-str?
  [name <- :wat::core::String] -> :wat::core::bool
  (:wat::core::if (:wat::core::if (:wat::core::string::contains? name "<")
                    (:wat::core::string::contains? name ">")
                    false)
    true
    (:wat::core::if (:wat::core::string::contains? name "(")
      (:wat::core::string::contains? name ")")
      false)))

;; ── Rule (1): arrow→:- — FULLY EXPRESSIBLE ──────────────────────────────────
;; kind="symbol" ∧ (name="<-" ∨ name="->") → Edit(offset, len, ":-")
;; The literal ":-" in :then is a StringLit — resolve_operand handles it directly.
(:wat::rete::defrule :fix::arrow->colon
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
  (:wat::rete::insert (:fix::Edit ?offset ?len ":-")))

;; ── Rule (2): head-keyword→symbol — STOP-1 DOCUMENTED ──────────────────────
;; kind="keyword" ∧ name has "::" ∧ ¬post-arrow ∧ ¬type-shaped → Edit(offset, len, ???)
;;
;; CONDITIONS: fully expressible via :where (demonstrated below).
;; RHS: STOP-1. The desired text is (keyword/to-symbol name) = e.g. "wat.core/defrecord",
;; but resolve_operand cannot evaluate a nested call expression. Using ?name in the text
;; position produces the RAW keyword string ":wat::core::defrecord" — wrong, but proves
;; that the CONDITIONS fire correctly. The converted text is unreachable from a String
;; binding via the v1 RHS mechanism.
(:wat::rete::defrule :fix::head-keyword->symbol
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
  ;; STOP-1: ?name is ":wat::core::defrecord" (raw), not "wat.core/defrecord" (wanted).
  ;; There is no v1 mechanism to compute keyword/to-symbol in the RHS.
  (:wat::rete::insert (:fix::Edit ?offset ?len ?name)))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
