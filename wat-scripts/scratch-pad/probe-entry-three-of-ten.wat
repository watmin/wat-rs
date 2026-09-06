;; probe-entry-three-of-ten.wat — provoke per-entry batch failure and measure.
;;
;; Arc 278. Every Store write is vector-in, one-outcome-out. :Transient on ten keys
;; cannot name which landed. This stone does not fix; it injects entry 3 of 10 failing
;; while 1, 2, 4–10 succeed, and reports what the stack does.
;;
;; :fs::failing-store satisfies Store, holds a real store peer, forwards every verb
;; unchanged except put/delete: at put-fail-bp / delete-fail-bp, it APPLIES k of n
;; (drops index 2 when n>2) then returns the only variant the surface allows.
;; A wrapper that errors without writing k of n models "the store did nothing" —
;; already handled, and not this fault.
;;
;; Two cells, each its own mem-store + failing-store + queue:
;;   PUT  — send a batch of 10 with put-fail-bp on. Store response, SendResponse,
;;          then drain: total / distinct.
;;   DEL  — send 10 cleanly, receive, ack with delete-fail-bp on. Store response,
;;          AckResponse, whether the remaining row redelivers.

(:wat::config::set-redef! true)
(:wat::load-file! "../queue/sqs.wat")

;; ── wrapper ──────────────────────────────────────────────────────────────────
(:wat::service::defservice :fs::failing-store
  :satisfies :wat::query::Store
  :durable   [inner-addr     <- (:wat::kernel::Address :- [:wat::query::Store::Op :wat::query::Store::Reply])
              put-fail-bp    <- :wat::core::i64
              delete-fail-bp <- :wat::core::i64
              drop-seed      <- :wat::core::i64]
  :ephemeral [inner <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])]
  :peers     [:wat::query::Store]
  :init (:wat::core::fn [record <- :fs::failing-store::Record] -> :fs::failing-store::State
          (:wat::core::let
            [addr (:fs::failing-store::Record/inner-addr record)
             inner (:wat::core::match (:wat::kernel::connect addr)
                     ((:wat::kernel::ConnectOutcome::Connected p) p)
                     (_ (:wat::kernel::assertion-failed! "fs: inner dial failed" :wat::core::None :wat::core::None)))]
            (:fs::failing-store::State :durable record :inner inner)))
  :impls
  [(ensure-schema [s ctx req]
     (:wat::core::let
       [inner (:fs::failing-store::State/inner s)
        sends (:wat::core::Vector :- [(:wat::service::Directed :- [:wat::query::Store::Reply])])
        none  (:wat::core::Vector :- [(:wat::service::Alarm :- [:fs::failing-store::Op])])]
       (:wat::core::match (:wat::query::Store/ensure-schema inner req)
         ((:wat::kernel::RecvOutcome::Message r)
           (:wat::service::Outcome::Continue s
             (:wat::core::Some (:wat::query::Store::Reply::EnsureSchema r)) sends none))
         (_ (:wat::kernel::assertion-failed! "fs: inner ensure-schema failed" :wat::core::None :wat::core::None)))))

   (put [s ctx req]
     (:wat::core::let
       [inner (:fs::failing-store::State/inner s)
        rec   (:fs::failing-store::State/durable s)
        rows  (:wat::query::Store::PutRequest/rows req)
        n     (:wat::core::count rows)
        rate  (:fs::failing-store::Record/put-fail-bp rec)
        sends (:wat::core::Vector :- [(:wat::service::Directed :- [:wat::query::Store::Reply])])
        none  (:wat::core::Vector :- [(:wat::service::Alarm :- [:fs::failing-store::Op])])
        pair  (:wat::core::if (:wat::i64::> rate 0)
                (:wat::rand::int-from (:fs::failing-store::Record/drop-seed rec) 0 10000)
                (:wat::core::Tuple (:fs::failing-store::Record/drop-seed rec) 0))
        seed1 (:wat::core::first pair)
        bp    (:wat::core::second pair)
        hit?  (:wat::core::and (:wat::i64::> rate 0) (:wat::i64::< bp rate) (:wat::i64::> n 0))
        rec'  (:fs::failing-store::Record
                :inner-addr (:fs::failing-store::Record/inner-addr rec)
                :put-fail-bp rate
                :delete-fail-bp (:fs::failing-store::Record/delete-fail-bp rec)
                :drop-seed seed1)
        s'    (:fs::failing-store::State :durable rec' :inner inner)]
       (:wat::core::if hit?
         (:wat::core::let
           [drop-i (:wat::core::if (:wat::i64::> n 2) 2 (:wat::i64::- n 1))
            kept (:wat::core::foldl
                   (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::query::StoredRow])  i <- :wat::core::i64]
                     -> (:wat::core::Vector :- [:wat::query::StoredRow])
                     (:wat::core::if (:wat::i64::= i drop-i)
                       acc
                       (:wat::core::conj acc (:wat::core::nth rows i))))
                   (:wat::core::Vector :- [:wat::query::StoredRow])
                   (:wat::core::range 0 n))
            applied (:wat::core::if (:wat::core::empty? kept)
                       (:wat::query::Store::PutResponse::Success)
                       (:wat::core::match
                         (:wat::query::Store/put inner (:wat::query::Store::PutRequest kept))
                         ((:wat::kernel::RecvOutcome::Message r) r)
                         (_ (:wat::kernel::assertion-failed! "fs: inner partial put failed" :wat::core::None :wat::core::None))))]
           (:wat::core::match applied
             ((:wat::query::Store::PutResponse::Success)
               (:wat::service::Outcome::Continue s'
                 (:wat::core::Some (:wat::query::Store::Reply::Put
                   (:wat::query::Store::PutResponse::Transient
                     (:wat::query::Transient
                       :reason (:wat::query::Fault
                                 :message "failing-store: applied k of n, dropped entry 3")))))
                 sends none))
             (_ (:wat::kernel::assertion-failed! "fs: inner partial put not Success" :wat::core::None :wat::core::None))))
         (:wat::core::match (:wat::query::Store/put inner req)
           ((:wat::kernel::RecvOutcome::Message r)
             (:wat::service::Outcome::Continue s'
               (:wat::core::Some (:wat::query::Store::Reply::Put r)) sends none))
           (_ (:wat::kernel::assertion-failed! "fs: inner put failed" :wat::core::None :wat::core::None))))))

   (delete [s ctx req]
     (:wat::core::let
       [inner (:fs::failing-store::State/inner s)
        rec   (:fs::failing-store::State/durable s)
        keys  (:wat::query::Store::DeleteRequest/keys req)
        n     (:wat::core::count keys)
        rate  (:fs::failing-store::Record/delete-fail-bp rec)
        sends (:wat::core::Vector :- [(:wat::service::Directed :- [:wat::query::Store::Reply])])
        none  (:wat::core::Vector :- [(:wat::service::Alarm :- [:fs::failing-store::Op])])
        pair  (:wat::core::if (:wat::i64::> rate 0)
                (:wat::rand::int-from (:fs::failing-store::Record/drop-seed rec) 0 10000)
                (:wat::core::Tuple (:fs::failing-store::Record/drop-seed rec) 0))
        seed1 (:wat::core::first pair)
        bp    (:wat::core::second pair)
        hit?  (:wat::core::and (:wat::i64::> rate 0) (:wat::i64::< bp rate) (:wat::i64::> n 0))
        rec'  (:fs::failing-store::Record
                :inner-addr (:fs::failing-store::Record/inner-addr rec)
                :put-fail-bp (:fs::failing-store::Record/put-fail-bp rec)
                :delete-fail-bp rate
                :drop-seed seed1)
        s'    (:fs::failing-store::State :durable rec' :inner inner)]
       (:wat::core::if hit?
         (:wat::core::let
           [drop-i (:wat::core::if (:wat::i64::> n 2) 2 (:wat::i64::- n 1))
            kept (:wat::core::foldl
                   (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::query::Key])  i <- :wat::core::i64]
                     -> (:wat::core::Vector :- [:wat::query::Key])
                     (:wat::core::if (:wat::i64::= i drop-i)
                       acc
                       (:wat::core::conj acc (:wat::core::nth keys i))))
                   (:wat::core::Vector :- [:wat::query::Key])
                   (:wat::core::range 0 n))
            applied (:wat::core::if (:wat::core::empty? kept)
                       (:wat::query::Store::DeleteResponse::Success)
                       (:wat::core::match
                         (:wat::query::Store/delete inner (:wat::query::Store::DeleteRequest kept))
                         ((:wat::kernel::RecvOutcome::Message r) r)
                         (_ (:wat::kernel::assertion-failed! "fs: inner partial delete failed" :wat::core::None :wat::core::None))))]
           (:wat::core::match applied
             ((:wat::query::Store::DeleteResponse::Success)
               (:wat::service::Outcome::Continue s'
                 (:wat::core::Some (:wat::query::Store::Reply::Delete
                   (:wat::query::Store::DeleteResponse::Transient
                     (:wat::query::Transient
                       :reason (:wat::query::Fault
                                 :message "failing-store: applied k of n, dropped entry 3")))))
                 sends none))
             (_ (:wat::kernel::assertion-failed! "fs: inner partial delete not Success" :wat::core::None :wat::core::None))))
         (:wat::core::match (:wat::query::Store/delete inner req)
           ((:wat::kernel::RecvOutcome::Message r)
             (:wat::service::Outcome::Continue s'
               (:wat::core::Some (:wat::query::Store::Reply::Delete r)) sends none))
           (_ (:wat::kernel::assertion-failed! "fs: inner delete failed" :wat::core::None :wat::core::None))))))

   (scan [s ctx req]
     (:wat::core::let
       [inner (:fs::failing-store::State/inner s)
        sends (:wat::core::Vector :- [(:wat::service::Directed :- [:wat::query::Store::Reply])])
        none  (:wat::core::Vector :- [(:wat::service::Alarm :- [:fs::failing-store::Op])])]
       (:wat::core::match (:wat::query::Store/scan inner req)
         ((:wat::kernel::RecvOutcome::Message r)
           (:wat::service::Outcome::Continue s
             (:wat::core::Some (:wat::query::Store::Reply::Scan r)) sends none))
         (_ (:wat::kernel::assertion-failed! "fs: inner scan failed" :wat::core::None :wat::core::None)))))

   (scan-index [s ctx req]
     (:wat::core::let
       [inner (:fs::failing-store::State/inner s)
        sends (:wat::core::Vector :- [(:wat::service::Directed :- [:wat::query::Store::Reply])])
        none  (:wat::core::Vector :- [(:wat::service::Alarm :- [:fs::failing-store::Op])])]
       (:wat::core::match (:wat::query::Store/scan-index inner req)
         ((:wat::kernel::RecvOutcome::Message r)
           (:wat::service::Outcome::Continue s
             (:wat::core::Some (:wat::query::Store::Reply::ScanIndex r)) sends none))
         (_ (:wat::kernel::assertion-failed! "fs: inner scan-index failed" :wat::core::None :wat::core::None)))))

   (count-index [s ctx req]
     (:wat::core::let
       [inner (:fs::failing-store::State/inner s)
        sends (:wat::core::Vector :- [(:wat::service::Directed :- [:wat::query::Store::Reply])])
        none  (:wat::core::Vector :- [(:wat::service::Alarm :- [:fs::failing-store::Op])])]
       (:wat::core::match (:wat::query::Store/count-index inner req)
         ((:wat::kernel::RecvOutcome::Message r)
           (:wat::service::Outcome::Continue s
             (:wat::core::Some (:wat::query::Store::Reply::CountIndex r)) sends none))
         (_ (:wat::kernel::assertion-failed! "fs: inner count-index failed" :wat::core::None :wat::core::None)))))])

