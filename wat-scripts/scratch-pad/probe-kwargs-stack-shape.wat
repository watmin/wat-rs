;; probe-kwargs-stack-shape.wat — what is the FULL stack under a kwargs call?
;;
;; `call-site` returns only frames[0], which for a kwargs fn is the synthesized
;; `<name>$impl` adapter at wat/core.wat:649 (measured: probe-call-site-kwargs.wat). The
;; question: is the frame we actually want — the one whose span is the USER's call line —
;; present in the stack, and if so at what depth?
;;
;; ⚠ FIRST VERSION OF THIS PROBE WAS WRONG, kept as the lesson: every call in it sat in
;; TAIL POSITION, and wat has proper TCO, so the frames collapsed into ONE and the stack
;; looked one-deep. That is TCO being measured, not stack shape. Every call below is
;; deliberately NON-TAIL (bound in a `let`, then a separate return) — which is also the real
;; shape of the site this is for: `[locus (svc/start ...)]` inside a `let`.
;;
;; wat exposes no whole-stack verb (that IS the gap). `assertion-failed!` captures the full
;; stack into its payload and the panic hook prints it, so: fail an assertion from inside a
;; kwargs fn and READ the frames.

(:wat::core::defn :probe::kw [& [tag <- :wat::core::String]] -> :wat::core::i64
  (:wat::core::let
    [_boom (:wat::kernel::assertion-failed! "deliberate — dumping the stack shape"
             :wat::core::None :wat::core::None)]
    0))

;; NON-tail: bind the call's result, then return something else.
(:wat::core::defn :probe::middle [] -> :wat::core::i64
  (:wat::core::let
    [r (:probe::kw :tag "x")]
    (:wat::i64::+ r 1)))

(:wat::core::defn :probe::outer [] -> :wat::core::i64
  (:wat::core::let
    [r (:probe::middle)]
    (:wat::i64::+ r 1)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [r (:probe::outer)]
    (:wat::kernel::println r)))
