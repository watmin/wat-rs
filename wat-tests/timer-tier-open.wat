;; arc 292 L3 — north-star for the tier-open timer (decision A, 2026-06-23).
;;
;; The locked surface: arg0 of `after` is a `:wat::program::PeerKind` (NOT a spawn-locus),
;; and the timer is a SELECTABLE that lives IN the `select'` vector (Go `select` /
;; Clojure `(alts! [ch (timeout d)])`). Its type is the tier-OPEN `(Timer' :- [O])`, which
;; fuses to the concrete tier of whatever homogeneous `select'` set it joins; alone in a
;; vector it is `(Timer' :- [O])` and `select'` projects `(ServiceEvent :- [nil O])`.
;;
;; The tier-agnostic idiom (not shown here — needs a peer to grab from) is
;; `(after (peer-kind-of (:wat::program::env)) d msg)`; here we use the explicit literal
;; `:wat::program::PeerKind::process` ("I want a process-tier timer").
;;
;; RED at HEAD on EXACTLY the new design: `after` currently takes a ThreadOpts/ProcessOpts
;; spawn-locus, not a PeerKind; and there is no tier-open `Timer'` type. Everything else
;; (select', Vector, ServiceEvent, PeerKind, :wat::time::Millisecond) already exists.
;;
(:wat::test::deftest :wat-tests::timer::tier-open-after-peerkind
  
  (:wat::test::assert-eq
    (:wat::core::match
      (:wat::kernel::select
        (:wat::core::Vector :- [(:wat::kernel::Peer :- [:wat::core::nil :wat::core::keyword])]
          (:wat::kernel::after :wat::program::PeerKind::process
                               (:wat::time::Milliseconds 50)
                               :tick)))
       
      ((:wat::spawn::ServiceEvent::Message _idx msg) msg)
      ((:wat::spawn::ServiceEvent::Closed _idx) :no-tick)
      ((:wat::spawn::ServiceEvent::Lost _idx _cause) :no-tick)
      ((:wat::spawn::ServiceEvent::Malformed _idx _cause) :no-tick)  ;; arc 278 — unreachable for a timer
      ((:wat::spawn::ServiceEvent::Rejected _idx _cause) :no-tick)   ;; arc 278 Stone 1a — unreachable for a timer
      (:wat::spawn::ServiceEvent::Shutdown :no-tick)
      ((:wat::spawn::ServiceEvent::Connection _peer) :no-tick)
      ((:wat::spawn::ServiceEvent::Admin _msg) :no-tick))
    :tick))