;; ── helpers ──────────────────────────────────────────────────────────────────
(:wat::core::defn :e3::bodies [] -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String])  i <- :wat::core::i64]
      -> (:wat::core::Vector :- [:wat::core::String])
      (:wat::core::conj acc (:wat::core::format "m{i}" :i i)))
    (:wat::core::Vector :- [:wat::core::String])
    (:wat::core::range 0 10)))

(:wat::core::defn :e3::rows
  [q <- :wat::core::String  now-ns <- :wat::core::i64]
  -> (:wat::core::Vector :- [:wat::query::StoredRow])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::query::StoredRow])  i <- :wat::core::i64]
      -> (:wat::core::Vector :- [:wat::query::StoredRow])
      (:wat::core::let
        [body (:wat::core::nth (:e3::bodies) i)
         sk   (:wat::core::format "sk{i}" :i i)
         isk  (:wat::edn::write (:wat::time::at-nanos (:wat::i64::+ now-ns i)))]
        (:wat::core::conj acc
          (:wat::query::StoredRow
            :pk q :sk sk :data body
            :index-keys (:wat::core::HashMap :- [:wat::core::String :wat::query::IndexKey]
                          "by-visible-at" (:wat::query::IndexKey :ipk q :isk isk))))))
    (:wat::core::Vector :- [:wat::query::StoredRow])
    (:wat::core::range 0 10)))

