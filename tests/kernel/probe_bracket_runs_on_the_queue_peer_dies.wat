;; Peer-path disposition on the SAME RecvOutcome::TimedOut the queue client
;; surfaces (call-by-deadline DeadlineFired). drop-recv-bp=10000 always drops
;; the reply, so TimedOut fires. The five runner-loop arms panic with this
;; exact string; this program is that match, driven by a fault the queue
;; path retries.

(:wat::core::defn :user::compute [] -> :wat::core::nil
  (:wat::core::let
    [msh (:wat::query::mem-store/start :locus (:wat::spawn::thread)
           :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     qh  (:wat::queue::queue/start :locus (:wat::spawn::thread)
           :record (:wat::queue::queue::Record
                     :cap 1024
                     :store-addr (:wat::query::mem-store::Handle/addr msh)
                     :drop-recv-bp 10000 :drop-ack-bp 0 :drop-seed 1))
     addr (:wat::queue::queue::Handle/addr qh)
     q    (:wat::core::match (:wat::kernel::connect addr)
            ((:wat::kernel::ConnectOutcome::Connected p) p)
            ((:wat::kernel::ConnectOutcome::Refused c)
              (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
            ((:wat::kernel::ConnectOutcome::Rejected c)
              (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
            ((:wat::kernel::ConnectOutcome::Failed c)
              (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     _send (:wat::core::match
             (:wat::queue::Queue/send q
               (:wat::queue::Queue::SendRequest
                 :queue "peer"
                 :bodies (:wat::core::Vector :- [:wat::core::String] "1")
                 :now-ns (:wat::time::epoch-nanos (:wat::time::now))))
             ((:wat::kernel::RecvOutcome::Message _r) nil)
             (_ nil))
     empty (:wat::core::Vector :- [:wat::queue::Envelope])
     inert (:wat::queue::Queue::Reply::Receive
             (:wat::queue::Queue::ReceiveResponse::Ok empty))
     req   (:wat::queue::Queue::ReceiveRequest
             :queue "peer"
             :now-ns (:wat::time::epoch-nanos (:wat::time::now))
             :visibility-ns 1000000000
             :limit 1
             :wait (:wat::queue::Queue::Wait::Immediate))]
    (:wat::core::match
      (:wat::service::call-by-deadline q (:wat::queue::Queue::Op::Receive req) 200 inert)
      ((:wat::service::CallOutcome::Answered _r) nil)
      ((:wat::service::CallOutcome::DeadlineFired)
        (:wat::kernel::assertion-failed!
          "recv: timed out — the peer is alive and silent"
          :wat::core::None :wat::core::None))
      ((:wat::service::CallOutcome::Lost _c) nil)
      ((:wat::service::CallOutcome::Closed) nil)
      ((:wat::service::CallOutcome::Malformed _cause)
        (:wat::kernel::assertion-failed!
          "recv: malformed frame — the peer could not decode our message; this arm is an UNMIGRATED PLACEHOLDER (a-momentary-failure-is-not-fatal, stone 2 replaces it with report-final)"
          :wat::core::None :wat::core::None)))))
