;; probe-bracket-closure-seam.wat — DISCONFIRMING probe #2 for "brackets done right".
;;
;; CLAIM under test (the remote-ready ergonomic seam): the caller passes a RUNTIME WORK-FN
;; CLOSURE (a Peer'-safe / pure value), and the NOT-SHARED path closure-extracts it and ships
;; it over the byte channel — so the caller writes a closure exactly like the thread bracket
;; today, and the same code goes process (local pipe) or, later, remote (socket) unchanged.
;;
;; This tests closure-extraction of a CAPTURED work-fn (not hand-written forms) — the bracket's
;; distinction from defservice (whose :impls are macro-authored). `spawn-process` is the
;; wat-callable closure path (auto-extracts via closure_extract; ImpureCapture = the EDN gate).
;;
;; EXPECT (if the closure seam holds): "6 10". If it stops, the checker/runtime names the gap.

;; a runner that CAPTURES a runtime work-fn value and streams it — generic over the work.
(:wat::core::defn :probe::runner
  [self <- :wat::kernel::Peer'<wat::core::i64,wat::core::i64>
   work <- :wat::core::Fn(wat::core::i64)->wat::core::i64] -> :wat::core::nil
  (:wat::core::let
    [item (:wat::kernel::recv' self)
     _    (:wat::kernel::send' self (work item))]
    (:probe::runner self work)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [;; the work-fn as a RUNTIME closure value (the caller's job, like bracket::map's work-fn)
     work (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::* x 2))
     ;; ship a runner CLOSURE that CAPTURES `work` to a not-shared worker; closure-extraction
     ;; must carry `work` across the byte channel (the "can this be EDN?" viability path).
     w (:wat::kernel::spawn-process
         (:wat::core::fn [self <- :wat::kernel::Peer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
           (:probe::runner self work)))
     _ (:wat::kernel::send' w 3)
     _ (:wat::kernel::send' w 5)
     a (:wat::kernel::recv' w)
     b (:wat::kernel::recv' w)]
    (:wat::kernel::println
      (:wat::core::string::concat
        (:wat::core::i64::to-string a)
        (:wat::core::string::concat " " (:wat::core::i64::to-string b))))))
