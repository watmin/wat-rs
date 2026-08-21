;; wat-scripts/scratch-pad/arc109-2i-colon-mode-verbatim-probe.wat — arc 109 Stone ②-i.
;;
;; Coordinator reported COLON mode emitting a "third spelling" (:wat.core/Vector instead of
;; :wat::core::Vector) when read through `write-forms`. Traced statically: `write-forms` goes
;; through `watast_to_edn` (Carriage::Display), whose Keyword arm is UNCONDITIONAL —
;; `keyword_from_wat_path` re-spells ANY `::`-keyword to dotted/slashed EDN form, regardless of
;; where it came from. `edn_shim.rs`'s own doc on `ast->source` names this exact symptom:
;; "write-forms on a ::-form emits :wat.core/fn, not :wat::core::fn" — and names `ast->source`
;; as the verbatim tool (`WatAST::Keyword(k,_) => out.push_str(k)`, byte-literal, no translation).
;;
;; This probe prints EACH ladder rung through BOTH printers, COLON mode, side by side, so the
;; DISPLAY-vs-VERBATIM gap is visible directly rather than argued from source reading alone.
;; Predicted (if the rung-1/2/3 Colon-mode code is right, which it should already be — it never
;; routes the actual Colon output through wat_keyword_to_clojure_symbol):
;;
;;   rung   keyword-in                                            ast->source (VERBATIM, correct check)     write-forms (DISPLAY, the misleading one)
;;   1      :wat::core::i64                                       :wat::core::i64                            :wat.core/i64
;;   1 (P)  :wat::core::HashMap<wat::core::String,wat::core::i64>  (:wat::core::HashMap [:wat::core::String :wat::core::i64])   (:wat.core/HashMap [:wat.core/String :wat.core/i64])
;;   2      :i64                                                   :wat::core::i64                            :wat.core/i64
;;   3      :wat::holon::HolonAST                                  :wat::holon::HolonAST                      :wat.holon/HolonAST
;;   4      :T                                                     T                                          T
;;
;; Scratch, per holon/CLAUDE.md's `.wat` scratch convention (not the ephemeral session tmp).

(:wat::core::defn :user::probe-one
  [label <- :wat::core::String kw <- :wat::core::String] -> :wat::core::i64
  (:wat::core::let [node (:wat::core::keyword/to-type-form-colon (:wat::core::keyword-node kw))]
    (:wat::core::do
      (:wat::kernel::println (:wat::core::string::interpolate "{l} verbatim(ast->source) : {v}" :l label :v (:wat::core::ast->source node)))
      (:wat::kernel::println (:wat::core::string::interpolate "{l} display  (write-forms) : {v}" :l label :v (:wat::core::write-forms node)))
      0)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:user::probe-one "rung1-scalar  " ":wat::core::i64")
    (:user::probe-one "rung1-parametr" ":wat::core::HashMap<wat::core::String,wat::core::i64>")
    (:user::probe-one "rung2-bare-prm" ":i64")
    (:user::probe-one "rung3-usertype" ":wat::holon::HolonAST")
    (:user::probe-one "rung4-type-var" ":T")
    nil))
