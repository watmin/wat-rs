;; PROBE — can a BARE `recv` ever return TimedOut? A DIFFERENTIAL with a positive control.
;;
;; THE CLAIM UNDER TEST. `wat/service.wat`'s generated `child-main` waits for its owner's startup
;; ship with a BARE `(:wat::kernel::recv self)` and writes a `RecvOutcome::TimedOut` arm that raises
;; "recv: timed out — the peer is alive and silent". Arc 109's NOTE line 43 says a bare recv
;; "blocks forever and can never return TimedOut", and `recv_outcome_timedout()` is called at three
;; sites in src/runtime.rs, ALL inside recv-by-deadline. If that holds, then at the single most
;; leveraged point in the system — EVERY service's startup — a slow owner does not raise. It HANGS,
;; and the arm that appears to handle it is dead code reading as coverage.
;;
;; ⛔ WHY A DIFFERENTIAL AND NOT ONE CASE: "it blocked" and "the variant does not exist" look
;; identical from one arm. The CONTROL proves the variant IS producible on the SAME never-firing
;; peer, so a block in the SUBJECT is a fact about the CALL SITE, not about the peer or the enum.
;; Five of this orchestrator's claims died this session for want of exactly this.
;;
;; ⛔ THIS PROBE DIES BY DESIGN, AND THE DEATH IS THE MEASUREMENT. `recv-by-deadline` REFUSES a
;;   `Peer` at runtime:
;;     "recv-by-deadline: expected owner handle ((Thread :- [I O]) | (Process :- [I O])),
;;      got :wat::kernel::Peer"
;;   That refusal is the finding: the deadline-bearing recv arc 109 shipped serves the OWNER side
;;   only, so a child awaiting its owner's startup ship has NO bounded recv available. Precedent for
;;   a probe whose deliverable is a documented refusal: `probe-owner-select-deadline.wat:37`
;;   ("peer-wire? REFUSES a lineage handle: runtime TypeMismatch").
;;   ⭑ The bound IS achievable on a Peer — `call-by-deadline` does `(select [peer tmr])`
;;   (wat/service.wat:4283). It is a USAGE gap, not a substrate gap. Census:
;;   docs/excursus/2026/08/001-sns-sqs/every-waiter-can-bound-its-wait/
;;
;; EXPECTED, if the claim holds:
;;   control=bounded      select over [silent-peer, short-timer] RETURNS  → a bound IS achievable
;;                        on a Peer. ⛔ recv-by-deadline CANNOT be used: it rejects a Peer outright
;;                        ("expected owner handle (Thread|Process), got :wat::kernel::Peer"), so the
;;                        deadline-bearing recv arc 109 shipped serves the OWNER side only.
;;   subject-starting     printed, then NOTHING — the bare recv never returns. A HANG is the finding.
;; A `subject=` line of any kind REFUTES the claim, and that is the more interesting outcome.

;; A peer that will not send inside our window: a timer set far past it.
(:wat::core::defn :br::silent-peer [] -> (:wat::kernel::Peer :- [:wat::core::keyword :wat::core::keyword])
  (:wat::kernel::after :wat::program::PeerKind::thread (:wat::time::Milliseconds 3600000) :never))

(:wat::core::defn :br::name [o <- (:wat::kernel::RecvOutcome :- [:wat::core::keyword])] -> :wat::core::String
  (:wat::core::match o
    ((:wat::kernel::RecvOutcome::Message _m) "Message")
    ((:wat::kernel::RecvOutcome::Lost _c) "Lost")
    (:wat::kernel::RecvOutcome::Stopped "Stopped")
    (:wat::kernel::RecvOutcome::Closed "Closed")
    (:wat::kernel::RecvOutcome::TimedOut "TimedOut")
    ((:wat::kernel::RecvOutcome::Malformed _c) "Malformed")))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [;; ⭑ CONTROL — arc 109's UNTAKEN route: select over [silent peer, short timer]. Both are
    ;;   Peers, so the mixed-tier refusal that blocked the OWNER case does not apply here.
    ;;   If this returns, a bound on a Peer IS achievable and child-main simply never took it.
     ev (:wat::kernel::select
          (:wat::core::Vector :- [(:wat::kernel::Peer :- [:wat::core::keyword :wat::core::keyword])]
            (:br::silent-peer)
            (:wat::kernel::after :wat::program::PeerKind::thread (:wat::time::Milliseconds 300) :tick)))
     _ (:wat::kernel::println
         (:wat::core::match ev
           ((:wat::spawn::ServiceEvent::Message idx _m)
             (:wat::core::format "control=bounded-by-select idx={i}" :i (:wat::i64::to-string idx)))
           ((:wat::spawn::ServiceEvent::Closed _i) "control=Closed")
           ((:wat::spawn::ServiceEvent::Lost _i _c) "control=Lost")
           ((:wat::spawn::ServiceEvent::Malformed _i _c) "control=Malformed")
           ((:wat::spawn::ServiceEvent::Connection _p) "control=Connection")
           ((:wat::spawn::ServiceEvent::Admin _a) "control=Admin")
           ((:wat::spawn::ServiceEvent::Rejected _i _c) "control=Rejected")
           (:wat::spawn::ServiceEvent::Shutdown "control=Shutdown")))
     _ (:wat::kernel::println "subject-starting")
     ;; ⛔ SUBJECT — the same wait, BARE, exactly as child-main does it for the startup ship.
     s (:wat::kernel::recv (:br::silent-peer))
     _ (:wat::kernel::println (:wat::string::concat "subject=" (:br::name s)))]
    nil))
