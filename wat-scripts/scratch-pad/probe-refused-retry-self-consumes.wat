;; probe-refused-retry-self-consumes.wat — why probe_async_publish::refused_subscriber_is_retried
;; TIMED OUT on the floor of 2026-09-03T09-14-58Z instead of failing an assertion.
;;
;; HYPOTHESIS: the test consumes the very message it then waits for.
;;
;; :user::refused-is-retried DID, in order:
;;     dummy-id    <- take-one subq          ;; drain the cap-1 filler
;;     ack-one subq dummy-id                 ;; subq is now FREE
;;     after-drain <- take-one subq          ;; expects "" -- but this is a DESTRUCTIVE read
;;                                           ;; with :visibility-ns 1000000000000 (~1000 s)
;;     await-timer-ms 350
;;     wait-pending subq                     ;; UNBOUNDED spin until pending >= 1
;; Stone D: absence is q-depth; presence is one blocking receive. wait-pending is gone.
;;
;; The worker holds "hello" in-flight in the INBOX under a 200 ms visibility. When that
;; expires it re-receives and sends to subq -- which the ack above just freed. If that
;; redelivery lands BEFORE `after-drain` runs, `after-drain` takes "hello" and hides it for
;; ~1000 s. The inbox is then empty (the worker acked on Ok), so subq pending can never
;; reach 1 again and `wait-pending` spins forever. The Rust assertion `after-drain == "none"`
;; never gets to fire, because the wat function never returns. TIMEOUT, empty stdout.
;;
;; The window between `wait-inflight` returning and `after-drain` running is three service
;; round-trips -- microseconds on an idle box, which is why it passes 1.35 s alone. It only
;; has to exceed 200 ms once, under a 45-binary floor, to stall permanently.
;;
;; THE VARIABLE IS THE GAP, and nothing else. Same program, same services, same 200 ms
;; vis-ns; only the delay between the ack and `after-drain` moves.
;;
;;   gap=0    -> after-drain=none ; pending=1  -- wait-pending would return   (control)
;;   gap=300  -> after-drain=got  ; pending=0  -- wait-pending can NEVER return (the hang)
;;
;; Reports the depth instead of waiting on it, so the stall is VISIBLE rather than a timeout.

(:wat::config::set-redef! true)
(:wat::load-file! "../topic/sns-fanout.wat")

;; Bounded poll: returns the number of naps until subq pending >= 1, or -1 if it never
;; happened inside the budget. This is the claim under test -- "spins forever" is a statement
;; about the FUTURE, and pending=0 at one instant does not establish it.
(:wat::core::defn :rr::poll-pending
  [q <- :queue::Queue  attempts <- :wat::core::i64  ms <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::if (:wat::i64::<= attempts 1)
    (:wat::core::if (:wat::i64::>= (:wat::core::first (:demo::q-depth q)) 1) 0 -1)
    (:wat::core::if (:wat::i64::>= (:wat::core::first (:demo::q-depth q)) 1)
      0
      (:wat::core::let
        [_ (:demo::await-timer-ms ms)
         r (:rr::poll-pending q (:wat::i64::- attempts 1) ms)]
        (:wat::core::if (:wat::i64::< r 0) -1 (:wat::i64::+ r 1))))))