(:wat::core::defn :e3::ensure!
  [st <- :wat::query::Store] -> :wat::core::nil
  (:wat::core::match
    (:wat::query::Store/ensure-schema st
      (:wat::query::Store::EnsureSchemaRequest
        :table   (:wat::query::TableSchema :pk "pk" :sk "sk")
        :indexes (:wat::core::Vector :- [:wat::query::IndexSchema]
                   (:wat::query::IndexSchema
                     :name "by-visible-at" :pk "pk" :sk "sk" :ipk "ipk" :isk "isk"))))
    ((:wat::kernel::RecvOutcome::Message _r) nil)
    (_ (:wat::kernel::assertion-failed! "e3: ensure-schema failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :e3::put-tag
  [r <- :wat::query::Store::PutResponse] -> :wat::core::String
  (:wat::core::match r
    ((:wat::query::Store::PutResponse::Success) "Success")
    ((:wat::query::Store::PutResponse::Transient _e) "Transient")
    ((:wat::query::Store::PutResponse::Constraint _e) "Constraint")
    ((:wat::query::Store::PutResponse::Fatal _e) "Fatal")
    ((:wat::query::Store::PutResponse::RequestTooLarge _b _c) "RequestTooLarge")
    ((:wat::query::Store::PutResponse::RequestMalformed _p _e _g) "RequestMalformed")))

(:wat::core::defn :e3::del-tag
  [r <- :wat::query::Store::DeleteResponse] -> :wat::core::String
  (:wat::core::match r
    ((:wat::query::Store::DeleteResponse::Success) "Success")
    ((:wat::query::Store::DeleteResponse::Transient _e) "Transient")
    ((:wat::query::Store::DeleteResponse::Constraint _e) "Constraint")
    ((:wat::query::Store::DeleteResponse::Fatal _e) "Fatal")
    ((:wat::query::Store::DeleteResponse::RequestTooLarge _b _c) "RequestTooLarge")
    ((:wat::query::Store::DeleteResponse::RequestMalformed _p _e _g) "RequestMalformed")))

(:wat::core::defn :e3::scan-td
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
        (_ (:wat::kernel::assertion-failed! "e3: scan-index not Success" :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "e3: scan-index recv failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :e3::dial-store
  [a <- (:wat::kernel::Address :- [:wat::query::Store::Op :wat::query::Store::Reply])]
  -> :wat::query::Store
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected c) c)
    (_ (:wat::kernel::assertion-failed! "e3: store dial failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :e3::dial-q
  [a <- (:wat::kernel::Address :- [:queue::Queue::Op :queue::Queue::Reply])]
  -> :queue::Queue
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected c) c)
    (_ (:wat::kernel::assertion-failed! "e3: queue dial failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :e3::send-tag
  [q <- :queue::Queue  name <- :wat::core::String  now-ns <- :wat::core::i64]
  -> :wat::core::String
  (:wat::core::match
    (:queue::Queue/send q
      (:queue::Queue::SendRequest :queue name :bodies (:e3::bodies) :now-ns now-ns))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:queue::Queue::SendResponse::Ok) "Ok")
        ((:queue::Queue::SendResponse::Full _d _c) "Full")
        ((:queue::Queue::SendResponse::RequestTooLarge _b _c) "RequestTooLarge")
        ((:queue::Queue::SendResponse::RequestTooManyEntries _e _c) "RequestTooManyEntries")
        ((:queue::Queue::SendResponse::RequestMalformed _p _e _g) "RequestMalformed")))
    ((:wat::kernel::RecvOutcome::Lost _c) "Lost")
    (:wat::kernel::RecvOutcome::Closed "Closed")
    (:wat::kernel::RecvOutcome::Stopped "Stopped")
    (:wat::kernel::RecvOutcome::TimedOut "TimedOut")))

(:wat::core::defn :e3::ack-tag
  [q <- :queue::Queue  name <- :wat::core::String  ids <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::String
  (:wat::core::match
    (:queue::Queue/ack q (:queue::Queue::AckRequest :queue name :ids ids))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:queue::Queue::AckResponse::Ok) "Ok")
        ((:queue::Queue::AckResponse::RequestTooLarge _b _c) "RequestTooLarge")
        ((:queue::Queue::AckResponse::RequestMalformed _p _e _g) "RequestMalformed")))
    ((:wat::kernel::RecvOutcome::Lost _c) "Lost")
    (:wat::kernel::RecvOutcome::Closed "Closed")
    (:wat::kernel::RecvOutcome::Stopped "Stopped")
    (:wat::kernel::RecvOutcome::TimedOut "TimedOut")))

