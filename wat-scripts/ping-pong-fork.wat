;; wat-scripts/ping-pong-fork.wat — fork variant of the ping-pong
;; proof. Same shape as ping-pong.wat (5 round trips of EDN over
;; pipes), but the child runs in a real OS process via
;; :wat::kernel::fork-program-ast instead of :wat::kernel::spawn-
;; program (thread).
;;
;; Usage:
;;   ./target/release/wat ./wat-scripts/ping-pong-fork.wat
;;
;; Expected output:
;;   round 1: ping → pong (forked)
;;   round 2: ping → pong (forked)
;;   round 3: ping → pong (forked)
;;   round 4: ping → pong (forked)
;;   round 5: ping → pong (forked)
;;   done — 5 round trips (real OS fork)
;;
;; The shape:
;;
;;   wat-cli (Rust binary, OS process A)
;;     └─ ping-pong-fork.wat (frozen world A, main thread of A)
;;          ├─ stdin/stdout/stderr → real OS handles
;;          └─ :wat::kernel::fork-program-ast ──fork(2)──┐
;;                                                      ↓
;;   OS process B (child of A, separate address space, separate fd table)
;;     └─ pong loop (frozen world B, this process's :user::main)
;;          └─ stdin/stdout/stderr → 3 OS pipe ends (dup2'd to 0/1/2
;;             via fork-program-ast's child branch)
;;
;; Bidirectional traffic over real cross-process pipes. The first
;; honest fork-pipe demo with interleaved Ping/Pong — every prior
;; fork-program-ast user (run-sandboxed-hermetic-ast, wat_fork.rs
;; tests) was monologue (child writes, parent reads after). This
;; proves fork-program-ast holds up under round-trip pressure.
;;
;; Why this matters: it de-risks a hypothetical wat-cli rewrite
;; (always-fork-the-program, "the cli is the surface") that would
;; rest on the same primitive.

(:wat::core::struct :demo::Ping
  (n :i64))

(:wat::core::struct :demo::Pong
  (n :i64))


;; Recursive ping-pong loop. Sends a Ping via process-send, reads the
;; Pong via process-recv, asserts the n echoes correctly, logs the
;; round, recurses with round+1 until round == total.
(:wat::core::define
  (:demo::ping-pong::loop
    (proc    :wat::kernel::Process<demo::Ping,demo::Pong>)
    (out     :wat::io::IOWriter)
    (round   :i64)
    (total   :i64)
    -> :())
  (:wat::core::if (:wat::core::i64::>= round total) -> :()
    ()
    (:wat::core::let*
      (((ping :demo::Ping) (:demo::Ping/new round))
       ((_send :())
        (:wat::core::result::expect -> :()
          (:wat::kernel::process-send proc ping)
          "ping-pong-fork: send to child failed"))
       ((pong :demo::Pong)
        (:wat::core::match (:wat::kernel::process-recv proc) -> :demo::Pong
          ((Ok (:wat::core::Some v)) v)
          ((Ok :wat::core::None)
           (:wat::core::panic! "ping-pong-fork: child closed stdout early"))
          ((Err _died)
           (:wat::core::panic! "ping-pong-fork: child died"))))
       ((n-back :i64) (:demo::Pong/n pong))
       ((_check :())
        (:wat::core::if (:wat::core::= n-back round) -> :()
          ()
          (:wat::core::panic! "ping-pong-fork: pong n mismatch")))
       ((_log :())
        (:wat::io::IOWriter/println out
          (:wat::core::string::concat
            (:wat::core::string::concat
              "round "
              (:wat::core::i64::to-string (:wat::core::i64::+ round 1)))
            ": ping → pong (forked)"))))
      (:demo::ping-pong::loop proc out
        (:wat::core::i64::+ round 1) total))))


(:wat::core::define
  (:user::main
    (stdin  :wat::io::IOReader)
    (stdout :wat::io::IOWriter)
    (stderr :wat::io::IOWriter)
    -> :())
  (:wat::core::let*
    (((total :i64) 5)
     ;; The child program — a fresh frozen world built by
     ;; fork-program-ast in process B. The forms are captured
     ;; UNEVALUATED via :wat::core::forms (the variadic-quote
     ;; substrate); fork-program-ast hands them to startup_from_forms
     ;; in the child branch.
     ((child-forms :Vec<wat::WatAST>)
      (:wat::core::forms
        (:wat::core::struct :demo::Ping
          (n :i64))
        (:wat::core::struct :demo::Pong
          (n :i64))
        (:wat::core::define
          (:demo::pong::loop
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            -> :())
          (:wat::core::match (:wat::io::IOReader/read-line stdin) -> :()
            (:wat::core::None ())
            ((:wat::core::Some line)
             (:wat::core::let*
               (((ping :demo::Ping) (:wat::edn::read line))
                ((n    :i64)         (:demo::Ping/n ping))
                ((pong :demo::Pong) (:demo::Pong/new n))
                ((_    :())
                 (:wat::io::IOWriter/println stdout (:wat::edn::write pong))))
               (:demo::pong::loop stdin stdout)))))
        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:demo::pong::loop stdin stdout))))
     ;; Fork. Process B starts; runs child-forms's :user::main with
     ;; pipe-backed stdio. Returns a Process<Ping,Pong> struct with the
     ;; parent-side pipe ends + a ProgramHandle for join.
     ((proc :wat::kernel::Process<demo::Ping,demo::Pong>)
      (:wat::kernel::fork-program-ast child-forms))
     ;; The conversation. Five round trips; mutual blocking on each.
     ((_loop :()) (:demo::ping-pong::loop proc stdout 0 total))
     ;; End the conversation. Closing the child's stdin via the
     ;; Process stdin accessor releases the kernel pipe write-end
     ;; → child's read-line returns :None → child exits its loop
     ;; → child process exits → Process/join-result reaps it.
     ((_close :()) (:wat::io::IOWriter/close (:wat::kernel::Process/stdin proc)))
     ((_wait :())
      (:wat::core::match (:wat::kernel::Process/join-result proc) -> :()
        ((Ok _) ())
        ((Err _died)
         (:wat::core::panic! "ping-pong-fork: child exited non-zero")))))
    (:wat::io::IOWriter/println stdout
      (:wat::core::string::concat
        "done — "
        (:wat::core::string::concat
          (:wat::core::i64::to-string total)
          " round trips (real OS fork)")))))
