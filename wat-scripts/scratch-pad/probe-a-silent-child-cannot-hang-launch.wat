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
;; child — an `:init` that PARKS on a timer channel for **10× that deadline, capped at
;; 60 s** (so 60 s at the default 30000, 2 s at `=200`; see the park comment below) — so
;; the launcher must report the timeout instead of hanging.
;;
;; ⭐ THE DEADLINE IS NOW INJECTABLE, which is why this probe is affordable at all:
;;   WAT_STARTUP_HANDSHAKE_DEADLINE_MS=200 time timeout -k 2 90 ./target/release/wat \
;;     wat-scripts/scratch-pad/probe-a-silent-child-cannot-hang-launch.wat
;; ⚠ Read the ARM's timestamp, not the process's wall-clock — on the THREAD tier the parked
;; child thread is still joined at teardown, so the process lives out the WHOLE park
;; regardless of when the arm fired (60 s at the default deadline, 2 s at `=200`; that join
;; is why the park is derived rather than a literal, and it is the whole cost of the thread
;; floor row). Prefix each line to see it:
;;   … 2>&1 | perl -MTime::HiRes=time -ne 'BEGIN{$t0=time} printf "+%07.3f %s", time-$t0, $_'
;; ⛔ Do NOT set that variable for a whole test suite. Measured: `wat/test.wat`'s two
;; harness sites wait for the child's COMPLETION signal, not merely its readiness, so the
;; deadline is a forked test child's entire runtime budget — at 200 ms a hermetic child
;; starves before it can boot (threshold measured between 300 and 500 ms on this box).
;;
;; ⛔ HOW TO READ IT: the TimedOut arm in `launch` is `assertion-failed!`, so a
;; SUCCESSFUL probe RAISES and exits **2** with
;;   `recv: timed out — the peer is alive and silent`
;; at the handshake deadline. A HANG (no output, killed by `timeout`) would be the
;; unbounded defect still present. ⛔ Use `timeout -k`: a blocked `wat` IGNORES SIGTERM —
;; re-measured 2026-09-15 on this probe, deadline unset: plain `timeout 1` returned only
;; after **60.56 s** (the park ran out; the signal did nothing), `timeout -s KILL 1` at
;; **1.008 s**.
;;   time timeout -k 2 90 ./target/release/wat \
;;     wat-scripts/scratch-pad/probe-a-silent-child-cannot-hang-launch.wat [process]
;; ⭐ The TIER is `argv[2]`: `process` for the process tier, absent (or anything else) for
;; the thread tier — see the note above `:user::main`. It used to be a `:locus` source
;; literal flipped by hand between runs; two floor rows cannot do that.
;;
;; ⭐ THIS FILE IS NOW RUN BY THE FLOOR — twice, one row per tier, with the deadline
;; injected at 200 ms: `tests/probes/scratch_pad_manifest.rs`, rows
;; `a_silent_child_cannot_hang_launch_on_the_{thread,process}_tier` (excursus 001
;; `the-probes-run-in-the-floor`). It is no longer merely parsed by
;; `every_wat_scripts_file_loads`, and the older claim here that "it is never run there,
;; so the 60 s park costs the floor nothing" is retired: the park is now derived from the
;; deadline (below) and the two rows cost ~2.5 s (thread, bounded by that park) and ~0.7 s
;; (process). ⛔ A CHANGE HERE CAN REDDEN THE FLOOR — the rows assert the tier echo, the
;; arm message and the launch FRAME. Run
;; `cargo nextest run --release -E 'binary_id(wat::probes)'` after editing this file.
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
  ;; ⭐ THE PARK IS DERIVED FROM THE DEADLINE, NOT A 60000 LITERAL — added by
  ;; `the-probes-run-in-the-floor` so a floor row can afford the THREAD tier. The park
  ;; must OUTLAST the handshake deadline (otherwise the child's own recv times out first
  ;; and the launcher sees Lost, not TimedOut), but on the thread tier the parked child
  ;; thread is still JOINED AT TEARDOWN, so the park is also the whole process's
  ;; wall-clock. `10 x deadline, capped at 60000` keeps both true:
  ;;   deadline unset (30000) → park 60000  — byte-identical behaviour to the literal
  ;;                                          this replaced; hand-runs read as before
  ;;   deadline =200          → park 2000   — arm at ~0.2 s, process gone by ~2.3 s
  ;; The nullary is read HERE, in the child's own frame, so on the process tier the
  ;; forked child computes its park from the env it inherited (proven to cross the fork
  ;; by `the-handshake-deadline-is-injectable` row 4A).
  :init (:wat::core::fn [record <- :sht::slow::Record] -> :sht::slow::State
          (:wat::core::let
            [deadline-ms (:wat::program::startup-handshake-deadline-ms)
             park-ms (:wat::core::if
                       (:wat::i64::> (:wat::i64::* 10 deadline-ms) 60000)
                       60000
                       (:wat::i64::* 10 deadline-ms))
             _ (:wat::core::match (:wat::kernel::recv
                   (:wat::kernel::after :wat::program::PeerKind::thread
                     (:wat::time::Milliseconds park-ms) :done))
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

;; If `launch` is bounded the `start` below RAISES the TimedOut arm at the handshake
;; deadline and the "UNEXPECTED" println is never reached. If `launch` is unbounded it
;; hangs there forever.
;;
;; ⭐ THE TIER IS AN ARGV WORD, NOT A SOURCE EDIT — added by `the-probes-run-in-the-floor`,
;; which needs BOTH tiers as two floor rows and cannot hand-edit `:locus` between them.
;;   ./target/release/wat …probe-a-silent-child-cannot-hang-launch.wat           → thread
;;   ./target/release/wat …probe-a-silent-child-cannot-hang-launch.wat process   → process
;; `(:wat::runtime::argv)` carries the WHOLE OS argv (argv[0]=binary, argv[1]=this file,
;; argv[2..]=the caller's words — `wat-scripts/scratch-pad/probe-execve-argv-cow-leak.wat`
;; is the exemplar for reading it), so argv[2] is the tier word; the length guard is there
;; because `:wat::core::nth` RAISES out of range and a bare `wat <file>` has no argv[2].
;; (⚠ `:wat::vector::contains?` would have been shorter and does NOT type-check here:
;; it takes `(PersistentVector :- [T])` and `argv` is a `(Vector :- [String])`.)
;; The two `start` calls are written out per branch rather
;; than selecting a locus VALUE: `ThreadOpts` and `ProcessOpts` are distinct types with
;; distinct `:wat::spawn::Locus` impls, so there is no one expression that is either.
;; The tier is echoed on stdout (`sht: tier=…`) so a captured log says which arm it drove —
;; and the raised frame (`ThreadOpts/launch` vs `ProcessOpts/launch`) says it independently.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [argv (:wat::runtime::argv)
     process? (:wat::core::if (:wat::i64::> (:wat::core::length argv) 2)
                (:wat::core::= (:wat::core::nth argv 2) "process")
                false)
     _t (:wat::kernel::println (:wat::core::if process? "sht: tier=process" "sht: tier=thread"))
     _p (:wat::kernel::println "sht: starting a service whose :init parks (10x the handshake deadline, capped 60s)")]
    (:wat::core::if process?
      (:wat::core::let
        [h (:sht::slow/start :locus (:wat::spawn::process) :record (:sht::slow::Record))]
        (:wat::kernel::println "sht: UNEXPECTED — start returned; the launcher did not time out"))
      (:wat::core::let
        [h (:sht::slow/start :locus (:wat::spawn::thread) :record (:sht::slow::Record))]
        (:wat::kernel::println "sht: UNEXPECTED — start returned; the launcher did not time out")))))
