;; probe-handler-issues-request-and-returns.wat — the FEASIBILITY probe for
;; docs/excursus/2026/08/001-sns-sqs/a-handler-must-not-block.
;;
;; THE QUESTION: can a `defservice` handler issue a request to a declared peer and RETURN,
;; with the reply arriving as one of the service's OWN ops?
;;
;; THE ANSWER THIS PROBE ESTABLISHES: **HALF.**
;;
;;   OUTBOUND — YES.  `(:wat::kernel::send st op)` inside a handler is non-blocking and the
;;                    handler returns immediately (`poke`, below). GREEN here proves it.
;;
;;   INBOUND  — NO.   The reply does NOT arrive as an op. It is STRANDED in the client peer's
;;                    own channel, reachable only by a LATER handler blocking on `recv`
;;                    (`-collect`, below) — which is the same defect one tick later.
;;
;; WHY the inbound half does not exist for an author — MEASURED IN SITU, not read.
;;
;; The serve loop's select set is `selectables` (`selectable-entry-vec-ty`, wat/service.wat:1594),
;; element `(i64, (Peer :- [<proto>::Reply <service>::Op]))` (`selectable-peer-ty`,
;; wat/service.wat:1566). `Peer`'s arg order is <S,R> — SEND-type first (src/types.rs:2279; a
;; timer is the uninhabited-send `(Peer :- [Never O])`, src/types.rs:6377). So every member of
;; that set is a peer this service SENDS Replies on and RECEIVES its own Ops from: an accepted
;; inbound client, or an alarm timer. A client peer dialed AT a peer service is
;; `(Peer :- [<their>::Op <their>::Reply])` — WRONG IN BOTH POSITIONS.
;;
;; ⚠ AND `selectables` IS LEXICALLY IN SCOPE INSIDE A HANDLER BODY. That is not a designed
;; affordance, it is an unhygienic macro capture: op bodies are spliced INLINE into the serve
;; arm inside a `let` (wat/service.wat:1834 builds the binding vector; :1813 says outright
;; "`selectables`/`idx` are bare — literal identifiers in the GENERATED code"). A handler can
;; READ it — `(:wat::core::length selectables)` type-checks and runs, demonstrated GREEN below.
;;
;; It still cannot be USED, and the checker says so against the REAL captured `selectables` of
;; a REAL generated serve loop. The attempt was run; VERBATIM:
;;
;;   #wat.check/TypeMismatch {:message ":wat::core::conj: parameter #2 expects
;;     :(wat::core::i64,(wat::kernel::Peer :- [probe::Asker::Reply probe::asker::Op]));
;;     got :(wat::core::i64,(wat::kernel::Peer :- [probe::Store::Op probe::Store::Reply]))"
;;     :callee ":wat::core::conj" :param "#2"}
;;
;; And a READ is all it is: the loop recurses on ITS OWN `selectables`
;; (`(~serve-name self l (:wat::core::foldl ~arm-fn selectables arms) …)`, wat/service.wat:2074),
;; so a handler's rebuilt vector is discarded — the handler's only return channel is an
;; `Outcome`. `:wat::kernel::select`/`poll` also take a HOMOGENEOUS `(Vector :- [(Peer :- [I O])])`
;; (`infer_select_prime`, src/check.rs:12305), so there is no heterogeneous set to join even in
;; principle.
;;
;; The set is grown at exactly TWO sites, both internal: `ServiceEvent::Connection` (an ACCEPTED
;; inbound client, wat/service.wat:2308) and `arm-fn` folding `Alarm`s into `after` timers
;; (wat/service.wat:1885). There is no `:selectables` clause (recognized clauses,
;; wat/service.wat:410) and no outcome field naming a peer (`Alarm` wat/service.wat:67,
;; `Directed` :71-73, `Outcome` :80-89, `SelfOutcome` :90-95). `:peers` (wat/service.wat:811-829) is a DECLARATION/manifest clause —
;; a dependency DAG plus a bijection check against ephemeral peer fields — it wires nothing into
;; the select set.
;;
;; GREEN = the split above is exactly as described: the send returns, the reply is stranded, and
;; only an explicit blocking `recv` in a later handler retrieves it.

(:wat::core::defsurface :probe::Store :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Store::BumpRequest [by <- :wat::core::i64])
   (:wat::core::defenum :probe::Store::BumpResponse :wat::enum::Pure
     :Ok               [n <- :wat::core::i64]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(bump [self <- :probe::Store  req <- :probe::Store::BumpRequest] -> :probe::Store::BumpResponse :max-request-bytes 524288)])

