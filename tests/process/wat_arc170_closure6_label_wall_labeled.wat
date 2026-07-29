;; tests/process/wat_arc170_closure6_label_wall_labeled.wat — arc 170 closure #6.
;;
;; Outer driver, run as its own `wat` subprocess (never in-process — spawn_process_peer
;; forks, and forking inside cargo's own multi-threaded test binary is the hazard
;; tests/kernel/spawn_program_prime_process.rs works around with #[ignore] +
;; integration-run.sh; running THIS file via `Command::new(CARGO_BIN_EXE_wat)`, as an
;; ordinary CLI test like tests/cli/wat_cli.rs does, sidesteps it entirely — the fork
;; happens inside a freshly-exec'd, single-threaded `wat` process).
;;
;; Protocol with the harness (see the sibling .rs file):
;;   1. spawns a LABELED process-tier child (`#wat.process/Service {:name ...}`).
;;   2. the child self-asserts its OWN `(:wat::runtime::argv)` is empty (the
;;      "label present, ambient argv still empty" invariant) BEFORE reporting anything —
;;      an assert-eq failure here panics the child, which the outer surfaces as a raise,
;;      which crosses to a non-zero outer exit code the harness can see.
;;   3. the child reports its own pid (`wat.process-id` off its Env) over the peer wire;
;;      the outer relays it to STDOUT and then blocks on its OWN stdin — so BOTH the outer
;;      and the (still-held) child peer stay alive while the harness reads
;;      /proc/<pid>/cmdline for the REAL OS argv the label produced.
;;   4. one line on stdin releases the outer, which drops the child peer (closing its
;;      input pipe → the child's own `readln` sees Eof → it exits cleanly) and exits 0.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [locus (:wat::spawn::with-label (:wat::spawn::process)
             (:wat::process::Service :name (:wat::core::keyword/from-string "my::demo::labeled-svc")))
     p (:wat::test::spawn-peer locus
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::core::let
               [_a (:wat::test::assert-eq 0 (:wat::core::length (:wat::runtime::argv)))
                _p (:wat::kernel::println (:wat::program::Env/wat.process-id (:wat::program::env)))
                outcome (:wat::kernel::readln)]
               (:wat::core::match outcome
                 ((:wat::kernel::ReadlnOutcome::Datum _d) nil)
                 (:wat::kernel::ReadlnOutcome::Eof nil)
                 (:wat::kernel::ReadlnOutcome::Stopped nil))))))
     child-pid (:wat::core::match (:wat::kernel::recv p)
                 ((:wat::kernel::RecvOutcome::Message m) m)
                 ((:wat::kernel::RecvOutcome::Lost cause)
                   (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
                 (:wat::kernel::RecvOutcome::Closed
                   (:wat::kernel::assertion-failed! "labeled child closed before sending its pid" :wat::core::None :wat::core::None)))
     _ (:wat::kernel::println child-pid)
     release-outcome (:wat::kernel::readln)]
    (:wat::core::match release-outcome
      ((:wat::kernel::ReadlnOutcome::Datum _d) nil)
      (:wat::kernel::ReadlnOutcome::Eof nil)
      (:wat::kernel::ReadlnOutcome::Stopped nil))))
