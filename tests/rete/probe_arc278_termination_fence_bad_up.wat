;; ⛔ THE SOUNDNESS TWIN — `k+1` while `k > 500` DIVERGES and must stay REFUSED.
;;
;; It is ONE CHARACTER from its terminating sibling (`<` vs `>`), and that character is the
;; entire proof. The rule fires only while `k > 500` and produces `k + 1`, which is also
;; `> 500` — it satisfies its own guard forever. **A fence is not evidence of termination; a
;; fence pointing AGAINST the step is.** Without this fixture the measure analysis could be
;; written to accept any fence at all and every other row would still pass.
;;
;; Arc 278 item 8 — the termination verifier reads the `where` fence.
(:wat::core::defrecord :b8::N [k <- :wat::core::i64])

(:wat::rete::defrule :b8::bad-up
  :when
  [(:b8::N (?k <- :k))
   (:wat::rete::where (:wat::rete::core::i64::> ?k 500))]
  :then
  [(:b8::N :k (:wat::rete::core::i64::+ ?k 1 :undefined 0))])

(:wat::rete::defquery :b8::q :params [] :when [(?fact <- :b8::N)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::match
    (:wat::rete::compile-all (:wat::rete::collect-rules :b8) (:wat::core::PersistentVector (:b8::q)))
    ((:wat::rete::CompileOutcome::Compiled __s) (:wat::kernel::println "ADMITTED"))
    ((:wat::rete::CompileOutcome::MayNotTerminate rule fact-type)
      ;; Same vocabulary as every other converted fixture in this suite — the arm NAME first, then
      ;; its fields — so one reader (`arm_str`) serves them all.
      (:wat::core::do
        (:wat::kernel::println "ARM MayNotTerminate")
        (:wat::kernel::println rule)
        (:wat::kernel::println fact-type)))))