(:wat::service::defservice :probe::store
  :satisfies :probe::Store
  :durable   [n <- :wat::core::i64]
  :ephemeral []
  :init (:wat::core::fn [record <- :probe::store::Record] -> :probe::store::State
          (:probe::store::State :durable record))
  :impls
  [(bump [s ctx req]
     (:wat::core::let
       [n' (:wat::i64::+ (:probe::store::Record/n (:probe::store::State/durable s))
                         (:probe::Store::BumpRequest/by req))]
       (:wat::service::Outcome::Continue
         (:probe::store::State :durable (:probe::store::Record :n n'))
         (:wat::core::Some (:probe::Store::Reply::Bump (:probe::Store::BumpResponse::Ok n')))
         (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::Store::Reply])])
         (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::store::Op])]))))])

(:wat::core::defsurface :probe::Asker :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Asker::PokeRequest [])
   (:wat::core::defrecord :probe::Asker::ReadRequest [])
   (:wat::core::defenum :probe::Asker::PokeResponse :wat::enum::Pure
     :Ok               [ok <- :wat::core::bool]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])
   (:wat::core::defenum :probe::Asker::ReadResponse :wat::enum::Pure
     :Ok               [n <- :wat::core::i64]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(poke [self <- :probe::Asker  req <- :probe::Asker::PokeRequest] -> :probe::Asker::PokeResponse :max-request-bytes 524288)
   (read [self <- :probe::Asker  req <- :probe::Asker::ReadRequest] -> :probe::Asker::ReadResponse :max-request-bytes 524288)])

(:wat::service::defservice :probe::asker
  :satisfies :probe::Asker
  :durable   [store-addr <- (:wat::kernel::Address :- [:probe::Store::Op :probe::Store::Reply])
              last <- :wat::core::i64]
  :ephemeral [st <- (:wat::kernel::Peer :- [:probe::Store::Op :probe::Store::Reply])]
  :peers     [:probe::Store]
  :init (:wat::core::fn [record <- :probe::asker::Record] -> :probe::asker::State
          (:probe::asker::State :durable record
            :st (:wat::core::match (:wat::kernel::connect (:probe::asker::Record/store-addr record))
                  ((:wat::kernel::ConnectOutcome::Connected p) p)
                  ((:wat::kernel::ConnectOutcome::Refused c)
                    (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
                  ((:wat::kernel::ConnectOutcome::Rejected c)
                    (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
                  ((:wat::kernel::ConnectOutcome::Failed c)
                    (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))))
  :impls
  ;; ★ THE OUTBOUND HALF — GREEN. `send` is issued and the handler RETURNS. No select, no
  ;; recv, no deadline timer: nothing here parks. The service is answerable the whole time.
  [(poke [s ctx req]
     (:wat::core::let
       [;; ⚠ THE UNHYGIENIC CAPTURE, demonstrated GREEN: `selectables` — the generated serve
        ;; loop's own select set — is in scope in an author's handler body. Readable; useless.
        ;; If a future hygiene fix reddens this line, that is CORRECT: the finding expired.
        nsel (:wat::core::length selectables)
        sr (:wat::core::match (:wat::kernel::send (:probe::asker::State/st s)
                                 (:probe::Store::Op::Bump (:probe::Store::BumpRequest :by 7)))
              (:wat::kernel::SendOutcome::Sent 1)
              (:wat::kernel::SendOutcome::Closed 2)
              (:wat::kernel::SendOutcome::Stopped 3)
              ((:wat::kernel::SendOutcome::Lost _c) 4))
        rec0 (:probe::asker::State/durable s)
        s0 (:probe::asker::State :durable
             (:probe::asker::Record :store-addr (:probe::asker::Record/store-addr rec0) :last (:wat::i64::+ (:wat::i64::* sr 1000) (:wat::i64::* nsel 100)))
             :st (:probe::asker::State/st s))]
       (:wat::core::match 1
         (_ (:wat::service::Outcome::Continue s0
            (:wat::core::Some (:probe::Asker::Reply::Poke (:probe::Asker::PokeResponse::Ok true)))
            (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::Asker::Reply])])
            ;; the ONLY thing a handler may schedule: a self-message. NOT the peer's reply.
              [(:wat::service::Alarm :delay (:wat::time::Milliseconds 20) :op :-collect)])))))
   ;; ⛔ THE INBOUND HALF — MISSING. The reply did not arrive as an op, so this arm must go
   ;; and FETCH it, and the only fetch is a BLOCKING `recv` on the client peer. That is the
   ;; defect displaced by one tick, not removed.
   (-collect [s ctx]
     (:wat::core::let
       [rec (:probe::asker::State/durable s)
        got (:wat::core::match (:wat::kernel::recv (:probe::asker::State/st s))
              ((:wat::kernel::RecvOutcome::Message m)
                (:wat::core::match m
                  ((:probe::Store::Reply::Bump r)
                    (:wat::core::match r
                      ((:probe::Store::BumpResponse::Ok n) n)
                      ((:probe::Store::BumpResponse::RequestTooLarge _b _c) -1)
                      ((:probe::Store::BumpResponse::RequestMalformed _p _e _g) -2)))
                  (_ -7)))
              ((:wat::kernel::RecvOutcome::Lost _c) -3)
              (:wat::kernel::RecvOutcome::Closed -4)
              (:wat::kernel::RecvOutcome::Stopped -5)
              (:wat::kernel::RecvOutcome::TimedOut -6) ((:wat::kernel::RecvOutcome::Malformed _cause) (:wat::kernel::assertion-failed! "recv: malformed frame — the peer could not decode our message; this arm is an UNMIGRATED PLACEHOLDER (a-momentary-failure-is-not-fatal, stone 2 replaces it with report-final)" :wat::core::None :wat::core::None)))]
       (:wat::service::SelfOutcome::Continue
         (:probe::asker::State :durable
           (:probe::asker::Record :store-addr (:probe::asker::Record/store-addr rec)
             :last (:wat::i64::+ (:probe::asker::Record/last rec) got))
           :st (:probe::asker::State/st s))
         (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::Asker::Reply])])
         (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::asker::Op])]))))
   (read [s ctx req]
     (:wat::service::Outcome::Continue s
       (:wat::core::Some (:probe::Asker::Reply::Read
                           (:probe::Asker::ReadResponse::Ok
                             (:probe::asker::Record/last (:probe::asker::State/durable s)))))
       (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::Asker::Reply])])
       (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::asker::Op])])))])

