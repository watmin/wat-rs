;; wat-scripts/scratch-pad/arc109-tuple-arm-faults.wat — arc 109 Stone ②-iii's blocker.
;;
;; The `TypeExpr::Tuple` arm of `type_expr_to_clojure_form` (src/edn_shim.rs) is the ONE arm
;; ②-i left un-migrated: it is mode-blind (always the `wat.type/Tuple` Symbol head) AND it
;; splices its items FLAT (`(wat.type/Tuple a b)`) instead of bracketing them.
;;
;; It is reached far more often than "tuples in the corpus" suggests, because
;; `parse_type_expr` CANONICALIZES `:wat::core::nil` into `TypeExpr::Tuple(vec![])`
;; (src/types.rs:4728) — so EVERY `-> :wat::core::nil` annotation routes through this arm.
;;
;; Builder's ruling, 2026-08-20: *"nil is rust's unit ... but `nil != ()` in wat. nil is not an
;; empty list. `(wat.type/Tuple)` is illegal, it'd be `(wat.type/Tuple [])` to be an empty tuple."*
;;
;; So the fix has two halves and this probe measures both:
;;   (a) `nil` must render back as `:wat::core::nil`, NOT as any Tuple form  → rows 1, 2
;;   (b) a real tuple must BRACKET its members, and honour the COLON head    → rows 3, 4
;;
;; Read through `ast->source` (byte-verbatim), NEVER `write-forms` (Carriage::Display re-spells
;; every `::`-keyword to EDN-dotted form and would hide the very thing under test).

(:wat::core::defn :user::show
  [label <- :wat::core::String kw <- :wat::core::String] -> :wat::core::nil
  (:wat::core::let [node (:wat::core::keyword/to-type-form-colon (:wat::core::keyword-node kw))]
    (:wat::kernel::println
      (:wat::core::string::interpolate "{l} : {v}" :l label :v (:wat::core::ast->source node)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:user::show "1 nil bare      " ":wat::core::nil")
    (:user::show "2 nil nested    " ":wat::core::Result<wat::core::nil,wat::core::String>")
    (:user::show "3 tuple 3-ary   " ":(wat::core::i64,wat::core::i64,wat::core::String)")
    (:user::show "4 tuple empty   " ":()")
    (:user::show "5 control parm  " ":wat::core::Vector<wat::core::i64>")
    nil))
