;; MEASUREMENT for both-ends-bound-the-same-handshake, row 6 — DRIVE the launcher's
;; `RecvOutcome::TimedOut` arm.
;;
;; The defect: `:wat::spawn::ThreadOpts`/`ProcessOpts` `launch` awaited the child's
;; readiness announcement on a BARE `(:wat::kernel::recv …)`. A child that neither
;; crashes (→ Lost) nor exits (→ Closed) nor announces readiness (→ Message) simply
;; hung the launcher forever — and an unbounded wait cannot be stopped by SIGTERM
;; (125 s ignored, measured 2026-09-15). The `TimedOut` arm was already written and
;; was UNREACHABLE: the code named a failure it could not observe.
;;
;; The fix bounds both waits with the startup-handshake deadline, which since excursus 001
;; `the-handshake-deadline-is-injectable` is the nullary
;; `(:wat::program::startup-handshake-deadline-ms)` (default 30000, one definition in
;; `src/intrinsic/program.rs`) rather than the retired `def`
;; `:wat::spawn::STARTUP-HANDSHAKE-DEADLINE-MS`. This probe builds exactly the pathological
;; child — an `:init` that PARKS on a timer channel for 60 s, twice the default deadline —
;; so the launcher must report the timeout instead of hanging.
;;
;; ⭐ THE DEADLINE IS NOW INJECTABLE, which is why this probe is affordable at all:
;;   WAT_STARTUP_HANDSHAKE_DEADLINE_MS=200 time timeout -k 2 90 ./target/release/wat \
;;     wat-scripts/scratch-pad/probe-a-silent-child-cannot-hang-launch.wat
;; ⚠ Read the ARM's timestamp, not the process's wall-clock — on the THREAD tier the parked
;; child thread is still joined at teardown, so the process lives out the full 60 s park
;; regardless of when the arm fired. Prefix each line to see it:
;;   … 2>&1 | perl -MTime::HiRes=time -ne 'BEGIN{$t0=time} printf "+%07.3f %s", time-$t0, $_'
;; ⛔ Do NOT set that variable for a whole test suite. Measured: `wat/test.wat`'s two
;; harness sites wait for the child's COMPLETION signal, not merely its readiness, so the
;; deadline is a forked test child's entire runtime budget — at 200 ms a hermetic child
;; starves before it can boot (threshold measured between 300 and 500 ms on this box).
;;
;; ⛔ HOW TO READ IT: the TimedOut arm in `launch` is `assertion-failed!`, so a
;; SUCCESSFUL probe RAISES and exits non-zero with
;;   `recv: timed out — the peer is alive and silent`
;; in roughly 30 s. A HANG (no output, killed by `timeout`) would be the unbounded
;; defect still present. Run it deliberately, never as part of a suite:
;;   time timeout -k 2 90 ./target/release/wat \
;;     wat-scripts/scratch-pad/probe-a-silent-child-cannot-hang-launch.wat
;; Flip `:locus` between `(:wat::spawn::thread)` and `(:wat::spawn::process)` to drive
;; the two `launch` arms separately (they are separate `extend-type` impls, each with
;; its own bounded wait).
;;
;; ⚠ This file is only PARSED and TYPE-CHECKED by the floor's `every_wat_scripts_file_loads`
;; gate; it is never run there, so the 60 s park costs the floor nothing.
;;
;; ⭐ MEASURED 2026-09-15, excursus 001 `the-handshake-deadline-is-injectable` (arm timestamps,
;; relative to the "sht: starting…" println on the line above it — NOT process exit):
;;   thread,  unset → arm at Δ30.001 s      thread,  =200 → arm at Δ0.201 s
;;   process, unset → arm at Δ30.008 s      process, =200 → arm at Δ0.207 s
;; Same arm, same frame, same message in all four; only the deadline moved.
;;
;; MEASURED 2026-09-15 (both arms, timestamped per output line):
;;   thread  — TimedOut raised at +30.005 s, naming frame `:wat::spawn::ThreadOpts/launch`.
;;             The PROCESS then lingered to +60.5 s: the parked child THREAD is still
;;             joined at teardown, so the launcher no longer hangs forever but the
;;             interpreter's exit still waits out the child's own timer.
;;   process — TimedOut raised at +30.007 s, naming `:wat::spawn::ProcessOpts/launch`;
;;             the parent exited immediately (+0.03 s later). The forked child OUTLIVED
;;             it for the remainder of its own 60 s park and then exited on its own —
;;             no permanent stray (checked with `ps -eo etimes,args | grep release/wat`).
;; ⭐ Both are the same deferred concern, named in the stone: bounding the wait is not
;; supervision. Nothing kills a child that timed out; that is a separate stone.

