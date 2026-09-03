;; probe-zero-duration-disarms-at-process.wat — a zero Duration is a LIE at process locus.
;;
;; THE ACCEPTANCE CRITERION, INVERTED. Today this file type-checks, freezes, and runs.
;; After the Interval stone it must NOT COMPILE -- `(:wat::time::Nanosecond 0)` must have
;; no form, so `after` cannot be handed one.
;;
;; `unit_constructor` (src/intrinsic/time.rs:351) rejects n < 0 and admits n == 0, so
;; `Duration(0)` reaches `timerfd_settime` (src/comms/process.rs:1365) as
;; `it_value = {0,0}` -- which POSIX defines as DISARM, not "fire immediately".
;; At thread locus the futex path fires. Same expression, two loci, no error, no
;; diagnostic. That is the locus-transparency break the IPC-stands-in-for-the-network
;; model rests on NOT happening.
;;
;; Independently recorded at SCORE-the-sane-circuit.md:43 on 2026-09-01; this re-measures
;; it on the CURRENT runtime rather than citing it, and commits it as the probe.
;;
;; SELF-GUARDING BY ORDERING: the two cells that must fire run FIRST and print. The cell
;; that hangs runs LAST, so the shell's `timeout` is the watchdog and the lines already
;; printed are the evidence. No wait in this file can swallow its own result -- which is
;; the defect class this whole stone exists to remove.
;;
;; MEASURED 2026-09-03 on the current runtime, 3/3 deterministic:
;;
;;   $ timeout 20 ./target/release/wat .../probe-zero-duration-disarms-at-process.wat
;;   thread-ns0=FIRED       <- futex path fires on zero
;;   process-ns1ms=FIRED    <- the process path works at 1 ms
;;   process-ns0=CLOSED     <- the timer NEVER DELIVERS; recv returns Closed
;;   #wat.kernel/StopAccepted {...}
;;   EXIT=124               <- and the program CANNOT SHUT DOWN
;;
;; ★ TWO FAULTS, and the second is worse than the record. Sibling
;; probe-zero-duration-control.wat is this file with ONLY the zero-timer cell removed:
;; it exits 0, 3/3. So the disarmed timer does not merely fail to fire -- it WEDGES
;; TEARDOWN. The zero-duration peer is never reaped and shutdown waits on it forever.
;;
;; ★ AND THE ARM IS A LIE, not a silence: `Closed` is the substrate's word for "the peer
;; went away", which is indistinguishable from a severed connection. A zero-duration
;; timer MANUFACTURES a spurious Closed -- see the tracker's open `Closed`-after-sever
;; item, which treats Closed as fatal-and-real.
;;
;; ⚠ DIVERGES FROM THE RECORD. SCORE-the-sane-circuit.md:43 recorded, 2026-09-01:
;; "process ns=0 -> TIMED-OUT (500 ms guard)". Today the recv returns Closed promptly
;; instead of timing out. Both cannot be descriptions of the same behaviour. NOT
;; RESOLVED here: either the runtime changed between 09-01 and 09-03, or that guard
;; shape observed a different thing. The 2026-09-01 line is left standing, not
;; overwritten, until someone establishes which.

(:wat::config::set-redef! true)

(:wat::core::defn :zd::fire-thread [ns <- :wat::core::i64] -> :wat::core::String
  (:wat::core::match
    (:wat::kernel::recv
      (:wat::kernel::after :wat::program::PeerKind::thread (:wat::time::Nanosecond ns) :done))
    ((:wat::kernel::RecvOutcome::Message _m) "FIRED")
    ((:wat::kernel::RecvOutcome::Lost _c) "LOST")
    (:wat::kernel::RecvOutcome::Stopped "STOPPED")
    (:wat::kernel::RecvOutcome::Closed "CLOSED")))

(:wat::core::defn :zd::fire-process [ns <- :wat::core::i64] -> :wat::core::String
  (:wat::core::match
    (:wat::kernel::recv
      (:wat::kernel::after :wat::program::PeerKind::process (:wat::time::Nanosecond ns) :done))
    ((:wat::kernel::RecvOutcome::Message _m) "FIRED")
    ((:wat::kernel::RecvOutcome::Lost _c) "LOST")
    (:wat::kernel::RecvOutcome::Stopped "STOPPED")
    (:wat::kernel::RecvOutcome::Closed "CLOSED")))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [a (:zd::fire-thread 0)
     _ (:wat::kernel::println (:wat::core::format "thread-ns0={a}" :a a))
     b (:zd::fire-process 1000000)
     _ (:wat::kernel::println (:wat::core::format "process-ns1ms={b}" :b b))
     c (:zd::fire-process 0)]
    (:wat::kernel::println (:wat::core::format "process-ns0={c}" :c c))))