(:wat::core::defn :rr::run [gap-ms <- :wat::core::i64] -> :wat::core::String
  (:wat::core::let
    [ish (:wat::query::mem-store/start :locus (:wat::spawn::thread)
           :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     iqh (:queue::queue/start :locus (:wat::spawn::thread)
           :record (:queue::queue::Record :cap 64 :store-addr (:wat::query::mem-store::Handle/addr ish) :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     ssh (:wat::query::mem-store/start :locus (:wat::spawn::thread)
           :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     sqh (:queue::queue/start :locus (:wat::spawn::thread)
           :record (:queue::queue::Record :cap 1 :store-addr (:wat::query::mem-store::Handle/addr ssh) :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     qaddrs (:wat::core::Vector :- [(:wat::kernel::Address :- [:queue::Queue::Op :queue::Queue::Reply])]
              (:queue::queue::Handle/addr sqh))
     th (:demo::topic/start :locus (:wat::spawn::thread)
          :record (:demo::topic::Record :nsubs 1 :inbox-addr (:queue::queue::Handle/addr iqh)))
     wh (:demo::topic-worker/start :locus (:wat::spawn::thread)
          :record (:demo::mk-tw 200000000 (:queue::queue::Handle/addr iqh) qaddrs 0 0))
     inbox (:demo::dial-queue (:queue::queue::Handle/addr iqh))
     subq  (:demo::dial-queue (:queue::queue::Handle/addr sqh))
     tc    (:demo::dial-topic (:demo::topic::Handle/addr th))
     tw    (:demo::dial-topic-worker (:demo::topic-worker::Handle/addr wh))
     _ (:demo::send-one subq "q0" "dummy")
     _ (:demo::start-topic-worker! tw)
     _ (:demo::publish-until-accepted! tc "hello")
     _ (:demo::require! (:demo::poll-until-unacked inbox 2000))
     dummy-id (:demo::claim-one! subq "q0" 1000000000000)
     _ (:demo::ack-one subq "q0" dummy-id)
     ;; `(await-timer-ms 0)` is itself a zero wait and has no form after Stone A —
     ;; "wait for zero" IS "don't wait", the mode-as-magnitude this arc removed.
     _ (:wat::core::if (:wat::i64::> gap-ms 0) (:demo::await-timer-ms gap-ms) nil)
     after-visible (:wat::core::first (:demo::q-depth subq))
     _ (:demo::await-timer-ms 350)
     recovered (:rr::poll-pending subq 100 50)
     d (:demo::q-depth subq)
     di (:demo::q-depth inbox)]
    (:wat::core::format "gap={g};after-drain={a};pending={p};inflight={i};inbox={ip}/{ii};recovered-after-naps={r};verdict={v}"
      :g gap-ms
      :a (:wat::core::if (:wat::core::= after-visible 0) "none"
           (:wat::core::if (:wat::i64::< after-visible 0) "unread" "got"))
      :p (:wat::core::first d)
      :i (:wat::core::second d)
      :ip (:wat::core::first di)
      :ii (:wat::core::second di)
      :r recovered
      :v (:wat::core::if (:wat::i64::>= (:wat::core::first d) 1) "would-return"
           (:wat::core::if (:wat::i64::< (:wat::core::first d) 0) "unread" "empty-after-naps")))))

;; ── THE LOCKSTEP FORM ──────────────────────────────────────────────────────────────
;; Same program, same race window, same 200 ms vis-ns. Two verbs swapped back to their
;; jobs:
;;   absence  -> Queue/stats (q-depth). Non-destructive. Cannot eat what it observes.
;;   presence -> ONE Queue/receive with :wait :UpTo. Arrives on the wire. No spin.
;; Prediction: NEITHER cell stalls. gap=300 makes the race VISIBLE as pending=1 at the
;; absence check -- an assertion that names the race -- instead of swallowing it.
(:wat::core::defn :rr::take-blocking
  [q <- :queue::Queue  wait <- :queue::Queue::Wait] -> :wat::core::String
  (:wat::core::match
    (:queue::Queue/receive q
      (:queue::Queue::ReceiveRequest
        :queue "q0" :now-ns (:wat::time::epoch-nanos (:wat::time::now))
        :visibility-ns 200000000 :limit 1 :wait wait))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:queue::Queue::ReceiveResponse::Ok envs)
          (:wat::core::if (:wat::core::empty? envs)
            ""
            (:queue::Envelope/id (:wat::core::first envs))))
        (_ (:wat::kernel::assertion-failed! "take-blocking: not Ok" :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "take-blocking: recv failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :rr::run-lockstep [gap-ms <- :wat::core::i64] -> :wat::core::String
  (:wat::core::let
    [ish (:wat::query::mem-store/start :locus (:wat::spawn::thread)
           :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     iqh (:queue::queue/start :locus (:wat::spawn::thread)
           :record (:queue::queue::Record :cap 64 :store-addr (:wat::query::mem-store::Handle/addr ish) :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     ssh (:wat::query::mem-store/start :locus (:wat::spawn::thread)
           :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     sqh (:queue::queue/start :locus (:wat::spawn::thread)
           :record (:queue::queue::Record :cap 1 :store-addr (:wat::query::mem-store::Handle/addr ssh) :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     qaddrs (:wat::core::Vector :- [(:wat::kernel::Address :- [:queue::Queue::Op :queue::Queue::Reply])]
              (:queue::queue::Handle/addr sqh))
     th (:demo::topic/start :locus (:wat::spawn::thread)
          :record (:demo::topic::Record :nsubs 1 :inbox-addr (:queue::queue::Handle/addr iqh)))
     wh (:demo::topic-worker/start :locus (:wat::spawn::thread)
          :record (:demo::mk-tw 200000000 (:queue::queue::Handle/addr iqh) qaddrs 0 0))
     inbox (:demo::dial-queue (:queue::queue::Handle/addr iqh))
     subq  (:demo::dial-queue (:queue::queue::Handle/addr sqh))
     tc    (:demo::dial-topic (:demo::topic::Handle/addr th))
     tw    (:demo::dial-topic-worker (:demo::topic-worker::Handle/addr wh))
     _ (:demo::send-one subq "q0" "dummy")
     _ (:demo::start-topic-worker! tw)
     _ (:demo::publish-until-accepted! tc "hello")
     _ (:demo::require! (:demo::poll-until-unacked inbox 2000))
     dummy-id (:demo::claim-one! subq "q0" 1000000000000)
     _ (:demo::ack-one subq "q0" dummy-id)
     ;; `(await-timer-ms 0)` is itself a zero wait and has no form after Stone A —
     ;; "wait for zero" IS "don't wait", the mode-as-magnitude this arc removed.
     _ (:wat::core::if (:wat::i64::> gap-ms 0) (:demo::await-timer-ms gap-ms) nil)
     at-check (:wat::core::first (:demo::q-depth subq))
     got (:demo::receive-blocking subq "q0" 200000000
           (:queue::Queue::Wait::UpTo (:wat::time::Milliseconds 2000)))]
    (:wat::core::format "gap={g};pending-at-absence-check={c};delivered={d};raced={r}"
      :g gap-ms
      :c at-check
      :d (:wat::core::if (:wat::core::= got "") "NONE" "got")
      :r (:wat::core::if (:wat::i64::>= at-check 1) "yes-and-VISIBLE" "no"))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [a (:rr::run 0)
     b (:rr::run 300)
     c (:rr::run-lockstep 0)
     d (:rr::run-lockstep 300)]
    (:wat::core::let
      [_ (:wat::kernel::println a)
       _ (:wat::kernel::println b)
       _ (:wat::kernel::println c)]
      (:wat::kernel::println d))))
