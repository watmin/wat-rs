;; Does a generic fn substitute its type param I into a `forms` quote when called concretely?
;; CLAIM (measured, decisive NO): calling (probe::mk 5) does NOT substitute the caller's
;; concrete type (i64) into the quoted inner defn — both the arg type and the return type
;; keyword nodes in the emitted form are still the literal, un-substituted keyword `:I`.
;; `forms` is a reflective AST quote, not a monomorphizing template.
;;
;; ⚠ THIS PINS A KNOWN GAP, not a settled design. If generic substitution into `forms` quotes
;; is ever implemented, this probe's assertions WILL go red (expecting `:I`, finding `:i64`) —
;; that red is the signal to come rewrite this probe's expected node (and its CLAIM line) to
;; match the new, intentional behavior, not to re-widen the assertion or delete the probe.
;;
;; Asserted as AST-NODE equality (`wat.type/AST` extends Equatable; `WatAST`'s PartialEq
;; compares structure, skipping span — `crates/wat-reader/src/ast.rs`) against a node built by
;; `:wat::core::keyword-node`, never as an ast-kind/ast-name STRING pair standing in for the
;; node (coordinator correction, 255.76 weigh: "measuring strings is anti-wat").
(wat.core/defn probe/mk :- [I]
  [x :- I] :- (wat.type/Vector :- [wat.type/AST])
  (wat.core/forms
    (wat.core/defn probe/inner [a :- I] :- I a)))

(wat.core/defn user/main [] :- wat.type/nil
  (wat.core/let
    [forms     (probe/mk 5)
     defn-node (wat.core/nth forms 0)
     ch        (wat.core/ast->children defn-node)
     ;; ch = [defn-kw, name-kw, argspec-vec, ->-sym, ret-type-kw, body]
     argspec   (wat.core/nth ch 2)
     arg-ch    (wat.core/ast->children argspec)
     ;; arg-ch = [a-sym, <--sym, I-kw] (one param, flat)
     arg-ty    (wat.core/nth arg-ch 2)
     ret-ty    (wat.core/nth ch 4)
     expected  (wat.core/keyword-node ":I")]
    (wat.core/do
      (wat.test/assert-eq arg-ty expected)
      (wat.test/assert-eq ret-ty expected)
      (wat.kernel/println (wat.edn/write forms)))))
