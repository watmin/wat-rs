;; probe-transient-means-try-again.wat — honest SQLITE_BUSY: fail without applying, N times, then succeed.
;;
;; The step-1 wrapper applied k of n then reported Transient (a lying store).
;; This wrapper returns Transient WITHOUT writing, then forwards. That is what
;; a real SQLITE_BUSY looks like, and it is the only case this stone claims to fix.

(:wat::config::set-redef! true)
(:wat::load-file! "../queue/sqs.wat")

(:wat::service::defservice :bs::busy-store
  :satisfies :wat::query::Store
  :durable   [inner-addr       <- (:wat::kernel::Address :- [:wat::query::Store::Op :wat::query::Store::Reply])
              put-busy-left    <- :wat::core::i64
              delete-busy-left <- :wat::core::i64
              fail-kind        <- :wat::core::i64]
  :ephemeral [inner <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])]
  :peers     [:wat::query::Store]
  :init (:wat::core::fn [record <- :bs::busy-store::Record] -> :bs::busy-store::State
          (:wat::core::let
            [addr (:bs::busy-store::Record/inner-addr record)
             inner (:wat::core::match (:wat::kernel::connect addr)
                     ((:wat::kernel::ConnectOutcome::Connected p) p)
                     (_ (:wat::kernel::assertion-failed! "bs: inner dial failed" :wat::core::None :wat::core::None)))]
            (:bs::busy-store::State :durable record :inner inner)))
  :impls
  [(ensure-schema [s ctx req]
     (:wat::core::let
       [inner (:bs::busy-store::State/inner s)
        sends (:wat::core::Vector :- [(:wat::service::Directed :- [:wat::query::Store::Reply])])
        none  (:wat::core::Vector :- [(:wat::service::Alarm :- [:bs::busy-store::Op])])]
       (:wat::core::match (:wat::query::Store/ensure-schema inner req)
         ((:wat::kernel::RecvOutcome::Message r)
           (:wat::service::Outcome::Continue s
             (:wat::core::Some (:wat::query::Store::Reply::EnsureSchema r)) sends none))
         (_ (:wat::kernel::assertion-failed! "bs: inner ensure-schema failed" :wat::core::None :wat::core::None)))))

   (put [s ctx req]
     (:wat::core::let
       [inner (:bs::busy-store::State/inner s)
        rec   (:bs::busy-store::State/durable s)
        kind  (:bs::busy-store::Record/fail-kind rec)
        left  (:bs::busy-store::Record/put-busy-left rec)
        sends (:wat::core::Vector :- [(:wat::service::Directed :- [:wat::query::Store::Reply])])
        none  (:wat::core::Vector :- [(:wat::service::Alarm :- [:bs::busy-store::Op])])
        fault (:wat::query::Fault :message "busy-store")]
       (:wat::core::if (:wat::i64::= kind 1)
         (:wat::service::Outcome::Continue s
           (:wat::core::Some (:wat::query::Store::Reply::Put
             (:wat::query::Store::PutResponse::Constraint
               (:wat::query::Constraint :reason fault))))
           sends none)
         (:wat::core::if (:wat::i64::= kind 2)
           (:wat::service::Outcome::Continue s
             (:wat::core::Some (:wat::query::Store::Reply::Put
               (:wat::query::Store::PutResponse::Fatal
                 (:wat::query::Fatal :reason fault))))
             sends none)
           (:wat::core::if (:wat::i64::> left 0)
             (:wat::core::let
               [rec' (:bs::busy-store::Record
                       :inner-addr (:bs::busy-store::Record/inner-addr rec)
                       :put-busy-left (:wat::i64::- left 1)
                       :delete-busy-left (:bs::busy-store::Record/delete-busy-left rec)
                       :fail-kind kind)
                s' (:bs::busy-store::State :durable rec' :inner inner)]
               (:wat::service::Outcome::Continue s'
                 (:wat::core::Some (:wat::query::Store::Reply::Put
                   (:wat::query::Store::PutResponse::Transient
                     (:wat::query::Transient :reason fault))))
                 sends none))
             (:wat::core::match (:wat::query::Store/put inner req)
               ((:wat::kernel::RecvOutcome::Message r)
                 (:wat::service::Outcome::Continue s
                   (:wat::core::Some (:wat::query::Store::Reply::Put r)) sends none))
               (_ (:wat::kernel::assertion-failed! "bs: inner put failed" :wat::core::None :wat::core::None))))))))

   (delete [s ctx req]
     (:wat::core::let
       [inner (:bs::busy-store::State/inner s)
        rec   (:bs::busy-store::State/durable s)
        kind  (:bs::busy-store::Record/fail-kind rec)
        left  (:bs::busy-store::Record/delete-busy-left rec)
        sends (:wat::core::Vector :- [(:wat::service::Directed :- [:wat::query::Store::Reply])])
        none  (:wat::core::Vector :- [(:wat::service::Alarm :- [:bs::busy-store::Op])])
        fault (:wat::query::Fault :message "busy-store")]
       (:wat::core::if (:wat::i64::= kind 1)
         (:wat::service::Outcome::Continue s
           (:wat::core::Some (:wat::query::Store::Reply::Delete
             (:wat::query::Store::DeleteResponse::Constraint
               (:wat::query::Constraint :reason fault))))
           sends none)
         (:wat::core::if (:wat::i64::= kind 2)
           (:wat::service::Outcome::Continue s
             (:wat::core::Some (:wat::query::Store::Reply::Delete
               (:wat::query::Store::DeleteResponse::Fatal
                 (:wat::query::Fatal :reason fault))))
             sends none)
           (:wat::core::if (:wat::i64::> left 0)
             (:wat::core::let
               [rec' (:bs::busy-store::Record
                       :inner-addr (:bs::busy-store::Record/inner-addr rec)
                       :put-busy-left (:bs::busy-store::Record/put-busy-left rec)
                       :delete-busy-left (:wat::i64::- left 1)
                       :fail-kind kind)
                s' (:bs::busy-store::State :durable rec' :inner inner)]
               (:wat::service::Outcome::Continue s'
                 (:wat::core::Some (:wat::query::Store::Reply::Delete
                   (:wat::query::Store::DeleteResponse::Transient
                     (:wat::query::Transient :reason fault))))
                 sends none))
             (:wat::core::match (:wat::query::Store/delete inner req)
               ((:wat::kernel::RecvOutcome::Message r)
                 (:wat::service::Outcome::Continue s
                   (:wat::core::Some (:wat::query::Store::Reply::Delete r)) sends none))
               (_ (:wat::kernel::assertion-failed! "bs: inner delete failed" :wat::core::None :wat::core::None))))))))

   (scan [s ctx req]
     (:wat::core::let
       [inner (:bs::busy-store::State/inner s)
        sends (:wat::core::Vector :- [(:wat::service::Directed :- [:wat::query::Store::Reply])])
        none  (:wat::core::Vector :- [(:wat::service::Alarm :- [:bs::busy-store::Op])])]
       (:wat::core::match (:wat::query::Store/scan inner req)
         ((:wat::kernel::RecvOutcome::Message r)
           (:wat::service::Outcome::Continue s
             (:wat::core::Some (:wat::query::Store::Reply::Scan r)) sends none))
         (_ (:wat::kernel::assertion-failed! "bs: inner scan failed" :wat::core::None :wat::core::None)))))

   (scan-index [s ctx req]
     (:wat::core::let
       [inner (:bs::busy-store::State/inner s)
        sends (:wat::core::Vector :- [(:wat::service::Directed :- [:wat::query::Store::Reply])])
        none  (:wat::core::Vector :- [(:wat::service::Alarm :- [:bs::busy-store::Op])])]
       (:wat::core::match (:wat::query::Store/scan-index inner req)
         ((:wat::kernel::RecvOutcome::Message r)
           (:wat::service::Outcome::Continue s
             (:wat::core::Some (:wat::query::Store::Reply::ScanIndex r)) sends none))
         (_ (:wat::kernel::assertion-failed! "bs: inner scan-index failed" :wat::core::None :wat::core::None)))))

   (count-index [s ctx req]
     (:wat::core::let
       [inner (:bs::busy-store::State/inner s)
        sends (:wat::core::Vector :- [(:wat::service::Directed :- [:wat::query::Store::Reply])])
        none  (:wat::core::Vector :- [(:wat::service::Alarm :- [:bs::busy-store::Op])])]
       (:wat::core::match (:wat::query::Store/count-index inner req)
         ((:wat::kernel::RecvOutcome::Message r)
           (:wat::service::Outcome::Continue s
             (:wat::core::Some (:wat::query::Store::Reply::CountIndex r)) sends none))
         (_ (:wat::kernel::assertion-failed! "bs: inner count-index failed" :wat::core::None :wat::core::None)))))])