(:wat::core::defn :e3::recv-ids
  [q <- :queue::Queue  name <- :wat::core::String  now-ns <- :wat::core::i64  vis-ns <- :wat::core::i64]
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::match
    (:queue::Queue/receive q
      (:queue::Queue::ReceiveRequest
        :queue name :now-ns now-ns :visibility-ns vis-ns :limit 10
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
        (_ (:wat::kernel::assertion-failed! "e3: receive not Ok" :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "e3: receive recv failed" :wat::core::None :wat::core::None))))

;; ── cells ────────────────────────────────────────────────────────────────────
(:wat::core::defn :e3::cell-put [] -> :wat::core::String
  (:wat::core::let
    [T0 1000000000
     msh (:wat::query::mem-store/start :locus (:wat::spawn::thread)
            :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     fsh (:fs::failing-store/start :locus (:wat::spawn::thread)
            :record (:fs::failing-store::Record
                      :inner-addr (:wat::query::mem-store::Handle/addr msh)
                      :put-fail-bp 10000 :delete-fail-bp 0 :drop-seed 1))
     fs  (:e3::dial-store (:fs::failing-store::Handle/addr fsh))
     _es (:e3::ensure! fs)
     put-r (:wat::core::match
             (:wat::query::Store/put fs (:wat::query::Store::PutRequest (:e3::rows "w" T0)))
             ((:wat::kernel::RecvOutcome::Message r) (:e3::put-tag r))
             ((:wat::kernel::RecvOutcome::Lost _c) "Lost")
             (:wat::kernel::RecvOutcome::Closed "Closed")
             (:wat::kernel::RecvOutcome::Stopped "Stopped")
             (:wat::kernel::RecvOutcome::TimedOut "TimedOut"))
     wit (:e3::scan-td fs "w" (:wat::i64::+ T0 100))
     msh2 (:wat::query::mem-store/start :locus (:wat::spawn::thread)
             :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     fsh2 (:fs::failing-store/start :locus (:wat::spawn::thread)
             :record (:fs::failing-store::Record
                       :inner-addr (:wat::query::mem-store::Handle/addr msh2)
                       :put-fail-bp 10000 :delete-fail-bp 0 :drop-seed 1))
     qh  (:queue::queue/start :locus (:wat::spawn::thread)
            :record (:queue::queue::Record
                      :cap 1024 :store-addr (:fs::failing-store::Handle/addr fsh2)
                      :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     q   (:e3::dial-q (:queue::queue::Handle/addr qh))
     send-r (:e3::send-tag q "q" T0)
     ;; Drain on the inner mem-store, not the wrapper: receive re-puts, and a
     ;; still-armed put-fail would partial-fail the visibility hide.
     ;; now-ns is T0+20 so the 1ns-staggered isk values T0..T0+9 are all visible
     ;; (a receive at T0 would see only the first row).
     qh2 (:queue::queue/start :locus (:wat::spawn::thread)
            :record (:queue::queue::Record
                      :cap 1024 :store-addr (:wat::query::mem-store::Handle/addr msh2)
                      :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     q2  (:e3::dial-q (:queue::queue::Handle/addr qh2))
     mem (:e3::dial-store (:wat::query::mem-store::Handle/addr msh2))
     td  (:e3::scan-td mem "q" (:wat::i64::+ T0 100))
     got (:e3::recv-ids q2 "q" (:wat::i64::+ T0 20) 100)]
    (:wat::core::format
      "store={s};landed={k}/{n};send={snd};drain-n={dn};total={t};distinct={d}"
      :s put-r
      :k (:wat::core::first wit) :n 10
      :snd send-r
      :dn (:wat::core::count got)
      :t (:wat::core::first td) :d (:wat::core::second td))))

(:wat::core::defn :e3::cell-del [] -> :wat::core::String
  (:wat::core::let
    [T0 1000000000
     vis 100
     ;; Witness: put 10 cleanly, delete 10 with fail on.
     msh (:wat::query::mem-store/start :locus (:wat::spawn::thread)
            :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     fsh (:fs::failing-store/start :locus (:wat::spawn::thread)
            :record (:fs::failing-store::Record
                      :inner-addr (:wat::query::mem-store::Handle/addr msh)
                      :put-fail-bp 0 :delete-fail-bp 10000 :drop-seed 1))
     fs  (:e3::dial-store (:fs::failing-store::Handle/addr fsh))
     _es (:e3::ensure! fs)
     _p  (:wat::core::match
           (:wat::query::Store/put fs (:wat::query::Store::PutRequest (:e3::rows "w" T0)))
           ((:wat::kernel::RecvOutcome::Message _r) nil)
           (_ (:wat::kernel::assertion-failed! "e3: witness put failed" :wat::core::None :wat::core::None)))
     keys (:wat::core::foldl
            (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::query::Key])  i <- :wat::core::i64]
              -> (:wat::core::Vector :- [:wat::query::Key])
              (:wat::core::conj acc (:wat::query::Key :pk "w" :sk (:wat::core::format "sk{i}" :i i))))
            (:wat::core::Vector :- [:wat::query::Key])
            (:wat::core::range 0 10))
     del-r (:wat::core::match
             (:wat::query::Store/delete fs (:wat::query::Store::DeleteRequest keys))
             ((:wat::kernel::RecvOutcome::Message r) (:e3::del-tag r))
             ((:wat::kernel::RecvOutcome::Lost _c) "Lost")
             (:wat::kernel::RecvOutcome::Closed "Closed")
             (:wat::kernel::RecvOutcome::Stopped "Stopped")
             (:wat::kernel::RecvOutcome::TimedOut "TimedOut"))
     wit (:e3::scan-td fs "w" (:wat::i64::+ T0 100))
     ;; Queue cell: send+receive clean, ack with delete-fail on.
     msh2 (:wat::query::mem-store/start :locus (:wat::spawn::thread)
             :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     fsh2 (:fs::failing-store/start :locus (:wat::spawn::thread)
             :record (:fs::failing-store::Record
                       :inner-addr (:wat::query::mem-store::Handle/addr msh2)
                       :put-fail-bp 0 :delete-fail-bp 10000 :drop-seed 1))
     qh  (:queue::queue/start :locus (:wat::spawn::thread)
            :record (:queue::queue::Record
                      :cap 1024 :store-addr (:fs::failing-store::Handle/addr fsh2)
                      :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     q   (:e3::dial-q (:queue::queue::Handle/addr qh))
     _s  (:e3::send-tag q "q" T0)
     ids (:e3::recv-ids q "q" (:wat::i64::+ T0 20) vis)
     ack-r (:e3::ack-tag q "q" ids)
     qh2 (:queue::queue/start :locus (:wat::spawn::thread)
            :record (:queue::queue::Record
                      :cap 1024 :store-addr (:wat::query::mem-store::Handle/addr msh2)
                      :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     q2  (:e3::dial-q (:queue::queue::Handle/addr qh2))
     ;; Past the visibility window: the undeleted row must redeliver.
     red (:e3::recv-ids q2 "q" (:wat::i64::+ (:wat::i64::+ T0 20) (:wat::i64::+ vis 1)) vis)
     mem (:e3::dial-store (:wat::query::mem-store::Handle/addr msh2))
     td  (:e3::scan-td mem "q" (:wat::i64::+ T0 1000000))]
    (:wat::core::format
      "store={s};remaining={k}/{n};got={g};ack={a};redelivered={r};total={t};distinct={d}"
      :s del-r
      :k (:wat::core::first wit) :n 10
      :g (:wat::core::count ids)
      :a ack-r
      :r (:wat::core::count red)
      :t (:wat::core::first td) :d (:wat::core::second td))))

(:wat::core::defn :e3::run [] -> :wat::core::String
  (:wat::core::format "PUT={p};DEL={d}"
    :p (:e3::cell-put)
    :d (:e3::cell-del)))

(:wat::core::defn :user::compute [] -> :wat::core::String (:e3::run))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println (:e3::run)))
