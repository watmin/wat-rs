;; print the fn-forms output shape for a concrete named work-fn — to see where the
;; arg-type + return-type keywords live (for the parent-side AST-splice).
;; CLAIM: fn-forms on a closure capturing a named fn ships exactly 2 forms (the hoisted defn
;; + the bracket-target def alias), and return-type-of correctly reports the fn's return type.
;; `return-type-of` is String-typed BY API (its declared `@ret` is `:wat::core::String`, a
;; colon-free FQDN — see src/reflect/verbs.rs) — there is no non-string representation of its
;; result to compare against. Per the coordinator's correction (255.76 weigh), the fix is not
;; to pin a hand-typed text literal but to compare what the string DENOTES: `(:wat::core::type
;; <value>)` uses the SAME FQDN convention (the doc's own words: "directly comparable"), so
;; asserting `rt == (type (work-fn 5))` checks that return-type-of's declared-type report
;; agrees with the ACTUAL runtime type of a real i64 the fn produces — two independent live
;; computations agreeing, never a literal I chose.
(wat.core/defn my/double [n :- wat.type/i64] :- wat.type/i64 (wat.i64/* n 2))
(wat.core/defn user/main [] :- wat.type/nil
  (wat.core/let
    [work-fn (wat.core/fn [n :- wat.type/i64] :- wat.type/i64 (my/double n))
     forms   (wat.kernel/fn-forms work-fn bracket/__pool-work)
     rt      (wat.runtime/return-type-of work-fn)
     denoted (wat.core/type (work-fn 5))]
    (wat.core/do
      (wat.test/assert-eq (wat.core/length forms) 2)
      (wat.test/assert-eq rt denoted)
      (wat.kernel/println (wat.edn/write forms))
      (wat.kernel/println (wat.string/concat "return-type-of = " rt)))))