(:wat::core::defn :tr::bodies [] -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String])  i <- :wat::core::i64]
      -> (:wat::core::Vector :- [:wat::core::String])
      (:wat::core::conj acc (:wat::core::format "m{i}" :i i)))
    (:wat::core::Vector :- [:wat::core::String])
    (:wat::core::range 0 10)))

(:wat::core::defn :tr::dial-q
  [a <- (:wat::kernel::Address :- [:queue::Queue::Op :queue::Queue::Reply])]
  -> :queue::Queue
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected c) c)
    (_ (:wat::kernel::assertion-failed! "tr: queue dial failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :tr::dial-store
  [a <- (:wat::kernel::Address :- [:wat::query::Store::Op :wat::query::Store::Reply])]
  -> :wat::query::Store
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected c) c)
    (_ (:wat::kernel::assertion-failed! "tr: store dial failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :tr::send-tag
  [q <- :queue::Queue  name <- :wat::core::String  now-ns <- :wat::core::i64]
  -> :wat::core::String
  (:wat::core::match
    (:queue::Queue/send q
      (:queue::Queue::SendRequest :queue name :bodies (:tr::bodies) :now-ns now-ns))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:queue::Queue::SendResponse::Ok) "Ok")
        ((:queue::Queue::SendResponse::Full _d _c) "Full")
        ((:queue::Queue::SendResponse::RequestTooLarge _b _c) "RequestTooLarge")
        ((:queue::Queue::SendResponse::RequestMalformed _p _e _g) "RequestMalformed")))
    ((:wat::kernel::RecvOutcome::Lost c)
      (:wat::core::format "Lost:{m}" :m (:wat::kernel::LociDiedError/message c)))
    (:wat::kernel::RecvOutcome::Closed "Closed")
    (:wat::kernel::RecvOutcome::Stopped "Stopped")
    (:wat::kernel::RecvOutcome::TimedOut "TimedOut")))

