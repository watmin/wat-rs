;; arc 292 — RED probe for the timer-Peer (send_after / time-as-select).
;;
;; The contract (DESIGN.md rev2): a one-shot timer DELIVERS a caller-chosen, typed
;; message of the select' set's own type O, after a Duration — Erlang's send_after.
;; Because it yields an O, it drops into the HOMOGENEOUS select' next to real peers
;; with zero select' change. This probe is the north-star: it asserts the timer
;; delivers exactly the message it was handed (:tick), surfaced as a
;; ServiceEvent::Message out of select'.
;;
;; RED at HEAD: :wat::kernel::after is unregistered (post-255 → UnresolvedReference).
;; Everything else here already exists — select', Vector, ServiceEvent, Peer',
;; :wat::time::Millisecond — so the probe fails on EXACTLY the one missing primitive.
;;
;; Namespace decision (arc 292 D1): after/tick are :wat::kernel:: (effectful peer
;; constructors, beside connect'/select'); they CONSUME a :wat::time:: Duration.
;; Tier decision (arc 292 D2 = B1): the timer is a TIER peer — a LOCUS picks the
;; tier (and thus the reactor), so it satisfies select's Thread'|Process' constraint
;; with zero select' change. (after (thread) d msg) -> (Thread' :- [nil O]); mirrors start.

(:wat::test::deftest :wat-tests::timer::after-delivers-its-message
  
  (:wat::test::assert-eq
    (:wat::core::match
      (:wat::kernel::select
        (:wat::core::Vector :- [(:wat::kernel::Peer :- [:wat::core::nil :wat::core::keyword])]
          (:wat::kernel::after :wat::program::PeerKind::thread (:wat::time::Milliseconds 50) :tick)))
       
      ((:wat::spawn::ServiceEvent::Message _idx msg) msg)
      ((:wat::spawn::ServiceEvent::Closed _idx) :no-tick)
      ((:wat::spawn::ServiceEvent::Lost _idx _cause) :no-tick)
      ((:wat::spawn::ServiceEvent::Malformed _idx _cause) :no-tick)  ;; arc 278 — unreachable for a timer
      ((:wat::spawn::ServiceEvent::Rejected _idx _cause) :no-tick)   ;; arc 278 Stone 1a — unreachable for a timer
      (:wat::spawn::ServiceEvent::Shutdown :no-tick)
      ((:wat::spawn::ServiceEvent::Connection _peer) :no-tick)
      ((:wat::spawn::ServiceEvent::Admin _msg) :no-tick))
    :tick))
