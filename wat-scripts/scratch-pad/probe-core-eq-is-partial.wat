;; probe-core-eq-is-partial.wat — arc 255 Stone 1c-b-ii, the @Totality counterexample.
;;
;; CLAIM: `:wat::core::=` is @Totality Partial, not Total. `infer_equality` admits a call whenever
;; the two operand types unify, but `values_equal` (src/runtime.rs) has no `Value::Function` arm and
;; falls to its catch-all `_ => None`, which the caller turns into a located TypeMismatch.
;;
;; EXPECTED: `--check` exits 0 (the checker admits it); RUNNING it raises TypeMismatch.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println
      (:wat::core::show
        (:wat::core::= (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 x)
                       (:wat::core::fn [y <- :wat::core::i64] -> :wat::core::i64 y))))
    nil))
