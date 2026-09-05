;; probe-stats-sees-an-expired-unacked.wat — WHO reclaims an expired unacked message?
;;
;; Arc 278, R2. The drop-after circuit intermittently dies at circuit.wat:1318 with
;;   drained-never: last=[0/5] outbox=0 attempts=4000 elapsed=63565
;; visible=0, unacked=5, held for 63 SECONDS against a 200 ms visibility window
;; (circuit.wat:1181 sets vis=200000000 whenever drop-rate > 0).
;;
;; probe-visibility-redelivers.wat already proves an expired message COMES BACK -- but it
;; proves it on the RECEIVE path: it calls take-one after the window and gets the id.
;; The drain does not receive. `fanout::depth-of` reads Queue/stats and nothing else.
;;
;; So the untested question is exactly the one the drain rests on:
;;   DOES `stats` REPORT AN EXPIRED UNACKED MESSAGE AS VISIBLE AGAIN, WITH NO RECEIVER?
;;
;; Two mechanisms are live and this discriminates them:
;;   A. reclaim is EAGER (tick-driven) -> after the window, stats reads [1/0] on its own.
;;   B. reclaim is LAZY (inside receive) -> stats reads [0/1] forever, and a queue nobody
;;      receives from can never satisfy `visible == 0 AND unacked == 0`. That is a drain
;;      that cannot terminate, and it would explain [0/5] exactly.
;;
;; Cells, on ONE message, one queue, no workers:
;;   t0  send + receive (200 ms vis, NOT acked)   -> stats: expect [0/1]
;;   t1  wait 350 ms, PAST the window, NO receive -> stats: A=[1/0]  B=[0/1]   <- THE ROW
;;   t2  now receive                              -> got it back? (the exemplar's claim)
;;   t3  stats again                              -> what the receive changed

(:wat::config::set-redef! true)
(:wat::load-file! "../queue/sqs.wat")

(:wat::core::defn :su::await-timer-ms [ms <- :wat::core::i64] -> :wat::core::nil
  (:wat::core::match
    (:wat::kernel::recv
      (:wat::kernel::after :wat::program::PeerKind::thread (:wat::time::Milliseconds ms) :done))
    ((:wat::kernel::RecvOutcome::Message _m) nil)
    ((:wat::kernel::RecvOutcome::Lost _c) nil)
    (:wat::kernel::RecvOutcome::Stopped nil)
    (:wat::kernel::RecvOutcome::Closed nil) (:wat::kernel::RecvOutcome::TimedOut nil)))

(:wat::core::defn :su::dial
  [a <- (:wat::kernel::Address :- [:queue::Queue::Op :queue::Queue::Reply])] -> :queue::Queue
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected c) c)
    (_ (:wat::kernel::assertion-failed! "su: dial failed" :wat::core::None :wat::core::None))))

;; visible/unacked as the drain sees them -- Queue/stats, never a receive.
(:wat::core::defn :su::depth [q <- :queue::Queue] -> :wat::core::String
  (:wat::core::match (:queue::Queue/stats q (:queue::Queue::StatsRequest))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:queue::Queue::StatsResponse::Ok _calls _ticks visible unacked)
          (:wat::core::format "[{v}/{u}]" :v visible :u unacked))
        (_ "[stats-not-ok]")))
    (_ "[stats-lost]")))

(:wat::core::defn :su::take-one
  [q <- :queue::Queue  vis-ns <- :wat::core::i64] -> :wat::core::String
  (:wat::core::match
    (:queue::Queue/receive q
      (:queue::Queue::ReceiveRequest
        :queue "q" :now-ns (:wat::time::epoch-nanos (:wat::time::now))
        :visibility-ns vis-ns :limit 1 :wait (:queue::Queue::Wait::Immediate)))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:queue::Queue::ReceiveResponse::Ok envs)
          (:wat::core::if (:wat::core::empty? envs) "" (:queue::Envelope/id (:wat::core::first envs))))
        (_ (:wat::kernel::assertion-failed! "su: receive not Ok" :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "su: receive recv failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :su::send-one [q <- :queue::Queue] -> :wat::core::nil
  (:wat::core::match
    (:queue::Queue/send q
      (:queue::Queue::SendRequest :queue "q"
        :bodies (:wat::core::Vector :- [:wat::core::String] "m0")
        :now-ns (:wat::time::epoch-nanos (:wat::time::now))))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:queue::Queue::SendResponse::Ok) nil)
        (_ (:wat::kernel::assertion-failed! "su: send not Ok" :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "su: send recv failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :su::ack-one [q <- :queue::Queue  id <- :wat::core::String] -> :wat::core::nil
  (:wat::core::match (:queue::Queue/ack q (:queue::Queue::AckRequest :queue "q" :id id))
    ((:wat::kernel::RecvOutcome::Message _r) nil)
    (_ nil)))

(:wat::core::defn :su::run [] -> :wat::core::String
  (:wat::core::let
    [sh (:wat::query::mem-store/start :locus (:wat::spawn::thread)
          :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     qh (:queue::queue/start :locus (:wat::spawn::thread)
          :record (:queue::queue::Record :cap 64 :store-addr (:wat::query::mem-store::Handle/addr sh) :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     q  (:su::dial (:queue::queue::Handle/addr qh))
     _s (:su::send-one q)
     d-sent (:su::depth q)
     id0    (:su::take-one q 200000000)
     d-held (:su::depth q)
     _w     (:su::await-timer-ms 350)
     d-expired (:su::depth q)
     id1    (:su::take-one q 200000000)
     d-after   (:su::depth q)
     _a        (:su::ack-one q id1)
     d-acked   (:su::depth q)]
    (:wat::core::format
      "sent={a};held={b};EXPIRED-NO-RECEIVER={c};came-back={d};after-receive={e};AFTER-ACK={f}"
      :a d-sent :b d-held :c d-expired
      :d (:wat::core::if (:wat::core::= id1 "") "NONE"
           (:wat::core::if (:wat::core::= id0 id1) "same-id" "DIFFERENT-id"))
      :e d-after :f d-acked)))

(:wat::core::defn :user::compute [] -> :wat::core::String (:su::run))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:su::run)))
