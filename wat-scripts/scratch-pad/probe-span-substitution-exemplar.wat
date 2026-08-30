;; probe-span-substitution-exemplar.wat — the GOLDEN EXEMPLAR for the span-substitution fleet.
;;
;; `eval_vec_foldl` (src/collection/transform.rs:419) takes `call_span: &Span` — the location of
;; the user's own `(foldl …)` — and then invokes the user's fold function with
;; `apply_function(…, crate::rust_caller_span!())` at :454/:461/:468. So the frame pushed for the
;; user's function is stamped with a line in transform.rs, and anything that function raises is
;; reported as living in the Rust source.
;;
;; This probe makes that user-visible: a fold function that fails, called from a `(foldl …)` whose
;; line is known. Run it and read the reported :location / :frames.
;;
;;   BEFORE the fix -> src/collection/transform.rs:<one of 454/461/468>
;;   AFTER  the fix -> THIS file, at the `(foldl …)` line below
;;
;; That difference IS the exemplar every rider copies: a real span was in scope, a Rust one was
;; minted, and the user was pointed at the interpreter instead of their own program.

(:wat::core::defn :probe::boom
  [acc <- :wat::core::i64
   x   <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::kernel::assertion-failed! "boom inside the fold"
    :wat::core::None :wat::core::None))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [r (:wat::core::foldl :probe::boom 0 (:wat::core::Vector :- [:wat::core::i64] 1 2 3))] ;; <<foldl-call>>
    (:wat::kernel::println r)))
