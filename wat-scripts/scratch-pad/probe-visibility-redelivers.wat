;; probe-visibility-redelivers.wat — does an UNACKED message come back?
;;
;; The whole durable-topic design rests on retry-on-no-ack: an internal worker that fails to
;; hand a message to a subscriber simply does not ack, and the message is redelivered later.
;;
;; sqs.wat:62 states the intent -- "the message stays invisible until its visibility timeout
;; and the run ends" -- but NOTHING in the tree exercises it, and the circuit sets
;; visibility-ns to 10^12 ns (~1000 s) precisely so redelivery never happens. So the
;; mechanism everything is about to depend on has never been run.
;;
;; THREE THINGS, in one sequence, on ONE message:
;;   1. received once            -> got it, id recorded
;;   2. received again IMMEDIATELY, unacked  -> NOTHING (it is invisible, in flight)
;;   3. received again AFTER the visibility window -> THE SAME id back
;;
;; Step 2 matters as much as step 3: a queue that hands the same message to two workers at
;; once is not "redelivering", it is losing the visibility guarantee the circuit's dup=0
;; invariant rests on.

(:wat::config::set-redef! true)
(:wat::load-file! "../queue/sqs.wat")

;; Timer-channel recv, not a sleep — legal where mora forbids sleeping.
(:wat::core::defn :vr::await-timer-ms [ms <- :wat::core::i64] -> :wat::core::nil
  (:wat::core::match
    (:wat::kernel::recv
      (:wat::kernel::after :wat::program::PeerKind::thread (:wat::time::Milliseconds ms) :done))
    ((:wat::kernel::RecvOutcome::Message _m) nil)
    ((:wat::kernel::RecvOutcome::Lost _c) nil)
    (:wat::kernel::RecvOutcome::Stopped nil)
    (:wat::kernel::RecvOutcome::Closed nil)))

(:wat::core::defn :vr::dial
  [a <- (:wat::kernel::Address :- [:queue::Queue::Op :queue::Queue::Reply])] -> :queue::Queue
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected c) c)
    (_ (:wat::kernel::assertion-failed! "vr: dial failed" :wat::core::None :wat::core::None))))

;; one receive; returns the first envelope id, or "" when nothing came back
(:wat::core::defn :vr::take-one
  [q <- :queue::Queue  vis-ns <- :wat::core::i64] -> :wat::core::String
  (:wat::core::match
    (:queue::Queue/receive q
      (:queue::Queue::ReceiveRequest
        :queue "q" :now-ns (:wat::time::epoch-nanos (:wat::time::now))
        :visibility-ns vis-ns :limit 1 :wait (:queue::Queue::Wait::Immediate)))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:queue::Queue::ReceiveResponse::Ok envs)
          (:wat::core::if (:wat::core::empty? envs)
            ""
            (:queue::Envelope/id (:wat::core::first envs))))
        (_ (:wat::kernel::assertion-failed! "vr: receive not Ok" :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "vr: receive recv failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :vr::send-one [q <- :queue::Queue] -> :wat::core::nil
  (:wat::core::match
    (:queue::Queue/send q
      (:queue::Queue::SendRequest :queue "q"
        :bodies (:wat::core::Vector :- [:wat::core::String] "m0")
        :now-ns (:wat::time::epoch-nanos (:wat::time::now))))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:queue::Queue::SendResponse::Ok) nil)
        (_ (:wat::kernel::assertion-failed! "vr: send not Ok" :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "vr: send recv failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :vr::run [] -> :wat::core::String
  (:wat::core::let
    [sh (:wat::query::mem-store/start :locus (:wat::spawn::thread)
          :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     qh (:queue::queue/start :locus (:wat::spawn::thread)
          :record (:queue::queue::Record :cap 64 :store-addr (:wat::query::mem-store::Handle/addr sh) :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     q  (:vr::dial (:queue::queue::Handle/addr qh))
     _s (:vr::send-one q)
     first-id  (:vr::take-one q 200000000)          ;; 200 ms visibility, NOT acked
     while-inflight (:vr::take-one q 200000000)     ;; immediately again -> must be empty
     _n (:vr::await-timer-ms 350)                           ;; past the window
     after-expiry (:vr::take-one q 200000000)       ;; must be the SAME id
     out (:wat::core::format "first={a};while-inflight={b};after-expiry={c};same={d}"
           :a (:wat::core::if (:wat::core::= first-id "") "NONE" "got")
           :b (:wat::core::if (:wat::core::= while-inflight "") "none" "LEAKED")
           :c (:wat::core::if (:wat::core::= after-expiry "") "NONE" "got")
           :d (:wat::core::if (:wat::core::= first-id after-expiry) "yes" "no"))]
    out))

(:wat::core::defn :user::compute [] -> :wat::core::String (:vr::run))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:vr::run)))
