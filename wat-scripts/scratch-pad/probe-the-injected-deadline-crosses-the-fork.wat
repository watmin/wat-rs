;; MEASUREMENT for excursus 001 `the-handshake-deadline-is-injectable`, ROW 4 — the CHILD
;; end of the process-tier handshake, not just the parent's `launch`.
;;
;; Row 4 says: "show the child end (`child-main`'s wait) honouring the injected value, not
;; just the parent's `launch`. Env inheritance is the assumption — verify it." Two halves,
;; both OBSERVED here, and one thing that is NOT driven and says so.
;;
;; ── A. THE VALUE CROSSES THE FORK ────────────────────────────────────────────────────
;; `(:wat::test::run-hermetic …)` forks a PROCESS-tier child via
;; `:wat::test::spawn-hermetic-program` — itself one of the handshake's five sites. The
;; child reads `(:wat::program::startup-handshake-deadline-ms)` inside its OWN process and
;; carries the number back in its `Failure` message. If env inheritance did not hold, the
;; child would report the default while the parent reported the injected value.
;;
;;   ./target/release/wat wat-scripts/scratch-pad/probe-the-injected-deadline-crosses-the-fork.wat
;;   WAT_STARTUP_HANDSHAKE_DEADLINE_MS=1500 ./target/release/wat wat-scripts/scratch-pad/probe-the-injected-deadline-crosses-the-fork.wat
;;
;; ⛔ DO NOT use 200 here. `spawn-hermetic-program` awaits the child's COMPLETION signal,
;; not merely its readiness, so the deadline is the forked child's whole runtime budget and
;; 200 ms starves it before it can boot — the harness's own TimedOut arm fires instead and
;; nothing is learned about inheritance. Measured on this box: 300 ms starves, 500 ms does
;; not. 1500 is comfortably clear of that and unmistakably not the 30000 default.
;;
;; ── B. THE GENERATED `child-main` CALLS IT, RATHER THAN CARRYING A NUMBER ─────────────
;; `defservice` emits `(:<fqdn>::service-forms)` — the literal forms that cross the fork,
;; including the generated child `:user::main`. This prints them with
;; `:wat::core::ast->source` so the deadline argument can be READ rather than asserted:
;;
;;   ./target/release/wat …probe-the-injected-deadline-crosses-the-fork.wat \
;;     | grep -o 'recv-by-deadline [^)]*)\{0,1\}' | sort -u
;;   →  recv-by-deadline self (:wat::program::startup-handshake-deadline-ms)
;;
;; What must be there: `(:wat::program::startup-handshake-deadline-ms)` as the second
;; argument — a CALL the child evaluates in its own process. What must NOT be there: any
;; number in that position, which is what an unquoted (`~`) splice of the parent's
;; expansion-time value would have baked in — route (c), half a fence.
;;
;; ── ⚠ WHAT IS **NOT** DRIVEN HERE, AND WHY ───────────────────────────────────────────
;; `child-main`'s own `RecvOutcome::TimedOut` arm is NOT made to fire. It cannot be, from
;; outside: `ProcessOpts/launch` sends the startup ship DOWN immediately after
;; `spawn-program` returns, so by the time the forked child has loaded the runtime and
;; reached its `recv-by-deadline` the ship is already queued and the recv returns
;; `Message` at once. The only deadline short enough to lose that race is shorter than IPC
;; delivery, i.e. sub-millisecond and not reliably reproducible. Withholding the ship
;; would mean editing `wat/spawn.wat`. So: the child's VALUE is driven, the child's ARM is
;; not, and this comment is here so nobody reads A+B as the arm having been observed.
;;
;; MEASURED 2026-09-15 — see the stone's SCORE.md.
;;   unset → PARENT-deadline-ms=30000  child reported 30000
;;   =1500 → PARENT-deadline-ms=1500   child reported 1500     (also checked at 500 / 5000)
;;   =200  → the harness's own handshake TimedOut before the child could report (see above)
;;   half B, every run → recv-by-deadline self (:wat::program::startup-handshake-deadline-ms)

;; A minimal service that exists only so its `service-forms` can be read. It is never
;; started, so this probe spawns exactly one child (the hermetic one in half A).
(:wat::core::defsurface :dxf::Echo :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :dxf::Echo::PingRequest [])
   (:wat::core::defenum :dxf::Echo::PingResponse :wat::enum::Pure
     :Ok               [out <- :wat::core::String]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String
                        got <- :wat::core::String])]
  :features
  [(ping [self <- :dxf::Echo  req <- :dxf::Echo::PingRequest] -> :dxf::Echo::PingResponse :max-request-bytes 4096)])

(:wat::service::defservice :dxf::echo
  :satisfies :dxf::Echo
  :durable   []
  :ephemeral []
  :impls
  [(ping [s ctx req]
     (:wat::service::Outcome::Continue s
       (:wat::core::Some (:dxf::Echo::Reply::Ping (:dxf::Echo::PingResponse::Ok "pong")))
       (:wat::core::Vector :- [(:wat::service::Directed :- [:dxf::Echo::Reply])])
       (:wat::core::Vector :- [(:wat::service::Alarm :- [:dxf::echo::Op])])))])

(:wat::core::defn :dxf::show-forms
  [fs <- (:wat::core::Vector :- [:wat::WatAST])]
  -> :wat::core::nil
  (:wat::core::reduce
    (:wat::core::fn [_acc <- :wat::core::nil  f <- :wat::WatAST] -> :wat::core::nil
      (:wat::kernel::println (:wat::core::ast->source f)))
    nil
    fs))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [_ (:wat::kernel::println
         (:wat::string::interpolate "PARENT-deadline-ms={p}"
           :p (:wat::i64::to-string (:wat::program::startup-handshake-deadline-ms))))
     ;; ── A: the forked process-tier child reads its own, from the inherited env.
     ;; ⛔ The child reports by DYING, not by printing. Measured: a `println` in a
     ;; hermetic child does not reach this process's stdout, so a print would have been an
     ;; unfalsifiable absence. `assertion-failed!` carries the child's OWN reading back
     ;; across the fork as a `Failure`, which the parent can read out below. The
     ;; "failure" is the payload channel, not a defect.
     r (:wat::test::run-hermetic
         (:wat::kernel::assertion-failed!
           (:wat::string::interpolate "CHILD-PROCESS-deadline-ms={c}"
             :c (:wat::i64::to-string (:wat::program::startup-handshake-deadline-ms)))
           :wat::core::None :wat::core::None))
     _ (:wat::core::match r
         (:wat::kernel::RunResult::Passed
           (:wat::kernel::println "hermetic-child=Passed — UNEXPECTED, the child was supposed to report by dying"))
         ((:wat::kernel::RunResult::Failed f)
           (:wat::kernel::println
             (:wat::string::interpolate "hermetic-child-reported: {m}"
               :m (:wat::kernel::Failure/message f)))))
     ;; ── B: the generated child-main, as forms, for reading.
     _ (:wat::kernel::println "── (:dxf::echo::service-forms) ──")
     _ (:dxf::show-forms (:dxf::echo::service-forms))]
    nil))
