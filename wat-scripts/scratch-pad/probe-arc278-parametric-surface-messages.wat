;; probe-arc278-parametric-surface-messages.wat
;;
;; Arc 278 — a peer surface whose `:messages` are PARAMETRIC. This file is the GREEN half of a
;; two-part finding; the RED half is recorded below in prose because it CANNOT be written as a
;; loadable form (it does not check).
;;
;; ── GREEN (this file): the DECLARATION wall ─────────────────────────────────────────────────
;; Before the `:messages` base-name normalization (`message_is_declared`, src/types/surface.rs),
;; the surface below did not declare. Verbatim:
;;
;;   malformed :wat::core::defsurface declaration: surface :probe::PCache feature `get`
;;   references protocol type :probe::PCache::GetRequest which is not declared in this
;;   surface's :messages …
;;
;; Note the reported name carries no `<K>` while `:messages` declares `GetRequest<K>`. Two sides
;; of one string comparison, one of them normalized: the DECLARATION side stored the `:messages`
;; keyword verbatim (params included), while the REFERENCE side (`collect_user_type_paths`)
;; emitted a `TypeExpr::Parametric` leaf as its HEAD alone. Third in the series — `7336464e`
;; (`box-svc<T>::Record`, suffix appended past the params) and `10107da9` (the flat type-arg
;; split) were the same defect in different clothes. Membership is a question about a type's
;; IDENTITY, not its instantiation, so both walls now ask it of the BASE name. A message with no
;; type params has no `<` on either side → the normalization is the IDENTITY for it (verified:
;; the whole `.wat` corpus `--check --check-output edn` is byte-identical across the change).
;;
;; ── RED (NOT in this file): the WIRE wall, one layer out ─────────────────────────────────────
;; Adding a `(:wat::service::defservice :probe::pcache-svc<K,V> :satisfies :probe::PCache<K,V> …)`
;; to this file does NOT check. The declaration is fine; the SERVICE machinery cannot name a
;; parametric message. Read from `(:wat::core::macroexpand '(defservice …))`, not inferred:
;;
;;   (defenum :…::cx-svc::Op :wat::enum::Pure :Get [req <- :…::Cx::GetRequest])   ← `<K>` GONE,
;;       and `cx-svc::Op` is itself non-parametric while its siblings `Record<K>`, `State<K>`,
;;       `Admin<K>`, `Status<K>`, `serve<K>` all carry `<K>`.
;;   (defn :…::cx-svc/get [c <- :wat::kernel::Peer'<…::Cx::Op,…::Cx::Reply>
;;                         req <- :…::Cx::GetRequest]
;;     -> :wat::kernel::RecvOutcome<…::Cx::GetResponse> …)                        ← bare on both.
;;
;; The cause is that `wat/service.wat` derives every message type NAME by string concatenation
;; from the surface base + the op's pascal name — `req-ty` at :852 (handler impl) and :1247,
;; `resp-ty-str` at :1250 (client fn) — a convention that has no channel for the message's own
;; type arguments. On the Rust side, `synthesize_surface_protocol` (src/types.rs:2393-2402)
;; mints `<S>::Op` / `<S>::Reply` with `type_params: vec![]` while copying the surface member's
;; TypeExpr verbatim, so `Cx::Op`'s `Get` variant field is `GetRequest<K>` with **K unbound in
;; the enum**. The two Op enums then meet at a `(:wat::core::derive :…::Cx::Op :…::cx-svc::Op)`.
;;
;; This reproduces with a MONOMORPHIC service and a message referenced at a CONCRETE
;; instantiation (`req <- :…::GetRequest<wat::core::String>`), so it is not about the service's
;; genericity — it is the name derivation itself. Making the wire carry a parametric payload is
;; therefore a design extension across `synthesize_surface_protocol` + the whole `wat/service.wat`
;; generation pipeline (Op/Reply, client fn, serve-loop arms, the `Peer'<Op,Reply>` wire types),
;; plus the open question of whether the EDN decode ENFORCES `K` at the boundary. That is a
;; builder ruling, not a rider's — so this file stops at the declaration wall, honestly.

(:wat::core::defsurface :probe::PCache<K,V> :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :probe::PCache::GetRequest<K>
     [probes <- :wat::core::Vector<K>])
   (:wat::core::defenum :probe::PCache::GetResponse<K,V> :wat::enum::Pure
     ;; `echo` carries the K-typed probes back so a WIRE gate — once the layer above is ruled —
     ;; can assert on the actual values in BOTH directions, not just "no crash".
     :Ok              [echo    <- :wat::core::Vector<K>
                       results <- :wat::core::Vector<wat::core::Option<V>>]
     ;; ruling A — every serviceable op-Response carries the protocol-tier too-large variant.
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64])]
  :features
  ;; Stone 16.3 — `:max-request-bytes` is MANDATORY on a `:nature :Peer'` op.
  [(get [self <- :probe::PCache<K,V>  req <- :probe::PCache::GetRequest<K>]
     -> :probe::PCache::GetResponse<K,V> :max-request-bytes 1024)])
