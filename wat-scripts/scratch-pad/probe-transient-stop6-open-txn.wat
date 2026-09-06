;; probe-transient-stop6-open-txn.wat — STOP-6 of transient-means-try-again.
;;
;; After a failed put, sqlite-store leaves the transaction OPEN (no rollback verb
;; exists). Does a subsequent put on the same connection succeed?
;; If not, the rollback is a prerequisite and the retry stone stops.

(:wat::config::set-redef! true)

(:wat::core::defn :s6::err-tag
  [e <- :wat::sqlite::Error] -> :wat::core::String
  (:wat::core::match e
    ((:wat::sqlite::Error::Transient f)
      (:wat::core::format "Transient:{c}:{m}"
        :c (:wat::sqlite::Fault/code f) :m (:wat::sqlite::Fault/message f)))
    ((:wat::sqlite::Error::Constraint f)
      (:wat::core::format "Constraint:{c}:{m}"
        :c (:wat::sqlite::Fault/code f) :m (:wat::sqlite::Fault/message f)))
    ((:wat::sqlite::Error::Fatal f)
      (:wat::core::format "Fatal:{c}:{m}"
        :c (:wat::sqlite::Fault/code f) :m (:wat::sqlite::Fault/message f)))))

(:wat::core::defn :s6::res-tag
  [r <- (:wat::core::Result :- [:wat::core::nil :wat::sqlite::Error])] -> :wat::core::String
  (:wat::core::match r
    ((:wat::core::Ok _) "Ok")
    ((:wat::core::Err e) (:s6::err-tag e))))

(:wat::core::defn :s6::put-tag
  [r <- :wat::query::Store::PutResponse] -> :wat::core::String
  (:wat::core::match r
    ((:wat::query::Store::PutResponse::Success) "Success")
    ((:wat::query::Store::PutResponse::Transient e)
      (:wat::core::format "Transient:{m}" :m (:wat::query::Fault/message (:wat::query::Transient/reason e))))
    ((:wat::query::Store::PutResponse::Constraint e)
      (:wat::core::format "Constraint:{m}" :m (:wat::query::Fault/message (:wat::query::Constraint/reason e))))
    ((:wat::query::Store::PutResponse::Fatal e)
      (:wat::core::format "Fatal:{m}" :m (:wat::query::Fault/message (:wat::query::Fatal/reason e))))
    ((:wat::query::Store::PutResponse::RequestTooLarge _b _c) "RequestTooLarge")
    ((:wat::query::Store::PutResponse::RequestMalformed _p _e _g) "RequestMalformed")))

(:wat::core::defn :s6::row [] -> :wat::query::StoredRow
  (:wat::query::StoredRow
    :pk "q" :sk "sk0" :data "m0"
    :index-keys (:wat::core::HashMap :- [:wat::core::String :wat::query::IndexKey]
                  "by-visible-at" (:wat::query::IndexKey :ipk "q" :isk "0"))))

;; Cell A — raw connection: begin, fail inside, begin again.
(:wat::core::defn :s6::cell-conn [] -> :wat::core::String
  (:wat::core::let
    [conn (:wat::core::Result/expect (:wat::sqlite::open ":memory:") "s6: open")
     none (:wat::core::Vector :- [:wat::sqlite::Param])
     b1   (:s6::res-tag (:wat::sqlite::begin conn))
     e1   (:wat::core::match
            (:wat::sqlite::execute conn "INSERT INTO nosuch (x) VALUES (1)" none)
            ((:wat::core::Ok _) "Ok")
            ((:wat::core::Err e) (:s6::err-tag e)))
     b2   (:s6::res-tag (:wat::sqlite::begin conn))
     cmt  (:s6::res-tag (:wat::sqlite::commit conn))]
    (:wat::core::format "begin1={a};fail={b};begin2={c};commit={d}"
      :a b1 :b e1 :c b2 :d cmt)))

;; Cell B — Store put with no schema: begin+DELETE FROM main fails, txn open, put again.
(:wat::core::defn :s6::cell-store [] -> :wat::core::String
  (:wat::core::let
    [h (:wat::query::sqlite-store/start :locus (:wat::spawn::thread)
          :record (:wat::query::sqlite-store::Record
                    :path ":memory:"
                    :index-names (:wat::core::Vector :- [:wat::core::String] "by-visible-at")))
     st (:wat::core::match (:wat::kernel::connect (:wat::query::sqlite-store::Handle/addr h))
          ((:wat::kernel::ConnectOutcome::Connected c) c)
          (_ (:wat::kernel::assertion-failed! "s6: store dial failed" :wat::core::None :wat::core::None)))
     req (:wat::query::Store::PutRequest
           (:wat::core::Vector :- [:wat::query::StoredRow] (:s6::row)))
     p1 (:wat::core::match (:wat::query::Store/put st req)
          ((:wat::kernel::RecvOutcome::Message r) (:s6::put-tag r))
          ((:wat::kernel::RecvOutcome::Lost _c) "Lost")
          (:wat::kernel::RecvOutcome::Closed "Closed")
          (:wat::kernel::RecvOutcome::Stopped "Stopped")
          (:wat::kernel::RecvOutcome::TimedOut "TimedOut"))
     p2 (:wat::core::match (:wat::query::Store/put st req)
          ((:wat::kernel::RecvOutcome::Message r) (:s6::put-tag r))
          ((:wat::kernel::RecvOutcome::Lost _c) "Lost")
          (:wat::kernel::RecvOutcome::Closed "Closed")
          (:wat::kernel::RecvOutcome::Stopped "Stopped")
          (:wat::kernel::RecvOutcome::TimedOut "TimedOut"))]
    (:wat::core::format "put1={a};put2={b}" :a p1 :b p2)))

(:wat::core::defn :s6::run [] -> :wat::core::String
  (:wat::core::format "CONN={c};STORE={s}" :c (:s6::cell-conn) :s (:s6::cell-store)))

(:wat::core::defn :user::compute [] -> :wat::core::String (:s6::run))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println (:s6::run)))
