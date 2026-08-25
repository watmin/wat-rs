;; wat-tests/service-parametric-messages.wat — arc 278: the PARAMETRIC PROTOCOL, on the wire.
;;
;; The third and last of the parametric-service gates, and the only one where a type param is
;; load-bearing IN THE PAYLOAD:
;;
;;   service-parametric.wat             `(Box :- [T])`      — T lives in the STATE; the messages carry
;;   service-parametric-two-params.wat  `(Pair :- [K V])`     no field of type T/K/V at all.
;;   THIS FILE                          `(PCache :- [K V])` — K and V are FIELD types of the request
;;                                                      and the response. The WIRE carries them.
;;
;; ── what was broken ─────────────────────────────────────────────────────────────────────────
;; `synthesize_surface_protocol` (src/types.rs) minted `<S>::Op` / `<S>::Reply` with
;; `type_params: vec![]` while copying each surface member's request/response `TypeExpr`
;; VERBATIM — so `PCache::Op`'s `Get` variant field was `GetRequest<K,V>` with **K and V unbound
;; in the enum**. And `wat/service.wat` derived every message type NAME by concatenating the
;; surface's base with the op's pascal name (`{base}::{Op}Request`), a convention with no channel
;; at all for a message's own type arguments. The surface DECLARED a parametric protocol; the wire
;; silently stripped it. Read off `macroexpand`, not inferred — the emitted superset was
;; `(defenum :…::pcache-svc::Op :wat::enum::Pure :Get [req <- :…::PCache::GetRequest])`.
;;
;; ── what closes it ──────────────────────────────────────────────────────────────────────────
;; `Op`/`Reply` inherit `surface.type_params`, and `:satisfies (:S :- [K V])` is the macro's CHANNEL:
;; the clause is split into base + type-ARGS (`proto-base` / `proto-tp`, the same split helper
;; `fqdn-base`/`fqdn-tp` has used since the parametric `defservice` landed) and the args re-attach
;; at every TYPE position — `(Op :- [K V])`, `(Reply :- [K V])`, the `Peer'`/`Listener'`/`Address'` wire types,
;; the `<service>::Op` superset, the `Locus/launch` head, and the derived message names. Name
;; positions (ctors, accessors, patterns, `derive` edges, the `retag-op'` runtime discriminator)
;; stay at the BASE, because a runtime `type_path` has no params to carry.
;;
;; ── what this gate proves that a `--check` cannot ───────────────────────────────────────────
;; A surface can DECLARE a parametric protocol and still have a dead wire. So the `K,V`-parametric service
;; is STOOD UP — on the THREAD locus and on the PROCESS locus, same expectation, one token apart —
;; a client `connect'`s to `Handle/addr`, and THREE probes run down one connection. K=`String` and
;; V=`i64`, two DIFFERENT concrete types (a gate where they coincided could not tell a correct
;; instantiation from a shifted one). One assertion, three tokens:
;;
;;   ["alpha" "beta"]|33|7          (1) the round trip. Real K-typed Strings out, rendered back
;;                                      VERBATIM; real V-typed i64s (11+22) back; the concrete
;;                                      `limit` echoed. Tags without payload cannot spell this.
;;   Malformed["limit"]/…i64/String (2) the request-shape wall is LIVE on a parametric message —
;;                                      it walked past the K-typed field and BIT on the concrete
;;                                      one. Without this token, (3) would be indistinguishable
;;                                      from "the wall gave up on parametric messages".
;;   [1 2]|33|7                     (3) the BOUNDARY LINE, measured rather than claimed: a
;;                                      `(Vector :- [K])` slot carrying integers is ACCEPTED. wat erases
;;                                      type params, so inside `serve :- [K V]` there is no `K` to
;;                                      check against; the server does not pretend to a check it
;;                                      cannot make. K is pinned STATICALLY, at the caller.
;;
;; The process tier is not a bonus here — it is the half that could not be inferred. On the thread
;; tier the payload is a verbatim in-process value; across a process boundary it is ENCODED EDN and
;; DECODED against the declared field types (`edn_to_typed_value`), type-param positions and all.
;;
;; ── the message-params rule (checker-locked) ────────────────────────────────────────────────
;; A parametric serviceable surface's `:messages` must be declared parametric in EXACTLY the
;; surface's params. RETIRED 2026-08-21 — each message declares only what IT consumes, so and
;; `(GetResponse :- [K V])` names both — but the rule holds even for a message that names none of them
;; (see the two sibling gates), because the derivation is a MACRO: freeze runs `expand_all` before
;; `register_types`, so at the moment `wat/service.wat` builds those names the type registry is
;; still empty and it cannot ask a message how many params it has. It can only re-attach the
;; surface's own. A violation is a located `MalformedDecl` on the defsurface form.

