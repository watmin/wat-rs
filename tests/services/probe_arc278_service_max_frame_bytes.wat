;; Arc 278 Stone 1a — the per-service hard frame limit `FOO` (:max-frame-bytes) + honest
;; over-FOO rejection. A defservice DECLARES its `FOO` (bytes-per-read); it threads to the
;; accepted-connection receivers. A frame over `FOO` → reject + CLOSE that connection with a
;; REASON (ServiceEvent::Lost, never the mute reason-free Closed), service keeps serving.
;;
;; Over-FOO is a PROCESS-tier concept (byte frames); thread tier has no frames.

(:wat::core::defsurface :probe::Big :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Big::PutRequest  [payload <- :wat::core::String])
   (:wat::core::defenum :probe::Big::PutResponse :wat::enum::Pure
     :Ok              [ok <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(put [self <- :probe::Big  req <- :probe::Big::PutRequest] -> :probe::Big::PutResponse :max-request-bytes 1048576)])

;; (a) a BULK service: declares a large FOO (1 MiB) so a ~600 KiB request ARRIVES.
(:wat::service::defservice :probe::bigfoo
  :satisfies :probe::Big
  :max-frame-bytes 1048576
  :durable   []
  :ephemeral []
  :impls
  [(put [s ctx req] (:wat::service::Outcome::Reply {:state s :reply (:probe::Big::PutResponse::Ok {:ok 7})}))])

;; (b) a SMALL-FOO service: declares FOO = 4096, so a > 4 KiB request is rejected + closed.
(:wat::service::defservice :probe::smallfoo
  :satisfies :probe::Big
  :max-frame-bytes 4096
  :durable   []
  :ephemeral []
  :impls
  [(put [s ctx req] (:wat::service::Outcome::Reply {:state s :reply (:probe::Big::PutResponse::Ok {:ok 7})}))])

;; Build a String of exactly n*32 bytes.
(:wat::core::defn :probe::payload-of [n <- :wat::core::i64] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String  _i <- :wat::core::i64] -> :wat::core::String
      (:wat::string::concat acc "0123456789ABCDEF0123456789ABCDEF"))
    ""
    (:wat::core::range 0 n)))

;; ── (a) large FOO ACCEPTS ~600 KiB: succeeds (returns 7). At HEAD (512 KiB default) it MUTES.
(:wat::core::defn :user::large-foo-accepts [] -> :wat::core::i64
  (:wat::core::let
    [big  (:probe::payload-of 19200)   ;; 19200*32 = 614400 bytes ≈ 600 KiB (> 512 KiB, < 1 MiB)
     h    (:probe::bigfoo/start :locus (:wat::spawn::process) :record (:probe::bigfoo::Record))
     c    (:wat::core::match (:wat::kernel::connect (:probe::bigfoo::Handle/addr h)) [:wat::kernel::ConnectOutcome::Connected {:peer p} p] [:wat::kernel::ConnectOutcome::Refused {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))] [:wat::kernel::ConnectOutcome::Rejected {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))] [:wat::kernel::ConnectOutcome::Failed {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))])
     r    (:probe::Big/put c (:probe::Big::PutRequest :payload big))]
    (:wat::core::match r [:wat::kernel::RecvOutcome::Message {:msg __recv} (:wat::core::match __recv 
      [:probe::Big::PutResponse::Ok {:ok ok} ok]
      ;; terminal caller: an unexpected wire-breach must SURFACE, never swallow.
      [:probe::Big::PutResponse::RequestTooLarge {:bytes bytes :cap cap}
        (:wat::kernel::assertion-failed! :message "large-foo-accepts: unexpected RequestTooLarge")]
      [:probe::Big::PutResponse::RequestMalformed {:path mpath :expected mexpected :got mgot}
        (:wat::kernel::assertion-failed! :message "unexpected RequestMalformed")])] [:wat::kernel::RecvOutcome::Lost {:cause __cause} (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message __cause))] [:wat::kernel::RecvOutcome::Stopped {} (:wat::kernel::assertion-failed! :message "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open")] [:wat::kernel::RecvOutcome::Closed {} (:wat::kernel::assertion-failed! :message "recv': peer closed")])))

;; ── (b) small FOO REJECTS a > 4 KiB request: the caller's op must FAIL WITH A REASON (not the
;; mute "peer closed"). EXACT DATA: :user::small-foo-rejects returns a structured :probe::Outcome —
;; the RecvOutcome variant that matched + a deterministic `names-frame-cap?` bool computed IN-WAT (the
;; per-run-variable reason location never leaves wat; only its boolean RESULT crosses to the .rs
;; golden #probe/Outcome.Lost {:sentinel-present? true}). Matching ::Lost (not the mute ::Closed) + the frame-cap-named
;; reason IS the LAW proven — the over-FOO reason reaches the caller. Mirrors probe_arc278_recv_outcome_wall.
(:wat::core::defenum :probe::Outcome :wat::enum::Pure
  :Message []                                          ;; matched ::Message (.rs asserts NEVER)
  :Lost    [names-frame-cap? <- :wat::core::bool]       ;; matched ::Lost — true iff the reason names the frame cap (the LAW: reason carried)
  ;; arc 278 #73 — matched ::Stopped (.rs asserts NEVER: this probe never asks the substrate to
  ;; stop, only over-FOO rejects). A structural twin of ::Closed, not folded into it.
  :Stopped []
  :Closed  [])                                         ;; matched ::Closed (the mute we killed — .rs asserts NEVER)
