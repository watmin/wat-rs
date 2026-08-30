;; wat-scripts/scratch-pad/282-verify-same-length-mismatch.wat — arc 282, the ORCHESTRATOR's
;; independent control. Row 1's raise fires on an OVERRUNNING claim, which a mere bounds check
;; would also catch. THIS asks the harder question: does the wall fire when the claim is EXACTLY
;; AS LONG as what is there and simply WRONG? If not, every same-length rename is unguarded.
;;   src         = "hello world"
;;   edit        = offset 6, claim "world" — the TRUE claim, new "there"
;;   the source at 6..11 is "world" — same length, different text.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [src   "hello world"
     edits (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
             (:wat::core::Tuple 6 "world" "there"))]
    (:wat::kernel::println (:wat::fix::fix-text-apply src edits))))