(:wat::core::defn :probe::dial :- [I O]
  [a <- (:wat::kernel::Address :- [:I :O])] -> (:wat::kernel::Peer :- [:I :O])
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    ((:wat::kernel::ConnectOutcome::Refused c)
      (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected c)
      (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed c)
      (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [sh (:probe::store/start :locus (:wat::spawn::thread) :record (:probe::store::Record :n 0))
     ah (:probe::asker/start :locus (:wat::spawn::thread)
          :record (:probe::asker::Record :store-addr (:probe::store::Handle/addr sh) :last 0))
     ac (:probe::dial (:probe::asker::Handle/addr ah))
     _p (:probe::Asker/poke ac (:probe::Asker::PokeRequest))
     ;; no sleep verb in the stdlib — wait honestly, on a timer peer (the `mora` ward):
     _s (:wat::kernel::recv
          (:wat::kernel::after :wat::program::PeerKind::thread (:wat::time::Milliseconds 300) 0))
     r  (:probe::Asker/read ac (:probe::Asker::ReadRequest))
     n  (:wat::core::match r
          ((:wat::kernel::RecvOutcome::Message m)
            (:wat::core::match m
              ((:probe::Asker::ReadResponse::Ok n) n)
              ((:probe::Asker::ReadResponse::RequestTooLarge _b _c) -1)
              ((:probe::Asker::ReadResponse::RequestMalformed _p _e _g) -2)))
          ((:wat::kernel::RecvOutcome::Lost _c) -3)
          (:wat::kernel::RecvOutcome::Closed -4)
          (:wat::kernel::RecvOutcome::Stopped -5)
          (:wat::kernel::RecvOutcome::TimedOut -6) ((:wat::kernel::RecvOutcome::Malformed _cause) (:wat::kernel::assertion-failed! "recv: malformed frame — the peer could not decode our message; this arm is an UNMIGRATED PLACEHOLDER (a-momentary-failure-is-not-fatal, stone 2 replaces it with report-final)" :wat::core::None :wat::core::None)))]
    (:wat::kernel::println
      ;; Observed, stably: code=1107 = SendOutcome::Sent(1)*1000 + selectables-length(1)*100 +
      ;; the reply(7) fetched by a BLOCKING recv in a LATER handler. The 7 never arrived as an
      ;; op — it was gone and got. selectables-length 1 = the one accepted client connection.
      (:wat::core::format "handler-send=Sent selectables-readable=yes reply-arrived-as-op=NO code={n}" :n n))))