;; ── the surface: messages PARAMETRIC, K and V in the payload ────────────────────────────────
(:wat::core::defsurface :wat-tests::PCache :- [K V] :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :wat-tests::PCache::GetRequest :- [K]
     ;; ONE type-param field and ONE concrete field, deliberately side by side: the request-shape
     ;; wall's reach is exactly the difference between them, and probes (2) and (3) below MEASURE
     ;; that difference instead of asserting it.
     [probes <- (:wat::core::Vector :- [K])
      limit  <- :wat::core::i64])
   (:wat::core::defenum :wat-tests::PCache::GetResponse :- [K V] :wat::enum::Pure
     ;; `echo` returns the K-typed probes, `results` the V-typed durable, `limit` the concrete
     ;; field. All three are read APART by the assertion, so a wire that dropped any one of
     ;; them — or that shifted K and V — is caught, not silently tolerated.
     :Ok              [echo    <- (:wat::core::Vector :- [K])
                       results <- (:wat::core::Vector :- [V])
                       limit   <- :wat::core::i64]
     ;; ruling A — every serviceable op-Response carries the protocol-tier too-large variant.
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     ;; arc 278 Stone 2 — and the request-SHAPE refusal, unconditionally generated.
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  ;; Stone 16.3 — `:max-request-bytes` is MANDATORY on a `:nature :Peer'` op.
  [(get [self <- (:wat-tests::PCache :- [K V])  req <- (:wat-tests::PCache::GetRequest :- [K])]
     -> (:wat-tests::PCache::GetResponse :- [K V]) :max-request-bytes 1024)])

;; ── the two-parameter service ───────────────────────────────────────────────────────────────
;; The handler is fully GENERIC: it echoes the K-typed probes it was handed and returns the
;; V-typed durable vector. It never constructs a K or a V — it cannot, and does not need to.
(:wat::service::defservice :wat-tests::pcache-svc :- [K V]
  :satisfies (:wat-tests::PCache :- [K V])
  :durable   [fills <- (:wat::core::Vector :- [V])]
  :ephemeral []
  :impls
  [(get [s ctx req]
     (:wat::service::Outcome::Reply s
       (:wat-tests::PCache::GetResponse::Ok
         (:wat-tests::PCache::GetRequest/probes req)
         (:wat-tests::pcache-svc::Record/fills (:wat-tests::pcache-svc::State/durable s))
         (:wat-tests::PCache::GetRequest/limit req))))])

;; ── the gate: stand it up, dial it, run the three probes ────────────────────────────────────
;; K is pinned to String and V to i64 at the `/start` + call sites — two DIFFERENT concrete types.
;; Sent:   probes = ["alpha" "beta"] (K = String), limit = 7 (concrete)
;; Seeded: fills  = [11 22]          (V = i64)
;;
;; The dial is a SEPARATELY TYPED verb (the `mal/dial` idiom, wat-tests/service-request-malformed
;; .wat) and that is load-bearing here, not stylistic: it is where K and V are PINNED. Inlined,
;; `connect'`'s argument would still have an open K at the point the §7 purity wall inspects the
;; peer's Reply type, and an unresolved type var is not pure. Naming the instantiation — a
;; two-level type-arg nest, `(Address' :- [(PCache::Op :- [String i64]) (PCache::Reply :- [String i64])])` — is the
;; honest fix AND a second assertion in its own right: the whole parametric protocol has to be
;; spellable by hand, at concrete args, for a caller to hold one.
(:wat::core::defn :wat-tests::pcache/dial
  [a <- (:wat::kernel::Address :- [(:wat-tests::PCache::Op :- [:wat::core::String :wat::core::i64]) (:wat-tests::PCache::Reply :- [:wat::core::String :wat::core::i64])])]
  -> (:wat::kernel::Peer :- [(:wat-tests::PCache::Op :- [:wat::core::String :wat::core::i64]) (:wat-tests::PCache::Reply :- [:wat::core::String :wat::core::i64])])
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    ((:wat::kernel::ConnectOutcome::Refused cz)
      (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cz) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected cz)
      (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cz) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed cz)
      (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cz) :wat::core::None :wat::core::None))))

