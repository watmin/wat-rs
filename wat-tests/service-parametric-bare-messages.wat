;; wat-tests/service-parametric-bare-messages.wat — arc 278: the surface-minted op alias STONE.
;;
;; `1ac85d96` forced every message on a parametric `:nature :Peer'` surface to spell the
;; surface's own params, even vacuously (`PutRequest<T>` whose fields name no `T` at all) —
;; because `wat/service.wat` derives each message type NAME by string concatenation, at expand
;; time, when the type registry is still empty; it cannot ask a message its real arity, so it
;; could only re-attach the surface's own params uniformly.
;;
;; THIS STONE removes that forcing. Rust holds the surface's `:features` at REGISTRATION time
;; (register_types, well after expand_all) and mints one `TypeDef::Alias` per op —
;; `<Surface>::<op>/Request` / `<Surface>::<op>/Response` — targeting each message's request/
;; response type EXACTLY as `:features` declared it. `wat/service.wat` now NAMES that alias
;; instead of guessing by concatenation, so a message that uses none of the surface's params is
;; free to say so, honestly, bare.
;;
;; RED before the stone (the message-params lock, src/types.rs, `1ac85d96`):
;;   op `put` in surface :wat-tests::BareBox<T>: its request type
;;   ":wat-tests::BareBox::PutRequest" must be declared parametric in EXACTLY this surface's
;;   type params, in order — `PutRequest<T>` (arc 278, the parametric protocol) …
;; GREEN after: the lock is deleted (the whole point of this stone is that the spelling is no
;; longer required), and the bare messages below resolve through the Rust-minted alias.
;;
;; WHAT THIS PROVES (beyond `--check`): the `<T>` service is STOOD UP and one `put` round-trips
;; on BOTH loci — the thread tier (the alias resolves in-process, verbatim values) and the
;; process tier (a forked child re-registers the surface from the shipped `service-forms`
;; bundle and the payload crosses as ENCODED EDN, decoded against the alias's target type). `T`
;; is pinned to `i64` at the `/start` seed, load-bearing in the STATE — the same shape
;; `service-parametric.wat` pins, minus the vacuous `<T>` this stone retires.

;; ── the surface: parametric, messages BARE (the whole point) ────────────────────────────────
(:wat::core::defsurface :wat-tests::BareBox :- [T] :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :wat-tests::BareBox::PutRequest [item <- :wat::core::i64])
   (:wat::core::defenum :wat-tests::BareBox::PutResponse :wat::enum::Pure
     ;; `echo` carries the handler's answer back so the round-trip asserts a VALUE, not just
     ;; "no crash": item + 1 when the generic durable holds something, item + 0 when empty.
     :Ok              [echo <- :wat::core::i64]
     ;; ruling A — every serviceable op-Response carries the protocol-tier too-large variant.
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  ;; Stone 16.3 — `:max-request-bytes` is MANDATORY on a `:nature :Peer'` op.
  [(put [self <- (:wat-tests::BareBox :- [T])  req <- :wat-tests::BareBox::PutRequest]
     -> :wat-tests::BareBox::PutResponse :max-request-bytes 1024)])

;; ── the parametric service ──────────────────────────────────────────────────────────────────
;; `held <- Option<T>` is the whole point: the durable record — and therefore ::State, ::Admin,
;; ::Status and ::Handle, each of which carries it — is generic in T.
(:wat::service::defservice :wat-tests::barebox-svc :- [T]
  :satisfies (:wat-tests::BareBox :- [T])
  :durable   [held <- (:wat::core::Option :- [T])]
  :ephemeral []
  :impls
  [(put [s ctx req]
     (:wat::service::Outcome::Reply s
       (:wat-tests::BareBox::PutResponse::Ok
         (:wat::core::i64::+
           (:wat-tests::BareBox::PutRequest/item req)
           ;; read the T-typed durable field generically — `v` is bound at type T
           (:wat::core::match
               (:wat-tests::barebox-svc::Record/held (:wat-tests::barebox-svc::State/durable s))
             ((:wat::core::Some v) 1)
             (:wat::core::None 0))))))])

;; ── the gate: stand it up, dial it, round-trip one call ──────────────────────────────────────
;; `T` is pinned to `i64` at the `/start` call site by the seed `(Some 42)`.
;; Expected: item 7 + 1 (durable is `Some`) = 8.
(:wat::core::defn :wat-tests::barebox/run [locus <- :wat::spawn::Locus] -> :wat::core::i64
  (:wat::core::let
    [h (:wat-tests::barebox-svc/start :locus locus
         :record (:wat-tests::barebox-svc::Record :held (:wat::core::Some 42)))
     c (:wat::core::match (:wat::kernel::connect (:wat-tests::barebox-svc::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected p) p)
         ((:wat::kernel::ConnectOutcome::Refused cz)
           (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cz) :wat::core::None :wat::core::None))
         ((:wat::kernel::ConnectOutcome::Rejected cz)
           (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cz) :wat::core::None :wat::core::None))
         ((:wat::kernel::ConnectOutcome::Failed cz)
           (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cz) :wat::core::None :wat::core::None)))
     r (:wat-tests::barebox-svc/put c (:wat-tests::BareBox::PutRequest :item 7))
     out (:wat::core::match r
           ((:wat::kernel::RecvOutcome::Message __recv)
             (:wat::core::match __recv
               ((:wat-tests::BareBox::PutResponse::Ok echo) echo)
               ;; terminal caller: an unexpected wire-breach must SURFACE, never swallow.
               ((:wat-tests::BareBox::PutResponse::RequestTooLarge bytes cap)
                 (:wat::kernel::assertion-failed! "barebox-svc put: unexpected RequestTooLarge"
                   :wat::core::None :wat::core::None))
               ((:wat-tests::BareBox::PutResponse::RequestMalformed mpath mexpected mgot)
                 (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None))))
           ((:wat::kernel::RecvOutcome::Lost __cause)
             (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Stopped
             (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Closed
             (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))
     _ (:wat-tests::barebox-svc/stop h)]
    out))

;; ── thread tier ─────────────────────────────────────────────────────────────────────────────
(:wat::test::deftest :wat-tests::service::parametric-bare-messages-round-trip-on-thread

  (:wat::test::assert-eq
    (:wat-tests::barebox/run (:wat::spawn::thread))
    8))

;; ── process tier ────────────────────────────────────────────────────────────────────────────
;; The half that could not be inferred from the thread tier: a forked child re-registers the
;; surface from the shipped `service-forms` bundle and the payload crosses as ENCODED EDN — so
;; the bare `PutRequest` resolves through the Rust-minted alias a SECOND time, independently, in
;; the child's own `register_types` pass.
(:wat::test::deftest :wat-tests::service::parametric-bare-messages-round-trip-on-process

  (:wat::test::assert-eq
    (:wat-tests::barebox/run (:wat::spawn::process))
    8))