(:wat::core::defsurface :sht::Slow :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :sht::Slow::PingRequest [])
   (:wat::core::defenum :sht::Slow::PingResponse :wat::enum::Pure
     :Ok               [out <- :wat::core::String]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String
                        got <- :wat::core::String])]
  :features
  [(ping [self <- :sht::Slow  req <- :sht::Slow::PingRequest] -> :sht::Slow::PingResponse :max-request-bytes 4096)])

;; The pathological child: `:init` runs BEFORE the child sends Status::Started (arc 278
;; startup-crash parity), so parking here is a child that is ALIVE, has not crashed,
;; has not closed, and will not announce readiness for 60 s.
(:wat::service::defservice :sht::slow
  :satisfies :sht::Slow
  :durable   []
  :ephemeral []
  ;; ⛔ The park is INLINE, not a call to `:sht::nap` above. Measured: on the PROCESS
  ;; tier the `:init` body is frozen into the child bundle, and a user helper defn is
  ;; NOT shipped with it — calling `:sht::nap` there died with
  ;; `LociDiedError/StartupError … UnresolvedReference {:path ":sht::nap"}` in 0.4 s,
  ;; which is a Lost, NOT the TimedOut this probe exists to drive. Inlining keeps the
  ;; child self-contained so the same file drives both tiers.
  :init (:wat::core::fn [record <- :sht::slow::Record] -> :sht::slow::State
          (:wat::core::let
            [_ (:wat::core::match (:wat::kernel::recv
                   (:wat::kernel::after :wat::program::PeerKind::thread
                     (:wat::time::Milliseconds 60000) :done))
                 ((:wat::kernel::RecvOutcome::Message _m) nil)
                 ((:wat::kernel::RecvOutcome::Lost _c)
                   (:wat::kernel::assertion-failed! "sht: nap lost" :wat::core::None :wat::core::None))
                 (:wat::kernel::RecvOutcome::Stopped
                   (:wat::kernel::assertion-failed! "sht: nap stopped" :wat::core::None :wat::core::None))
                 (:wat::kernel::RecvOutcome::Closed
                   (:wat::kernel::assertion-failed! "sht: nap closed" :wat::core::None :wat::core::None))
                 (:wat::kernel::RecvOutcome::TimedOut
                   (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None))
                 ((:wat::kernel::RecvOutcome::Malformed _cause)
                   (:wat::kernel::assertion-failed! "sht: nap malformed" :wat::core::None :wat::core::None)))]
            (:sht::slow::State :durable record)))
  :impls
  [(ping [s ctx req]
     (:wat::service::Outcome::Continue s
       (:wat::core::Some (:sht::Slow::Reply::Ping (:sht::Slow::PingResponse::Ok "pong")))
       (:wat::core::Vector :- [(:wat::service::Directed :- [:sht::Slow::Reply])])
       (:wat::core::Vector :- [(:wat::service::Alarm :- [:sht::slow::Op])])))])

;; If `launch` is bounded this line RAISES the TimedOut arm at ~30 s and the println
;; below is never reached. If `launch` is unbounded it hangs here forever.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [_ (:wat::kernel::println "sht: starting a service whose :init parks 60s (deadline is 30s)")
     h (:sht::slow/start :locus (:wat::spawn::thread) :record (:sht::slow::Record))]
    (:wat::kernel::println "sht: UNEXPECTED — start returned; the launcher did not time out")))
