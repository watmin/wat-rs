;; Arc 278 Stone 1a — the per-service hard frame limit `FOO` (:max-frame-bytes) + honest
;; over-FOO rejection. A defservice DECLARES its `FOO` (bytes-per-read); it threads to the
;; accepted-connection receivers. A frame over `FOO` → reject + CLOSE that connection with a
;; REASON (ServiceEvent::Lost, never the mute reason-free Closed), service keeps serving.
;;
;; Over-FOO is a PROCESS-tier concept (byte frames); thread tier has no frames.

(:wat::core::defsurface :probe::Big :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :probe::Big::PutRequest  [payload <- :wat::core::String])
   (:wat::core::defenum :probe::Big::PutResponse :wat::enum::Pure
     :Ok              [ok <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64])]
  :features
  [(put [self <- :probe::Big  req <- :probe::Big::PutRequest] -> :probe::Big::PutResponse :max-request-bytes 1048576)])

;; (a) a BULK service: declares a large FOO (1 MiB) so a ~600 KiB request ARRIVES.
(:wat::service::defservice :probe::bigfoo'
  :satisfies :probe::Big
  :max-frame-bytes 1048576
  :durable   []
  :ephemeral []
  :impls
  [(put [s req] (:wat::service::Outcome::Reply s (:probe::Big::PutResponse::Ok 7)))])

;; (b) a SMALL-FOO service: declares FOO = 4096, so a > 4 KiB request is rejected + closed.
(:wat::service::defservice :probe::smallfoo'
  :satisfies :probe::Big
  :max-frame-bytes 4096
  :durable   []
  :ephemeral []
  :impls
  [(put [s req] (:wat::service::Outcome::Reply s (:probe::Big::PutResponse::Ok 7)))])

;; Build a String of exactly n*32 bytes.
(:wat::core::defn :probe::payload-of [n <- :wat::core::i64] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String  _i <- :wat::core::i64] -> :wat::core::String
      (:wat::core::string::concat acc "0123456789ABCDEF0123456789ABCDEF"))
    ""
    (:wat::core::range 0 n)))

;; ── (a) large FOO ACCEPTS ~600 KiB: succeeds (returns 7). At HEAD (512 KiB default) it MUTES.
(:wat::core::defn :user::large-foo-accepts [] -> :wat::core::i64
  (:wat::core::let
    [big  (:probe::payload-of 19200)   ;; 19200*32 = 614400 bytes ≈ 600 KiB (> 512 KiB, < 1 MiB)
     h    (:probe::bigfoo'/start :locus (:wat::spawn::process) :record (:probe::bigfoo'::Record))
     c    (:wat::kernel::connect' (:probe::bigfoo'::Handle/addr h))
     r    (:probe::Big/put c (:probe::Big::PutRequest :payload big))]
    (:wat::core::match r -> :wat::core::i64
      ((:probe::Big::PutResponse::Ok ok) ok)
      ;; terminal caller: an unexpected wire-breach must SURFACE, never swallow.
      ((:probe::Big::PutResponse::RequestTooLarge bytes cap)
        (:wat::kernel::assertion-failed! "large-foo-accepts: unexpected RequestTooLarge"
          :wat::core::None :wat::core::None)))))

;; ── (b) small FOO REJECTS a > 4 KiB request: the caller's op must FAIL WITH A REASON (not the
;; mute "peer closed"). The .rs harness captures the raise.
(:wat::core::defn :user::small-foo-rejects [] -> :wat::core::i64
  (:wat::core::let
    [big  (:probe::payload-of 400)     ;; 400*32 = 12800 bytes > 4096
     h    (:probe::smallfoo'/start :locus (:wat::spawn::process) :record (:probe::smallfoo'::Record))
     c    (:wat::kernel::connect' (:probe::smallfoo'::Handle/addr h))
     r    (:probe::Big/put c (:probe::Big::PutRequest :payload big))]
    (:wat::core::match r -> :wat::core::i64
      ((:probe::Big::PutResponse::Ok ok) ok)
      ;; terminal caller: an unexpected wire-breach must SURFACE, never swallow.
      ((:probe::Big::PutResponse::RequestTooLarge bytes cap)
        (:wat::kernel::assertion-failed! "small-foo-rejects: unexpected RequestTooLarge"
          :wat::core::None :wat::core::None)))))

;; ── (b') SURVIVAL probe: c1 fires an over-FOO frame (send' only, fire-and-forget — no recv, so
;; this fn does not raise on c1); then a FRESH connection c2 issues an in-budget request. If the
;; service SURVIVED the over-FOO, c2's put returns 7. If the over-FOO crashed the whole service
;; child, c2's put fails. (Determinism note: c1's frame is fired before c2's request is issued.)
(:wat::core::defn :user::small-foo-survives [] -> :wat::core::i64
  (:wat::core::let
    [big  (:probe::payload-of 400)     ;; > 4096 → over-FOO
     h    (:probe::smallfoo'/start :locus (:wat::spawn::process) :record (:probe::smallfoo'::Record))
     addr (:probe::smallfoo'::Handle/addr h)
     c1   (:wat::kernel::connect' addr)
     c2   (:wat::kernel::connect' addr)
     _    (:wat::kernel::send' c1 (:probe::Big::Op::Put (:probe::Big::PutRequest :payload big)))
     r    (:probe::Big/put c2 (:probe::Big::PutRequest :payload "small"))]
    (:wat::core::match r -> :wat::core::i64
      ((:probe::Big::PutResponse::Ok ok) ok)
      ;; terminal caller: an unexpected wire-breach must SURFACE, never swallow.
      ((:probe::Big::PutResponse::RequestTooLarge bytes cap)
        (:wat::kernel::assertion-failed! "small-foo-survives: unexpected RequestTooLarge"
          :wat::core::None :wat::core::None)))))
