;; MEASUREMENT for excursus 001 `the-handshake-deadline-is-injectable`, rows 5 and 7.
;;
;; Two things, in ONE process so they share one environment:
;;
;;  A. THE KNOB IS VISIBLE, AND STABLE WITHIN A RUN. Two calls to
;;     `(:wat::program::startup-handshake-deadline-ms)` print the value and whether they
;;     agree. ⚠ What this can and cannot see: agreement is NOT proof of a single env read
;;     (a per-call `std::env::var` would also agree with itself). It excludes a knob that
;;     DRIFTS within a run; the `OnceLock::get_or_init` in `src/intrinsic/program.rs` is
;;     the structural half of the read-once claim, and this is its wat-visible face.
;;
;;  B. ⛔ ROW 5 — `owner-recv-loop` IS NOT RE-TIMED. This calls
;;     `:wat::service::owner-recv-loop` directly with the SAME `budget-ms` literal all EIGHT
;;     of its generated call sites in wat/service.wat pass it — `10000`, at :3205 :3226
;;     :3343 :3360 :3476 :3482 :3579 :3585 — against a peer that is ALIVE and SILENT for
;;     60 s (a timer peer whose message is 60 s out). So the loop must burn its whole
;;     budget and return `GaveUp waited-ms≈10000 last="TimedOut"`.
;;
;;     ⭐ THE POINT: run this with `WAT_STARTUP_HANDSHAKE_DEADLINE_MS=200`. If the env var
;;     had been read inside `recv_by_deadline` — the trap-door the DESIGN forbids — this
;;     would come back in ~0.2 s with `waited-ms≈0`. It must come back in ~10 s. This is a
;;     DRIVEN negative control, not an assertion that the primitive was left alone.
;;     `owner-recv-loop` takes `budget-ms` as a PARAMETER, so calling it here is the real
;;     function on the real number, not a stand-in.
;;
;; HOW TO RUN (each takes ~10 s — the budget, deliberately spent):
;;   time timeout -k 2 60 ./target/release/wat \
;;     wat-scripts/scratch-pad/probe-the-handshake-deadline-is-injectable.wat
;;   WAT_STARTUP_HANDSHAKE_DEADLINE_MS=200 time timeout -k 2 60 ./target/release/wat \
;;     wat-scripts/scratch-pad/probe-the-handshake-deadline-is-injectable.wat
;;
;; ⚠ The floor only PARSES and TYPE-CHECKS this file (`every_wat_scripts_file_loads`); it
;; is never run there, so the 10 s budget costs the floor nothing.
;;
;; MEASURED 2026-09-15 (see the stone's SCORE.md for the wall-clocks):
;;   unset : handshake-deadline-ms=30000  owner-recv-loop GaveUp waited-ms=10000
;;   =200  : handshake-deadline-ms=200    owner-recv-loop GaveUp waited-ms=10000   ← row 5

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [a (:wat::program::startup-handshake-deadline-ms)
     b (:wat::program::startup-handshake-deadline-ms)
     _ (:wat::kernel::println
         (:wat::string::interpolate "handshake-deadline-ms={a} second-read-agrees={eq}"
           :a  (:wat::i64::to-string a)
           :eq (:wat::core::show (:wat::core::= a b))))
     ;; The silent-but-alive peer: a timer 60 s out, six times the budget below.
     t0 (:wat::time::epoch-nanos (:wat::time::now))
     out (:wat::service::owner-recv-loop
           (:wat::kernel::after :wat::program::PeerKind::thread
             (:wat::time::Milliseconds 60000) :never)
           t0 10000 "recv")
     _ (:wat::core::match out
         ((:wat::service::StopOutcome::GaveUp waited last)
           (:wat::kernel::println
             (:wat::string::interpolate "owner-recv-loop=GaveUp waited-ms={w} last={l}"
               :w (:wat::i64::to-string waited)
               :l last)))
         ((:wat::service::StopOutcome::Stopped m)
           (:wat::kernel::println
             (:wat::string::interpolate "owner-recv-loop=Stopped {m} — UNEXPECTED, the timer fired early"
               :m (:wat::core::show m))))
         ((:wat::service::StopOutcome::Gone c)
           (:wat::kernel::println
             (:wat::string::interpolate "owner-recv-loop=Gone {c} — UNEXPECTED, the timer peer died"
               :c (:wat::kernel::LociDiedError/message c)))))]
    nil))
