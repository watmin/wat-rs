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
;; ── The WIRE wall, one layer out — CLOSED (arc 278, the parametric protocol) ────────────────
;; When this probe was written, adding a `defservice :probe::pcache-svc<K,V> :satisfies
;; :probe::PCache<K,V>` did NOT check: `synthesize_surface_protocol` minted `<S>::Op`/`<S>::Reply`
;; with `type_params: vec![]` while copying the surface member's TypeExpr verbatim (so `K` was
;; UNBOUND in the enum), and `wat/service.wat` derived every message type NAME by concatenating
;; the surface base with the op's pascal name — a convention with no channel for the message's own
;; type arguments. Read off `(:wat::core::macroexpand '(defservice …))`, the emitted superset was
;; `(defenum :…::pcache-svc::Op :wat::enum::Pure :Get [req <- :…::PCache::GetRequest])` — the `<K>`
;; gone, and the enum itself non-parametric while `Record<K,V>` `State<K,V>` `Admin<K,V>`
;; `Status<K,V>` `serve<K,V>` all carried the params.
;;
;; Both halves shipped. `Op`/`Reply` inherit `surface.type_params`; the macro splits the surface's
;; `:satisfies :S<K,V>` into base + type-ARGS and re-attaches the args at every TYPE position
;; (`proto-tp` / `proto-op-ty-str`, wat/service.wat). The channel for a message's type arguments
;; is the SURFACE's own params, checker-locked: a parametric serviceable surface's `:messages` must
;; declare exactly those params, in order — which is why `GetRequest` below carries `<K,V>` and not
;; the `<K>` it was first written with, even though no field of it names V.
;;
;; The WIRE gate — a `<K,V>` service stood up on the thread locus, K=String / V=i64, real typed
;; payloads asserted in both directions — is `wat-tests/service-parametric-messages.wat`. This file
;; stays as the DECLARATION-wall pin it always was.

(:wat::core::defsurface :probe::PCache :- [K V] :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::PCache::GetRequest :- [K]
     [probes <- (:wat::core::Vector :- [K])])
   (:wat::core::defenum :probe::PCache::GetResponse :- [K V] :wat::enum::Pure
     ;; `echo` carries the K-typed probes back so a WIRE gate — once the layer above is ruled —
     ;; can assert on the actual values in BOTH directions, not just "no crash".
     :Ok              [echo    <- (:wat::core::Vector :- [K])
                       results <- (:wat::core::Vector :- [(:wat::core::Option :- [V])])]
     ;; ruling A — every serviceable op-Response carries the protocol-tier too-large variant.
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  ;; Stone 16.3 — `:max-request-bytes` is MANDATORY on a `:nature :Peer'` op.
  [(get [self <- (:probe::PCache :- [K V])  req <- (:probe::PCache::GetRequest :- [K])]
     -> (:probe::PCache::GetResponse :- [K V]) :max-request-bytes 1024)])
