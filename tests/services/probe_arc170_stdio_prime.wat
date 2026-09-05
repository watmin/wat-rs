;; probe_arc170_stdio_prime.wat — arc 170 stdio-as-defservice PHASE 1, the P1 UNIT PROOF fixture.
;;
;; Drives the REAL stdlib primed stdio defservices (:wat::kernel::{stdout,stdin}-svc) on a CONTROLLED
;; fd passed in from the Rust probe (which built a pipe pair). The fixture is :user::-namespaced (a
;; test program cannot define :wat::kernel:: services — reserved-prefix gate — nor call the
;; kernel-restricted `from-fd` directly; it reaches from-fd legitimately only THROUGH the kernel
;; service's generated `::init`). It starts the real service on the given fd, dials its OWN client
;; Peer', and round-trips the surface op — the same client face any caller uses.

;; ── run-stdout: start stdout-svc on `fd`, connect', write two lines, return the # of ::Ok acks. ──
(:wat::core::defn :user::run-stdout [fd <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::let
    [h  (:wat::kernel::stdout-svc/start :locus (:wat::spawn::thread)
          :record (:wat::kernel::stdout-svc::Record) :fd fd)
     c  (:wat::core::match (:wat::kernel::connect (:wat::kernel::stdout-svc::Handle/addr h))
          ((:wat::kernel::ConnectOutcome::Connected p) p)
          ((:wat::kernel::ConnectOutcome::Refused cc)  (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cc) :wat::core::None :wat::core::None))
          ((:wat::kernel::ConnectOutcome::Rejected cc) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cc) :wat::core::None :wat::core::None))
          ((:wat::kernel::ConnectOutcome::Failed cc)   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cc) :wat::core::None :wat::core::None)))
     r1 (:wat::core::match (:wat::kernel::StdOut/write c (:wat::kernel::StdOut::WriteRequest :bytes "primed-line-1\n"))
          ((:wat::kernel::RecvOutcome::Message resp)
            (:wat::core::match resp
              ((:wat::kernel::StdOut::WriteResponse::Ok) 1)
              ((:wat::kernel::StdOut::WriteResponse::RequestTooLarge b cap) 0)
              ((:wat::kernel::StdOut::WriteResponse::RequestMalformed mpath mexpected mgot)
                (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None))))
          ((:wat::kernel::RecvOutcome::Lost cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
          (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "write: stopped — the substrate was asked to stop; the peer was ALIVE" :wat::core::None :wat::core::None))
          (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "write: peer closed" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))
     r2 (:wat::core::match (:wat::kernel::StdOut/write c (:wat::kernel::StdOut::WriteRequest :bytes "primed-line-2\n"))
          ((:wat::kernel::RecvOutcome::Message resp)
            (:wat::core::match resp
              ((:wat::kernel::StdOut::WriteResponse::Ok) 1)
              ((:wat::kernel::StdOut::WriteResponse::RequestTooLarge b cap) 0)
              ((:wat::kernel::StdOut::WriteResponse::RequestMalformed mpath mexpected mgot)
                (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None))))
          ((:wat::kernel::RecvOutcome::Lost cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
          (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "write: stopped — the substrate was asked to stop; the peer was ALIVE" :wat::core::None :wat::core::None))
          (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "write: peer closed" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))]
    (:wat::i64::+ r1 r2)))

;; ── run-stdout-batched: start stdout-svc on `fd`, connect', then drive the BATCHED helper
;;    (:wat::kernel::stdio-write-out) with the FULL `payload` — the write-side fragmentation path the
;;    println verb uses. An oversized payload is CHUNKED into ≤budget raw `write`s (never RequestTooLarge)
;;    and lands VERBATIM on the pipe (write-string is raw — no added/spurious newlines). Returns nil;
;;    a RequestTooLarge / lost / closed inside the helper SURFACES (raise). ──────────────────────────
;; The batched write is done in BINDING position (`_w`) — the service Handle `h` (an earlier binding)
;; stays alive through it, exactly as run-stdout keeps `h` alive across its two writes. (In production the
;; freeze bootstrap holds the primes' Handles globally; here the fixture must hold `h` itself, or dropping
;; it stops the service and the next write sees RecvOutcome::Closed.)
(:wat::core::defn :user::run-stdout-batched [fd <- :wat::core::i64  payload <- :wat::core::String] -> :wat::core::nil
  (:wat::core::let
    [h (:wat::kernel::stdout-svc/start :locus (:wat::spawn::thread)
         :record (:wat::kernel::stdout-svc::Record) :fd fd)
     c (:wat::core::match (:wat::kernel::connect (:wat::kernel::stdout-svc::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected p) p)
         ((:wat::kernel::ConnectOutcome::Refused cc)  (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cc) :wat::core::None :wat::core::None))
         ((:wat::kernel::ConnectOutcome::Rejected cc) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cc) :wat::core::None :wat::core::None))
         ((:wat::kernel::ConnectOutcome::Failed cc)   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cc) :wat::core::None :wat::core::None)))
     _w (:wat::kernel::stdio-write-out c payload)]
    nil))

;; ── run-stdin: start stdin-svc on `fd`, connect', read ONE line → the line String; EOF → "EOF";
;;    RequestTooLarge → "RTL" (all three matchable — the no-hidden-failures EOF upgrade). ──────────
(:wat::core::defn :user::run-stdin [fd <- :wat::core::i64] -> :wat::core::String
  (:wat::core::let
    [h (:wat::kernel::stdin-svc/start :locus (:wat::spawn::thread)
         :record (:wat::kernel::stdin-svc::Record) :fd fd)
     c (:wat::core::match (:wat::kernel::connect (:wat::kernel::stdin-svc::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected p) p)
         ((:wat::kernel::ConnectOutcome::Refused cc)  (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cc) :wat::core::None :wat::core::None))
         ((:wat::kernel::ConnectOutcome::Rejected cc) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cc) :wat::core::None :wat::core::None))
         ((:wat::kernel::ConnectOutcome::Failed cc)   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cc) :wat::core::None :wat::core::None)))]
    (:wat::core::match (:wat::kernel::StdIn/read-frame c (:wat::kernel::StdIn::ReadFrameRequest :max-buffer-bytes 524288))
      ((:wat::kernel::RecvOutcome::Message resp)
        (:wat::core::match resp
          ((:wat::kernel::StdIn::ReadFrameResponse::Frame line) line)
          ((:wat::kernel::StdIn::ReadFrameResponse::Eof) "EOF")
          ;; Arc 170 stdin-joins-the-lock-step — a stop request is its OWN outcome, so it
          ;; reaches every consumer as a located non-exhaustive error rather than silently
          ;; folding into ::Eof. This probe never stops mid-read, so the arm is unreachable
          ;; here; it is written distinctly ("STOP", not "EOF") so a future run that DOES
          ;; hit it reports what happened instead of a plausible lie.
          ((:wat::kernel::StdIn::ReadFrameResponse::Stopped) "STOP")
          ((:wat::kernel::StdIn::ReadFrameResponse::RequestTooLarge b cap) "RTL")
          ((:wat::kernel::StdIn::ReadFrameResponse::RequestMalformed mpath mexpected mgot)
            (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None))))
      ((:wat::kernel::RecvOutcome::Lost cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
      ;; arc 278 #73 — a LOCAL recv() interruption while parked reading, distinct from the
      ;; wire-level ::ReadFrameResponse::Stopped above; same "STOP" surface either way.
      (:wat::kernel::RecvOutcome::Stopped "STOP")
      (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "read-frame: stdin service peer closed" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))