(:wat::core::defn :tr::ack-tag
  [q <- :queue::Queue  name <- :wat::core::String  ids <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::String
  (:wat::core::match
    (:queue::Queue/ack q (:queue::Queue::AckRequest :queue name :ids ids))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:queue::Queue::AckResponse::Ok) "Ok")
        ((:queue::Queue::AckResponse::RequestTooLarge _b _c) "RequestTooLarge")
        ((:queue::Queue::AckResponse::RequestMalformed _p _e _g) "RequestMalformed")))
    ((:wat::kernel::RecvOutcome::Lost c)
      (:wat::core::format "Lost:{m}" :m (:wat::kernel::LociDiedError/message c)))
    (:wat::kernel::RecvOutcome::Closed "Closed")
    (:wat::kernel::RecvOutcome::Stopped "Stopped")
    (:wat::kernel::RecvOutcome::TimedOut "TimedOut")))

(:wat::core::defn :tr::recv-ids
  [q <- :queue::Queue  name <- :wat::core::String  now-ns <- :wat::core::i64]
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::match
    (:queue::Queue/receive q
      (:queue::Queue::ReceiveRequest
        :queue name :now-ns now-ns :visibility-ns 100 :limit 10
        :wait (:queue::Queue::Wait::Immediate)))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:queue::Queue::ReceiveResponse::Ok envs)
          (:wat::core::foldl
            (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String])  e <- :queue::Envelope]
              -> (:wat::core::Vector :- [:wat::core::String])
              (:wat::core::conj acc (:queue::Envelope/id e)))
            (:wat::core::Vector :- [:wat::core::String])
            envs))
        (_ (:wat::kernel::assertion-failed! "tr: receive not Ok" :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "tr: receive recv failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :tr::scan-td
  [st <- :wat::query::Store  q <- :wat::core::String  now-ns <- :wat::core::i64]
  -> (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])
  (:wat::core::match
    (:wat::query::Store/scan-index st
      (:wat::query::Store::ScanIndexRequest
        :index "by-visible-at" :ipk q
        :isk-lo (:wat::edn::write (:wat::time::at-nanos 0))
        :isk-hi (:wat::edn::write (:wat::time::at-nanos now-ns))
        :limit 100 :cursor :wat::core::None))
    ((:wat::kernel::RecvOutcome::Message sresp)
      (:wat::core::match sresp
        ((:wat::query::Store::ScanIndexResponse::Success irows _c)
          (:wat::core::let
            [id-map (:wat::core::foldl
                      (:wat::core::fn
                        [acc <- (:wat::core::HashMap :- [:wat::core::String :wat::core::bool])
                         r   <- :wat::query::IndexRow]
                        -> (:wat::core::HashMap :- [:wat::core::String :wat::core::bool])
                        (:wat::hashmap::assoc acc (:wat::query::IndexRow/data r) true))
                      (:wat::core::HashMap :- [:wat::core::String :wat::core::bool])
                      irows)]
            (:wat::core::Tuple (:wat::core::count irows) (:wat::core::count (:wat::hashmap::keys id-map)))))
        (_ (:wat::kernel::assertion-failed! "tr: scan-index not Success" :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "tr: scan-index recv failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :tr::cell-retry [] -> :wat::core::String
  (:wat::core::let
    [T0 1000000000
     msh (:wat::query::mem-store/start :locus (:wat::spawn::thread)
            :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     bsh (:bs::busy-store/start :locus (:wat::spawn::thread)
            :record (:bs::busy-store::Record
                      :inner-addr (:wat::query::mem-store::Handle/addr msh)
                      :put-busy-left 2 :delete-busy-left 0 :fail-kind 0))
     qh  (:queue::queue/start :locus (:wat::spawn::thread)
            :record (:queue::queue::Record
                      :cap 1024 :store-addr (:bs::busy-store::Handle/addr bsh)
                      :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     q   (:tr::dial-q (:queue::queue::Handle/addr qh))
     mem (:tr::dial-store (:wat::query::mem-store::Handle/addr msh))
     snd (:tr::send-tag q "q" T0)
     td  (:tr::scan-td mem "q" (:wat::i64::+ T0 100))]
    (:wat::core::format "send={s};total={t};distinct={d}"
      :s snd :t (:wat::core::first td) :d (:wat::core::second td))))

(:wat::core::defn :tr::cell-exhaust [] -> :wat::core::String
  (:wat::core::let
    [msh (:wat::query::mem-store/start :locus (:wat::spawn::thread)
            :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     bsh (:bs::busy-store/start :locus (:wat::spawn::thread)
            :record (:bs::busy-store::Record
                      :inner-addr (:wat::query::mem-store::Handle/addr msh)
                      :put-busy-left 100 :delete-busy-left 0 :fail-kind 0))
     qh  (:queue::queue/start :locus (:wat::spawn::thread)
            :record (:queue::queue::Record
                      :cap 1024 :store-addr (:bs::busy-store::Handle/addr bsh)
                      :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     q   (:tr::dial-q (:queue::queue::Handle/addr qh))]
    (:wat::core::format "send={s}" :s (:tr::send-tag q "q" 1000000000))))

(:wat::core::defn :tr::cell-constraint [] -> :wat::core::String
  (:wat::core::let
    [msh (:wat::query::mem-store/start :locus (:wat::spawn::thread)
            :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     bsh (:bs::busy-store/start :locus (:wat::spawn::thread)
            :record (:bs::busy-store::Record
                      :inner-addr (:wat::query::mem-store::Handle/addr msh)
                      :put-busy-left 0 :delete-busy-left 0 :fail-kind 1))
     qh  (:queue::queue/start :locus (:wat::spawn::thread)
            :record (:queue::queue::Record
                      :cap 1024 :store-addr (:bs::busy-store::Handle/addr bsh)
                      :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     q   (:tr::dial-q (:queue::queue::Handle/addr qh))]
    (:wat::core::format "send={s}" :s (:tr::send-tag q "q" 1000000000))))

(:wat::core::defn :tr::cell-fatal [] -> :wat::core::String
  (:wat::core::let
    [msh (:wat::query::mem-store/start :locus (:wat::spawn::thread)
            :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     bsh (:bs::busy-store/start :locus (:wat::spawn::thread)
            :record (:bs::busy-store::Record
                      :inner-addr (:wat::query::mem-store::Handle/addr msh)
                      :put-busy-left 0 :delete-busy-left 0 :fail-kind 2))
     qh  (:queue::queue/start :locus (:wat::spawn::thread)
            :record (:queue::queue::Record
                      :cap 1024 :store-addr (:bs::busy-store::Handle/addr bsh)
                      :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     q   (:tr::dial-q (:queue::queue::Handle/addr qh))]
    (:wat::core::format "send={s}" :s (:tr::send-tag q "q" 1000000000))))

(:wat::core::defn :tr::cell-ack [] -> :wat::core::String
  (:wat::core::let
    [T0 1000000000
     msh (:wat::query::mem-store/start :locus (:wat::spawn::thread)
            :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     bsh (:bs::busy-store/start :locus (:wat::spawn::thread)
            :record (:bs::busy-store::Record
                      :inner-addr (:wat::query::mem-store::Handle/addr msh)
                      :put-busy-left 0 :delete-busy-left 2 :fail-kind 0))
     qh  (:queue::queue/start :locus (:wat::spawn::thread)
            :record (:queue::queue::Record
                      :cap 1024 :store-addr (:bs::busy-store::Handle/addr bsh)
                      :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     q   (:tr::dial-q (:queue::queue::Handle/addr qh))
     _s  (:tr::send-tag q "q" T0)
     ids (:tr::recv-ids q "q" (:wat::i64::+ T0 20))
     a   (:tr::ack-tag q "q" ids)]
    (:wat::core::format "got={g};ack={a}" :g (:wat::core::count ids) :a a)))

(:wat::core::defn :tr::run [] -> :wat::core::String
  (:wat::core::format "RETRY={a};EXHAUST={b};CONSTRAINT={c};FATAL={d};ACK={e}"
    :a (:tr::cell-retry)
    :b (:tr::cell-exhaust)
    :c (:tr::cell-constraint)
    :d (:tr::cell-fatal)
    :e (:tr::cell-ack)))

(:wat::core::defn :user::compute [] -> :wat::core::String (:tr::run))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println (:tr::run)))
