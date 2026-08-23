;; Arc 278 — the recv'-outcome wall RED GATE (acceptance; reshaped from
;; probe_arc278_crash_split_measure.{rs,wat}). Asserts, all four paths
;; (panic/rterr × thread/process):
;;   ADMIN  (Handle/handle) MATCHES `RecvOutcome::Lost cause` as a VALUE (never a raise)
;;          and `(Failure/message cause)` CONTAINS the crash sentinel — the owner gets the reason.
;;   CLIENT (connected peer) MATCHES `RecvOutcome::Lost` (NEVER `::Closed` — the mute we
;;          killed) and its cause message does NOT contain the sentinel (a reason-free 500).
;; At HEAD (pre-reshape) recv' raised → no RecvOutcome to match → RED. GREEN once the
;; enum + recv' reshape + serve-dispatch broadcast land.
;;
;; EXACT DATA: each helper returns a STRUCTURED :probe::Outcome (the RecvOutcome variant that
;; matched + a deterministic `sentinel-present?` bool computed IN-WAT — the per-run-variable Failure
;; location never leaves wat; only its boolean RESULT crosses to the .rs golden). "wat stdio is edn
;; — assert the structure exactly" (builder, 2026-07-22; R55 REVOLVTIONE, NVLLA LARVA).
(:wat::core::defenum :probe::Outcome :wat::enum::Pure
  :Message []                                       ;; matched ::Message (the .rs asserts this NEVER happens)
  :Lost    [sentinel-present? <- :wat::core::bool]  ;; matched ::Lost — admin: true (reason carried); client: false (reason-free 500)
  ;; arc 278 #73 — matched ::Stopped (the .rs asserts this NEVER happens either: this probe
  ;; never asks the substrate to stop, it only crashes the peer). A structural twin of
  ;; ::Closed, not folded into it — a stop is neither the peer dying nor the peer closing.
  :Stopped []
  :Closed  [])                                      ;; matched ::Closed (the mute we killed — the .rs asserts this NEVER happens)

(:wat::core::defsurface :probe::Crash :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Crash::BoomRequest [])
   (:wat::core::defrecord :probe::Crash::BoomrtRequest [])
   (:wat::core::defenum :probe::Crash::BoomResponse :wat::enum::Pure
     :Ok              [ok <- :wat::core::bool]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])
   (:wat::core::defenum :probe::Crash::BoomrtResponse :wat::enum::Pure
     :Ok              [ok <- :wat::core::bool]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(boom   [self <- :probe::Crash  req <- :probe::Crash::BoomRequest] -> :probe::Crash::BoomResponse
     :max-request-bytes 524288)
   (boomrt [self <- :probe::Crash  req <- :probe::Crash::BoomrtRequest] -> :probe::Crash::BoomrtResponse
     :max-request-bytes 524288)])

;; boom = a PANIC (assertion-failed!). boomrt = a RUNTIME-ERROR (div-by-zero on the durable x=0).
(:wat::service::defservice :probe::crash
  :satisfies :probe::Crash
  :durable   [x <- :wat::core::i64]
  :ephemeral []
  :init (:wat::core::fn [record <- :probe::crash::Record] -> :probe::crash::State
          (:probe::crash::State :durable record))
  :impls
  [(boom [s ctx req]
     (:wat::kernel::assertion-failed!
       "BOOM-CRASH-SENTINEL-9173"
       (:wat::core::Some "boom")
       (:wat::core::Some "ok")))
   (boomrt [s ctx req]
     (:wat::core::let
       [zero (:probe::crash::Record/x (:probe::crash::State/durable s))
        _    (:wat::core::i64::quot 987654321 zero)]        ;; RTERR-QUOT-SENTINEL: DivisionByZero at runtime
       (:wat::service::Outcome::Reply s (:probe::Crash::BoomrtResponse::Ok true))))])

;; ── CLIENT helpers: raw connect' + send' + recv', MATCH the RecvOutcome directly. ────────────────────
;; On ::Lost → (Outcome::Lost false) — the reason-free administrative message does NOT carry the
;; sentinel (the client never gets the reason). ::Closed → Outcome::Closed (the mute — the .rs
;; asserts this NEVER happens). ::Message → Outcome::Message.
(:wat::core::defn :probe::client-boom-msg [h <- :probe::crash::Handle] -> :probe::Outcome
  (:wat::core::let
    [c  (:wat::core::match (:wat::kernel::connect (:probe::crash::Handle/addr h)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     _s (:wat::kernel::send c (:probe::Crash::Op::Boom (:probe::Crash::BoomRequest)))]
    (:wat::core::match (:wat::kernel::recv c)
      ((:wat::kernel::RecvOutcome::Message _m) (:probe::Outcome::Message))
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:probe::Outcome::Lost (:wat::core::string::contains? (:wat::kernel::LociDiedError/message cause) "BOOM-CRASH-SENTINEL-9173")))
      (:wat::kernel::RecvOutcome::Stopped (:probe::Outcome::Stopped))
      (:wat::kernel::RecvOutcome::Closed (:probe::Outcome::Closed)))))