(:wat::core::defn :wat-tests::pcache/label
  [r <- (:wat::kernel::RecvOutcome :- [(:wat-tests::PCache::GetResponse :- [:wat::core::String :wat::core::i64])])]
  -> :wat::core::String
  (:wat::core::match r
    ((:wat::kernel::RecvOutcome::Message __recv)
      (:wat::core::match __recv
        ((:wat-tests::PCache::GetResponse::Ok echo results limit)
          ;; READ THE VALUES APART — the K-typed vector rendered VERBATIM (its actual Strings,
          ;; not a length or a tag), the V-typed i64s summed, the concrete i64 field echoed. A
          ;; wire that carried tags but dropped payload, or that shifted K and V, cannot produce
          ;; this string. `edn::write` rather than `nth`+concat on the K side is deliberate: it
          ;; renders whatever actually arrived, which is what makes probe (3)'s answer legible
          ;; instead of a crash.
          (:wat::string::concat (:wat::edn::write echo)
            (:wat::string::concat "|"
              (:wat::string::concat
                (:wat::core::i64::to-string
                  (:wat::core::i64::+ (:wat::core::nth results 0)
                                      (:wat::core::nth results 1)))
                (:wat::string::concat "|"
                  (:wat::core::i64::to-string limit))))))
        ;; terminal caller: an unexpected wire-breach must SURFACE, never swallow.
        ((:wat-tests::PCache::GetResponse::RequestTooLarge bytes cap) "TooLarge")
        ((:wat-tests::PCache::GetResponse::RequestMalformed mpath mexpected mgot)
          (:wat::string::concat "Malformed"
            (:wat::string::concat (:wat::edn::write mpath)
              (:wat::string::concat "/" (:wat::string::concat mexpected
                (:wat::string::concat "/" mgot))))))))
    ((:wat::kernel::RecvOutcome::Lost __cause)
      (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Stopped
      (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Closed
      (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None))))

(:wat::core::defn :wat-tests::pcache/run [locus <- :wat::spawn::Locus] -> :wat::core::String
  (:wat::core::let
    [h (:wat-tests::pcache-svc/start :locus locus
         :record (:wat-tests::pcache-svc::Record
                   :fills (:wat::core::Vector :wat::core::i64 11 22)))
     c (:wat-tests::pcache/dial (:wat-tests::pcache-svc::Handle/addr h))
     ;; (1) THE ROUND TRIP — a well-formed parametric request, real K-typed Strings out,
     ;;     real V-typed i64s back.
     good (:wat-tests::pcache/label
            (:wat-tests::pcache-svc/get c
              (:wat-tests::PCache::GetRequest
                :probes (:wat::core::Vector :wat::core::String "alpha" "beta")
                :limit  7)))
     ;; (2) THE CONCRETE FIELD IS STILL ENFORCED — a wrong-typed `limit` under the correct tag
     ;;     is REFUSED by the request-shape wall, on both tiers. This is what stops the
     ;;     type-param opacity below from being indistinguishable from "the wall gave up on
     ;;     parametric messages": the wall is live, it walked past `probes` and bit on `limit`.
     bad  (:wat-tests::pcache/label
            (:wat-tests::pcache-svc/get c
              (:wat::edn::read
                "#wat-tests.PCache/GetRequest {:probes [\"alpha\" \"beta\"] :limit \"seven\"}")))
     ;; (3) THE TYPE-PARAM POSITION IS OPAQUE — and this is the honest, measured limit of the
     ;;     guarantee, not a claim in a comment. `probes` is declared `(Vector :- [K])`; here it
     ;;     arrives holding INTEGERS while the client that sent it is typed at K=String. The
     ;;     server ACCEPTS it, because wat erases type params: inside `serve :- [K V]` there is no
     ;;     `K` to check against, so refusing would be pretending to a check it cannot make.
     ;;     K is pinned STATICALLY at the caller instead — no well-typed program can reach here.
     ;;     `echo` therefore comes back holding those INTEGERS, and the assertion below records
     ;;     that verbatim (`[1 2]`). Read that token as the guarantee's boundary line, written
     ;;     down where it cannot be forgotten: the boundary enforces every CONCRETE field, and a
     ;;     type-param position it cannot enforce it does not pretend to.
     opaque (:wat-tests::pcache/label
              (:wat-tests::pcache-svc/get c
                (:wat::edn::read
                  "#wat-tests.PCache/GetRequest {:probes [1 2] :limit 7}")))
     _    (:wat-tests::pcache-svc/stop h)]
    (:wat::string::concat good
      (:wat::string::concat " | " (:wat::string::concat bad
        (:wat::string::concat " | " opaque))))))

;; ── thread tier ─────────────────────────────────────────────────────────────────────────────
(:wat::test::deftest :wat-tests::service::parametric-messages-round-trip-on-thread

  (:wat::test::assert-eq
    (:wat-tests::pcache/run (:wat::spawn::thread))
    "[\"alpha\" \"beta\"]|33|7 | Malformed[\"limit\"]/:wat::core::i64/String | [1 2]|33|7"))

;; ── process tier ────────────────────────────────────────────────────────────────────────────
;; The SAME expectation, one token apart — tier-generality is the requirement, not a bonus. This
;; is the half that could not be inferred from the thread tier: a forked child re-registers the
;; surface from the shipped `service-forms` bundle and the payload crosses as ENCODED EDN, so the
;; request record is DECODED against its declared field types on the way in (`edn_to_typed_value`)
;; instead of arriving as a verbatim in-process value. `probes <- (Vector :- [K])` is decoded there
;; too — the same type-param position the sanitization wall faces — so this test is what proves
;; the codec carries a parametric payload, not just that the thread tier never had to.
(:wat::test::deftest :wat-tests::service::parametric-messages-round-trip-on-process

  (:wat::test::assert-eq
    (:wat-tests::pcache/run (:wat::spawn::process))
    "[\"alpha\" \"beta\"]|33|7 | Malformed[\"limit\"]/:wat::core::i64/String | [1 2]|33|7"))
