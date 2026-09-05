;; tests/process/wat_arc170_closure6_label_wall_unlabeled.wat — arc 170 closure #6.
;;
;; The negative control for the labeled fixture: NO `with-label` call, so
;; `ProcessOpts/label` stays its default `:None`. Proves argv is UNCHANGED from
;; before this field existed — `[exe]`, nothing appended — same protocol as the
;; labeled sibling (report the child's pid, then block on stdin for the harness).
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::core::let
               [_p (:wat::kernel::println (:wat::program::Env/process-id (:wat::program::env)))
                outcome (:wat::kernel::readln)]
               (:wat::core::match outcome
                 ((:wat::kernel::ReadlnOutcome::Datum _d) nil)
                 (:wat::kernel::ReadlnOutcome::Eof nil)
                 (:wat::kernel::ReadlnOutcome::Stopped nil))))))
     child-pid (:wat::core::match (:wat::kernel::recv p)
                 ((:wat::kernel::RecvOutcome::Message m) m)
                 ((:wat::kernel::RecvOutcome::Lost cause)
                   (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
                 (:wat::kernel::RecvOutcome::Stopped
                   (:wat::kernel::assertion-failed! "unlabeled child: stop requested before sending its pid — child was ALIVE, channel open" :wat::core::None :wat::core::None))
                 (:wat::kernel::RecvOutcome::Closed
                   (:wat::kernel::assertion-failed! "unlabeled child closed before sending its pid" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))
     _ (:wat::kernel::println child-pid)
     release-outcome (:wat::kernel::readln)]
    (:wat::core::match release-outcome
      ((:wat::kernel::ReadlnOutcome::Datum _d) nil)
      (:wat::kernel::ReadlnOutcome::Eof nil)
      (:wat::kernel::ReadlnOutcome::Stopped nil))))
