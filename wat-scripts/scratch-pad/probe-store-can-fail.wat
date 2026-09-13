;; Fire the queue's store Lost/Closed/TimedOut arms via the faulting proxy.
;; Disk (not the BRIEF): those arms Continue with Accepted 0 / Ack Ok — they do
;; not assertion-failed!. The product is that observed reply, plus the row in
;; the REAL store (write landed, reply destroyed).
;;
;; Run:
;;   ./target/release/wat wat-scripts/scratch-pad/probe-store-can-fail.wat
;; TimedOut path waits ~10 s (callee deadline). Die is prompt.

(:wat::config::set-redef! true)
(:wat::load-file! "../queue/sqs.wat")
(:wat::load-file! "../query/faulting-store.wat")

(:wat::core::defn :sf::dial-q
  [a <- (:wat::kernel::Address :- [:queue::Queue::Op :queue::Queue::Reply])]
  -> :queue::Queue
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    (_ (:wat::kernel::assertion-failed! "sf: dial queue failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :sf::dial-q-peer
  [a <- (:wat::kernel::Address :- [:queue::Queue::Op :queue::Queue::Reply])]
  -> (:wat::kernel::Peer :- [:queue::Queue::Op :queue::Queue::Reply])
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    (_ (:wat::kernel::assertion-failed! "sf: dial queue peer failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :sf::dial-store
  [a <- (:wat::kernel::Address :- [:wat::query::Store::Op :wat::query::Store::Reply])]
  -> (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    (_ (:wat::kernel::assertion-failed! "sf: dial store failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :sf::count-q
  [st <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
   now-ns <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::match
    (:wat::query::Store/count-index st
      (:wat::query::Store::CountIndexRequest
        :index "by-visible-at" :ipk "q"
        :isk-lo (:wat::edn::write (:wat::time::at-nanos 0))
        :isk-hi (:wat::edn::write (:wat::time::at-nanos now-ns))
        :limit 100000))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:wat::query::Store::CountIndexResponse::Ok n) n)
        (_ -1)))
    (_ -2)))

(:wat::core::defn :sf::send-label
  [q <- :queue::Queue  now-ns <- :wat::core::i64]
  -> :wat::core::String
  (:wat::core::match
    (:queue::Queue/send q
      (:queue::Queue::SendRequest
        :queue "q"
        :bodies (:wat::core::Vector :- [:wat::core::String] "body")
        :now-ns now-ns))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:queue::Queue::SendResponse::Accepted n)
          (:wat::core::format "Accepted {n}" :n n))
        (_ "other-reply")))
    ((:wat::kernel::RecvOutcome::Lost _) "Lost")
    (:wat::kernel::RecvOutcome::Closed "Closed")
    (:wat::kernel::RecvOutcome::TimedOut "TimedOut")
    (:wat::kernel::RecvOutcome::Stopped "Stopped")
    ((:wat::kernel::RecvOutcome::Malformed _) "Malformed")))

(:wat::core::defn :sf::boot
  [drop-bp <- :wat::core::i64  die-bp <- :wat::core::i64]
  -> (:wat::core::Tuple :- [:query::faulting-store::Handle
                            :queue::queue::Handle
                            :wat::query::mem-store::Handle])
  (:wat::core::let
    [sh (:wat::query::mem-store/start :locus (:wat::spawn::thread)
          :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     ph (:query::faulting-store/start :locus (:wat::spawn::thread)
          :record (:query::faulting-store::Record
                    :real-addr (:wat::query::mem-store::Handle/addr sh)
                    :drop-reply-bp drop-bp
                    :die-bp die-bp
                    :seed 1
                    :drops-fired 0
                    :dies-fired 0))
     qh (:queue::queue/start :locus (:wat::spawn::thread)
          :record (:queue::queue::Record
                    :cap 1024
                    :store-addr (:query::faulting-store::Handle/addr ph)
                    :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))]
    (:wat::core::Tuple ph qh sh)))

(:wat::core::defn :sf::run-passthrough [] -> :wat::core::String
  (:wat::core::let
    [boot (:sf::boot 0 0)
     ph (:wat::core::first boot)
     qh (:wat::core::second boot)
     sh (:wat::core::third boot)
     q (:sf::dial-q (:queue::queue::Handle/addr qh))
     now 1000000000
     t0 (:wat::time::epoch-nanos (:wat::time::now))
     lab (:sf::send-label q now)
     elapsed (:wat::i64::/ (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) t0) 1000000)
     st (:sf::dial-store (:wat::query::mem-store::Handle/addr sh))
     n (:sf::count-q st (:wat::i64::+ now 1))
     _ (:wat::service::stop-faced (:queue::queue/stop qh))
     _ (:wat::service::stop-faced (:query::faulting-store/stop ph))]
    (:wat::core::format "passthrough send={s};elapsed-ms={e};real-rows={n}"
      :s lab :e elapsed :n n)))

