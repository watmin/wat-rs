;; Arc 294 item 9a — REGRESSION: sequential macro registration during expansion.
;;
;; THE ROOT this pins (src/macros/expand.rs): `expand_all` is expand-then-hoist, and
;; `expand_form` used to hold the registry IMMUTABLY — so nothing could register
;; mid-expansion. A `defservice` emits ONE `do` carrying BOTH its minted `::Record`/
;; `::State` companion `defmacro`s AND the `serve` defn whose handlers CONSTRUCT those
;; types. The serve body was expanded BEFORE the companions registered, so the
;; construction stayed RAW and died at eval with
;;   `#wat.runtime/UnknownFunction: unknown function: :probe::echo::State`.
;; The fix makes a `do`/`let` body's children see their earlier siblings' `defmacro`s —
;; the guarantee the engine already documents at top level.
;;
;; A/B, differing in exactly one thing:
;;   ping -> handler constructs NOTHING minted (state returned unchanged). CONTROL: this
;;           passed even at the broken HEAD, so a red here means the fix broke the
;;           ordinary path.
;;   bump -> handler constructs the defservice's OWN minted `::State`/`::Record`. THE
;;           REGRESSION: red at the broken HEAD, green only with sequential registration.
;; Note the caller ALSO constructs `(:probe::echo::Record :count 0)` at /start — in the
;; CALLER's world, where the companion HAS registered. That identical construction always
;; worked; only the one inside the handler was raw. That asymmetry IS the ordering root.

(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :probe::Echo::PingRequest  [])
   (:wat::core::defenum :probe::Echo::PingResponse :wat::enum::Pure
     :Ok              [value <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64])
   (:wat::core::defrecord :probe::Echo::BumpRequest  [])
   (:wat::core::defenum :probe::Echo::BumpResponse :wat::enum::Pure
     :Ok              [value <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64])]
  :features
  [(ping [self <- :probe::Echo  req <- :probe::Echo::PingRequest] -> :probe::Echo::PingResponse :max-request-bytes 524288)
   (bump [self <- :probe::Echo  req <- :probe::Echo::BumpRequest] -> :probe::Echo::BumpResponse :max-request-bytes 524288)])

(:wat::service::defservice :probe::echo
  :satisfies :probe::Echo
  :durable   [count <- :wat::core::i64]
  :ephemeral []
  :impls
  [(ping [s req]
     ;; CONTROL — no minted construction; state returned unchanged.
     (:wat::service::Outcome::Reply s (:probe::Echo::PingResponse::Ok 1)))
   (bump [s req]
     ;; THE REGRESSION — constructs this defservice's OWN minted `::State`/`::Record`.
     (:wat::service::Outcome::Reply
       (:probe::echo::State :durable (:probe::echo::Record :count 7))
       (:probe::Echo::BumpResponse::Ok 7)))])

;; CONTROL: ping round-trips (and proves the caller-world construction at /start works).
(:wat::core::defn :user::compute-ping [] -> :wat::core::i64
  (:wat::core::let
    [h (:probe::echo/start :locus (:wat::spawn::thread) :record (:probe::echo::Record :count 0))
     c (:wat::kernel::connect' (:probe::echo::Handle/addr h))
     r (:probe::Echo/ping c (:probe::Echo::PingRequest))]
    (:wat::core::match r -> :wat::core::i64
      ((:probe::Echo::PingResponse::Ok value) value)
      ((:probe::Echo::PingResponse::RequestTooLarge bytes cap)
        (:wat::kernel::assertion-failed! "compute-ping: unexpected RequestTooLarge"
          :wat::core::None :wat::core::None)))))

;; THE REGRESSION: bump round-trips — the handler's own minted construction must expand.
(:wat::core::defn :user::compute-bump [] -> :wat::core::i64
  (:wat::core::let
    [h (:probe::echo/start :locus (:wat::spawn::thread) :record (:probe::echo::Record :count 0))
     c (:wat::kernel::connect' (:probe::echo::Handle/addr h))
     r (:probe::Echo/bump c (:probe::Echo::BumpRequest))]
    (:wat::core::match r -> :wat::core::i64
      ((:probe::Echo::BumpResponse::Ok value) value)
      ((:probe::Echo::BumpResponse::RequestTooLarge bytes cap)
        (:wat::kernel::assertion-failed! "compute-bump: unexpected RequestTooLarge"
          :wat::core::None :wat::core::None)))))
