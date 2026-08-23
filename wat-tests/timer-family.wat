;; wat-tests/timer-family.wat — arc 292: the time-family rides ONE primitive (after).
;;
;; The doctrine (DESIGN.md): every temporal behaviour is a timer that delivers a
;; typed message into a select'. There is no `sleep` verb — sleep is the timer-Peer
;; in disguise: `(select' [(after <locus> d msg)])`, discard the message. And
;; periodic (cron/heartbeat) needs NO separate `tick` primitive — it is a tail-
;; recursive re-arm of `after` (TCO). This file proves the family on the built
;; thread-tier `after`, deterministically (it counts re-arms, not wall-clock).

;; nap — "sleep", done right: select' on a one-shot after, ignore the tick.
;; A delay is a select (cascade-interruptible by construction); never a thread::sleep.
(:wat::core::defn :test::timer::nap
  [d <- :wat::time::Duration]
  -> :wat::core::nil
  (:wat::core::match
    (:wat::kernel::select
      (:wat::core::Vector (:wat::kernel::Peer :- [:wat::core::nil :wat::core::nil])
        (:wat::kernel::after :wat::program::PeerKind::thread d nil)))
     
    ((:wat::spawn::ServiceEvent::Message _idx _m) nil)
    ((:wat::spawn::ServiceEvent::Closed _idx) nil)
    ((:wat::spawn::ServiceEvent::Lost _idx _cause) nil)
    ((:wat::spawn::ServiceEvent::Malformed _idx _cause) nil)  ;; arc 278 — unreachable for a timer
    ((:wat::spawn::ServiceEvent::Rejected _idx _cause) nil)   ;; arc 278 Stone 1a — unreachable for a timer
    (:wat::spawn::ServiceEvent::Shutdown nil)
    ((:wat::spawn::ServiceEvent::Connection _peer) nil)
    ((:wat::spawn::ServiceEvent::Admin _msg) nil)))

;; retry-with-backoff — the dreaded pattern, as a tail-recursive re-arm of `after`.
;; Naps a growing delay between attempts; returns the attempt it "succeeded" on.
;; Each nap is a fresh one-shot `after` (periodic = re-armed one-shots, no `tick`).
(:wat::core::defn :test::timer::retry-until
  [target <- :wat::core::i64  attempt <- :wat::core::i64  millis <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::if (:wat::core::i64::>= attempt target) 
    attempt
    (:wat::core::let [_ (:test::timer::nap (:wat::time::Millisecond millis))]
      (:test::timer::retry-until
        target
        (:wat::core::i64::+ attempt 1)
        (:wat::core::i64::* millis 2)))))

;; Proof: 3 re-armed `after` naps (1ms → 2ms → 4ms backoff), succeeds on attempt 3.
(:wat::test::deftest :wat-tests::timer::family-backoff-rides-after
  
  (:wat::test::assert-eq
    (:test::timer::retry-until 3 0 1)
    3))

;; timeout's heart: select' over multiple deadlines — the sooner one fires first.
;; The generic "work OR deadline" timeout is this exact shape with one arm a real
;; work-peer; two timers make it deterministic (1ms always beats 20ms). Proves
;; select' multiplexes N timers and returns the first-ready's message.
(:wat::test::deftest :wat-tests::timer::first-deadline-wins
  
  (:wat::test::assert-eq
    (:wat::core::match
      (:wat::kernel::select
        (:wat::core::Vector (:wat::kernel::Peer :- [:wat::core::nil :wat::core::keyword])
          (:wat::kernel::after :wat::program::PeerKind::thread (:wat::time::Millisecond 20) :slow)
          (:wat::kernel::after :wat::program::PeerKind::thread (:wat::time::Millisecond 1) :fast)))
       
      ((:wat::spawn::ServiceEvent::Message _idx m) m)
      ((:wat::spawn::ServiceEvent::Closed _idx) :none)
      ((:wat::spawn::ServiceEvent::Lost _idx _cause) :none)
      ((:wat::spawn::ServiceEvent::Malformed _idx _cause) :none)  ;; arc 278 — unreachable for a timer
      ((:wat::spawn::ServiceEvent::Rejected _idx _cause) :none)   ;; arc 278 Stone 1a — unreachable for a timer
      (:wat::spawn::ServiceEvent::Shutdown :none)
      ((:wat::spawn::ServiceEvent::Connection _peer) :none)
      ((:wat::spawn::ServiceEvent::Admin _msg) :none))
    :fast))