(:wat::core::defn :sf::run-die [] -> :wat::core::String
  (:wat::core::let
    [boot (:sf::boot 0 10000)
     ph (:wat::core::first boot)
     qh (:wat::core::second boot)
     sh (:wat::core::third boot)
     q (:sf::dial-q (:queue::queue::Handle/addr qh))
     now 2000000000
     t0 (:wat::time::epoch-nanos (:wat::time::now))
     lab (:sf::send-label q now)
     elapsed (:wat::i64::/ (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) t0) 1000000)
     st (:sf::dial-store (:wat::query::mem-store::Handle/addr sh))
     n (:sf::count-q st (:wat::i64::+ now 1))
     _ (:wat::service::stop-faced (:queue::queue/stop qh))]
    (:wat::core::format "die send={s};elapsed-ms={e};real-rows={n}"
      :s lab :e elapsed :n n)))

(:wat::core::defn :sf::run-timedout [] -> :wat::core::String
  (:wat::core::let
    [boot (:sf::boot 10000 0)
     ph (:wat::core::first boot)
     qh (:wat::core::second boot)
     sh (:wat::core::third boot)
     q (:sf::dial-q-peer (:queue::queue::Handle/addr qh))
     now 3000000000
     t0 (:wat::time::epoch-nanos (:wat::time::now))
     inert (:queue::Queue::Reply::Send (:queue::Queue::SendResponse::RequestTooLarge 0 0))
     ;; Caller-side 20 s so the queue can finish the store TimedOut arm (10 s)
     ;; and still reply Accepted 0. Generated Queue/send's 10 s deadline races it.
     co (:wat::service::call-by-deadline q
          (:queue::Queue::Op::Send
            (:queue::Queue::SendRequest
              :queue "q"
              :bodies (:wat::core::Vector :- [:wat::core::String] "body")
              :now-ns now))
          20000 inert)
     lab (:wat::core::match co
           ((:wat::service::CallOutcome::Answered r)
             (:wat::core::match r
               ((:queue::Queue::Reply::Send resp)
                 (:wat::core::match resp
                   ((:queue::Queue::SendResponse::Accepted n)
                     (:wat::core::format "Accepted {n}" :n n))
                   (_ "other-send")))
               (_ "other-reply")))
           ((:wat::service::CallOutcome::DeadlineFired) "DeadlineFired")
           ((:wat::service::CallOutcome::Closed) "Closed")
           ((:wat::service::CallOutcome::Lost _) "Lost")
           ((:wat::service::CallOutcome::Malformed _) "Malformed"))
     elapsed (:wat::i64::/ (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) t0) 1000000)
     st (:sf::dial-store (:wat::query::mem-store::Handle/addr sh))
     n (:sf::count-q st (:wat::i64::+ now 1))
     po (:query::faulting-store/stop ph)
     drops (:wat::core::match po
             ((:wat::service::StopOutcome::Stopped rec)
               (:query::faulting-store::Record/drops-fired rec))
             (_ -1))
     _ (:wat::service::stop-faced (:queue::queue/stop qh))]
    (:wat::core::format "timedout send={s};elapsed-ms={e};real-rows={n};drops-fired={d}"
      :s lab :e elapsed :n n :d drops)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println (:sf::run-passthrough))
    (:wat::kernel::println (:sf::run-die))
    (:wat::kernel::println (:sf::run-timedout))))