(:wat::core::defn :probe::client-boomrt-msg [h <- :probe::crash::Handle] -> :probe::Outcome
  (:wat::core::let
    [c  (:wat::core::match (:wat::kernel::connect (:probe::crash::Handle/addr h)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     _s (:wat::kernel::send c (:probe::Crash::Op::Boomrt (:probe::Crash::BoomrtRequest)))]
    (:wat::core::match (:wat::kernel::recv c)
      ((:wat::kernel::RecvOutcome::Message _m) (:probe::Outcome::Message))
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:probe::Outcome::Lost (:wat::core::string::contains? (:wat::kernel::LociDiedError/message cause) "DivisionByZero")))
      (:wat::kernel::RecvOutcome::Stopped (:probe::Outcome::Stopped))
      (:wat::kernel::RecvOutcome::Closed (:probe::Outcome::Closed)))))

;; ── ADMIN helpers: raw send' the crashing op FIRE-AND-FORGET, then MATCH the Handle lineage peer. ────
;; On ::Lost → (Outcome::Lost true) — `(Failure/message cause)` CARRIES the sentinel (the owner gets
;; the exact reason). ::Closed → Outcome::Closed; ::Message → Outcome::Message (both asserted NEVER).
(:wat::core::defn :probe::admin-boom-msg [h <- :probe::crash::Handle] -> :probe::Outcome
  (:wat::core::let
    [c  (:wat::core::match (:wat::kernel::connect (:probe::crash::Handle/addr h)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     _s (:wat::kernel::send c (:probe::Crash::Op::Boom (:probe::Crash::BoomRequest)))]
    (:wat::core::match (:wat::kernel::recv (:probe::crash::Handle/handle h))
      ((:wat::kernel::RecvOutcome::Message _m) (:probe::Outcome::Message))
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:probe::Outcome::Lost (:wat::core::string::contains? (:wat::kernel::LociDiedError/message cause) "BOOM-CRASH-SENTINEL-9173")))
      (:wat::kernel::RecvOutcome::Stopped (:probe::Outcome::Stopped))
      (:wat::kernel::RecvOutcome::Closed (:probe::Outcome::Closed)))))

(:wat::core::defn :probe::admin-boomrt-msg [h <- :probe::crash::Handle] -> :probe::Outcome
  (:wat::core::let
    [c  (:wat::core::match (:wat::kernel::connect (:probe::crash::Handle/addr h)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     _s (:wat::kernel::send c (:probe::Crash::Op::Boomrt (:probe::Crash::BoomrtRequest)))]
    (:wat::core::match (:wat::kernel::recv (:probe::crash::Handle/handle h))
      ((:wat::kernel::RecvOutcome::Message _m) (:probe::Outcome::Message))
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:probe::Outcome::Lost (:wat::core::string::contains? (:wat::kernel::LociDiedError/message cause) "DivisionByZero")))
      (:wat::kernel::RecvOutcome::Stopped (:probe::Outcome::Stopped))
      (:wat::kernel::RecvOutcome::Closed (:probe::Outcome::Closed)))))

;; ── the 8 entrypoints: {boom,boomrt} × {thread,process} × {client,admin} ─────────────────────────────
(:wat::core::defn :user::boom-client-thread [] -> :probe::Outcome
  (:probe::client-boom-msg (:probe::crash/start :locus (:wat::spawn::thread)  :record (:probe::crash::Record :x 0))))
(:wat::core::defn :user::boom-admin-thread [] -> :probe::Outcome
  (:probe::admin-boom-msg  (:probe::crash/start :locus (:wat::spawn::thread)  :record (:probe::crash::Record :x 0))))
(:wat::core::defn :user::boom-client-process [] -> :probe::Outcome
  (:probe::client-boom-msg (:probe::crash/start :locus (:wat::spawn::process) :record (:probe::crash::Record :x 0))))
(:wat::core::defn :user::boom-admin-process [] -> :probe::Outcome
  (:probe::admin-boom-msg  (:probe::crash/start :locus (:wat::spawn::process) :record (:probe::crash::Record :x 0))))

(:wat::core::defn :user::boomrt-client-thread [] -> :probe::Outcome
  (:probe::client-boomrt-msg (:probe::crash/start :locus (:wat::spawn::thread)  :record (:probe::crash::Record :x 0))))
(:wat::core::defn :user::boomrt-admin-thread [] -> :probe::Outcome
  (:probe::admin-boomrt-msg  (:probe::crash/start :locus (:wat::spawn::thread)  :record (:probe::crash::Record :x 0))))
(:wat::core::defn :user::boomrt-client-process [] -> :probe::Outcome
  (:probe::client-boomrt-msg (:probe::crash/start :locus (:wat::spawn::process) :record (:probe::crash::Record :x 0))))
(:wat::core::defn :user::boomrt-admin-process [] -> :probe::Outcome
  (:probe::admin-boomrt-msg  (:probe::crash/start :locus (:wat::spawn::process) :record (:probe::crash::Record :x 0))))
