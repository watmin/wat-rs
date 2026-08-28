;; wat-scripts/scratch-pad/255-stone-o-iv-d-unevaluated-args-check.wat — arc 255 Stone
;; O-iv-d, row 0 evidence. Empirically checks whether
;; `:wat::intrinsic::variadic-args-measurement` evaluates its arguments. If it does NOT
;; (per its own doc: "evaluates none of them"), passing an erroring expression as an
;; argument must NOT raise — proving the UNEVALUATED-ARGS disqualifier applies, same shape
;; as O-iv-c-2's `:wat::holon::literal` finding.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::string::concat "count with an erroring arg (unevaluated if this prints, not crashes): "
      (:wat::i64::to-string
        (:wat::intrinsic::variadic-args-measurement (:wat::i64::/ 1 0) 2 3)))))
