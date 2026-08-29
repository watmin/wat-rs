;; arc 255 Stone P6-c-1 — reachability + correctness probe for the two verbs homed to
;; `#[wat_intrinsic]` (`:wat::form::matches?` — the ORDER shape; `:wat::program::cpu-count`
;; — the SUBSET shape). Both used to be hand-written match arms in `dispatch_keyword_head`;
;; both are now dispatched through the `IntrinsicRegistry`. This file just proves each is
;; still reachable and still correct post-move — no signature was touched on either callee.
;;
;;   ./target/release/wat wat-scripts/scratch-pad/255-p6c1-two-verbs-homed.wat   # EXIT=0

(:wat::core::defstruct :probe::P6c1Subject [amount <- :wat::core::i64])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    ;; cpu-count: nullary (SUBSET-shape context tail — span only), returns a positive i64.
    (:wat::test::assert-eq (:wat::i64::> (:wat::program::cpu-count) 0) true)

    ;; form::matches?: ORDER-shape context tail (span, env, sym). A matching subject.
    (:wat::test::assert-eq
      (:wat::form::matches? (:probe::P6c1Subject :amount 3)
        (:probe::P6c1Subject (= ?a :amount) (= ?a 3)))
      true)

    ;; and a non-matching subject — false, not an error (Clara semantics preserved).
    (:wat::test::assert-eq
      (:wat::form::matches? (:probe::P6c1Subject :amount 3)
        (:probe::P6c1Subject (= ?a :amount) (= ?a 4)))
      false)))