;; arc 278 recv'-wall: a peer-read yields a MATCHABLE RecvOutcome — never a raise (a raise unwinds
;; past the reader = the mask the wall kills). The client-method (:probe::Big/put) SCRUBS the cause
;; into a reason-free 500; the over-FOO reject IS a 400-class reason that must reach the caller, so we
;; send raw + recv' and MATCH, checking IN-WAT that the Reply::Failed cause names the frame cap.
(:wat::core::defn :user::small-foo-rejects [] -> :probe::Outcome
  (:wat::core::let
    [big  (:probe::payload-of 400)     ;; 400*32 = 12800 bytes > 4096
     h    (:probe::smallfoo/start :locus (:wat::spawn::process) :record (:probe::smallfoo::Record))
     c    (:wat::core::match (:wat::kernel::connect (:probe::smallfoo::Handle/addr h)) [:wat::kernel::ConnectOutcome::Connected {:peer p} p] [:wat::kernel::ConnectOutcome::Refused {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))] [:wat::kernel::ConnectOutcome::Rejected {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))] [:wat::kernel::ConnectOutcome::Failed {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))])
     _s   (:wat::kernel::send c (:probe::Big::Op::Put {:req (:probe::Big::PutRequest :payload big)}))]
    (:wat::core::match (:wat::kernel::recv c)
      [:wat::kernel::RecvOutcome::Message {:msg _m} (:probe::Outcome::Message)]
      [:wat::kernel::RecvOutcome::Lost {:cause cause}
        (:probe::Outcome::Lost (:wat::string::contains? (:wat::kernel::LociDiedError/message cause) "max-frame-bytes"))]
      [:wat::kernel::RecvOutcome::Stopped {} (:probe::Outcome::Stopped)]
      [:wat::kernel::RecvOutcome::Closed {} (:probe::Outcome::Closed)])))

;; ── (b') SURVIVAL probe: c1 fires an over-FOO frame (send' only, fire-and-forget — no recv, so
;; this fn does not raise on c1); then a FRESH connection c2 issues an in-budget request. If the
;; service SURVIVED the over-FOO, c2's put returns 7. If the over-FOO crashed the whole service
;; child, c2's put fails. (Determinism note: c1's frame is fired before c2's request is issued.)
(:wat::core::defn :user::small-foo-survives [] -> :wat::core::i64
  (:wat::core::let
    [big  (:probe::payload-of 400)     ;; > 4096 → over-FOO
     h    (:probe::smallfoo/start :locus (:wat::spawn::process) :record (:probe::smallfoo::Record))
     addr (:probe::smallfoo::Handle/addr h)
     c1   (:wat::core::match (:wat::kernel::connect addr) [:wat::kernel::ConnectOutcome::Connected {:peer p} p] [:wat::kernel::ConnectOutcome::Refused {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))] [:wat::kernel::ConnectOutcome::Rejected {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))] [:wat::kernel::ConnectOutcome::Failed {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))])
     c2   (:wat::core::match (:wat::kernel::connect addr) [:wat::kernel::ConnectOutcome::Connected {:peer p} p] [:wat::kernel::ConnectOutcome::Refused {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))] [:wat::kernel::ConnectOutcome::Rejected {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))] [:wat::kernel::ConnectOutcome::Failed {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))])
     _    (:wat::core::match (:wat::kernel::send c1 (:probe::Big::Op::Put {:req (:probe::Big::PutRequest :payload big)})) [:wat::kernel::SendOutcome::Sent {} nil] [:wat::kernel::SendOutcome::Closed {} nil] [:wat::kernel::SendOutcome::Stopped {} nil] [:wat::kernel::SendOutcome::Lost {:cause _c} nil])
     r    (:probe::Big/put c2 (:probe::Big::PutRequest :payload "small"))]
    (:wat::core::match r [:wat::kernel::RecvOutcome::Message {:msg __recv} (:wat::core::match __recv 
      [:probe::Big::PutResponse::Ok {:ok ok} ok]
      ;; terminal caller: an unexpected wire-breach must SURFACE, never swallow.
      [:probe::Big::PutResponse::RequestTooLarge {:bytes bytes :cap cap}
        (:wat::kernel::assertion-failed! :message "small-foo-survives: unexpected RequestTooLarge")]
      [:probe::Big::PutResponse::RequestMalformed {:path mpath :expected mexpected :got mgot}
        (:wat::kernel::assertion-failed! :message "unexpected RequestMalformed")])] [:wat::kernel::RecvOutcome::Lost {:cause __cause} (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message __cause))] [:wat::kernel::RecvOutcome::Stopped {} (:wat::kernel::assertion-failed! :message "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open")] [:wat::kernel::RecvOutcome::Closed {} (:wat::kernel::assertion-failed! :message "recv': peer closed")])))
