;; probe_arc170_stdio_prime.wat — arc 170 stdio-as-defservice PHASE 1, the P1 UNIT PROOF fixture.
;;
;; Drives the REAL stdlib primed stdio defservices (:wat::kernel::{stdout,stdin}-svc') on a CONTROLLED
;; fd passed in from the Rust probe (which built a pipe pair). The fixture is :user::-namespaced (a
;; test program cannot define :wat::kernel:: services — reserved-prefix gate — nor call the
;; kernel-restricted `from-fd` directly; it reaches from-fd legitimately only THROUGH the kernel
;; service's generated `::init`). It starts the real service on the given fd, dials its OWN client
;; Peer', and round-trips the surface op — the same client face any caller uses.

;; ── run-stdout: start stdout-svc' on `fd`, connect', write two lines, return the # of ::Ok acks. ──
(:wat::core::defn :user::run-stdout [fd <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::let
    [h  (:wat::kernel::stdout-svc'/start :locus (:wat::spawn::thread)
          :record (:wat::kernel::stdout-svc'::Record) :fd fd)
     c  (:wat::core::match (:wat::kernel::connect' (:wat::kernel::stdout-svc'::Handle/addr h))
          ((:wat::kernel::ConnectOutcome::Connected p) p)
          ((:wat::kernel::ConnectOutcome::Refused cc)  (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cc) :wat::core::None :wat::core::None))
          ((:wat::kernel::ConnectOutcome::Rejected cc) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cc) :wat::core::None :wat::core::None))
          ((:wat::kernel::ConnectOutcome::Failed cc)   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cc) :wat::core::None :wat::core::None)))
     r1 (:wat::core::match (:wat::kernel::StdOut'/write-line c (:wat::kernel::StdOut'::WriteLineRequest :line "primed-line-1"))
          ((:wat::kernel::RecvOutcome::Message resp)
            (:wat::core::match resp
              ((:wat::kernel::StdOut'::WriteLineResponse::Ok) 1)
              ((:wat::kernel::StdOut'::WriteLineResponse::RequestTooLarge b cap) 0)))
          ((:wat::kernel::RecvOutcome::Lost cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
          (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "write-line: peer closed" :wat::core::None :wat::core::None)))
     r2 (:wat::core::match (:wat::kernel::StdOut'/write-line c (:wat::kernel::StdOut'::WriteLineRequest :line "primed-line-2"))
          ((:wat::kernel::RecvOutcome::Message resp)
            (:wat::core::match resp
              ((:wat::kernel::StdOut'::WriteLineResponse::Ok) 1)
              ((:wat::kernel::StdOut'::WriteLineResponse::RequestTooLarge b cap) 0)))
          ((:wat::kernel::RecvOutcome::Lost cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
          (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "write-line: peer closed" :wat::core::None :wat::core::None)))]
    (:wat::core::i64::+ r1 r2)))

;; ── run-stdin: start stdin-svc' on `fd`, connect', read ONE line → the line String; EOF → "EOF";
;;    RequestTooLarge → "RTL" (all three matchable — the no-hidden-failures EOF upgrade). ──────────
(:wat::core::defn :user::run-stdin [fd <- :wat::core::i64] -> :wat::core::String
  (:wat::core::let
    [h (:wat::kernel::stdin-svc'/start :locus (:wat::spawn::thread)
         :record (:wat::kernel::stdin-svc'::Record) :fd fd)
     c (:wat::core::match (:wat::kernel::connect' (:wat::kernel::stdin-svc'::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected p) p)
         ((:wat::kernel::ConnectOutcome::Refused cc)  (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cc) :wat::core::None :wat::core::None))
         ((:wat::kernel::ConnectOutcome::Rejected cc) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cc) :wat::core::None :wat::core::None))
         ((:wat::kernel::ConnectOutcome::Failed cc)   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cc) :wat::core::None :wat::core::None)))]
    (:wat::core::match (:wat::kernel::StdIn'/read-line c (:wat::kernel::StdIn'::ReadLineRequest :max-buffer-bytes 524288))
      ((:wat::kernel::RecvOutcome::Message resp)
        (:wat::core::match resp
          ((:wat::kernel::StdIn'::ReadLineResponse::Line line) line)
          ((:wat::kernel::StdIn'::ReadLineResponse::Eof) "EOF")
          ((:wat::kernel::StdIn'::ReadLineResponse::RequestTooLarge b cap) "RTL")))
      ((:wat::kernel::RecvOutcome::Lost cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "read-line: peer closed" :wat::core::None :wat::core::None)))))
