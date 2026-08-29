;; probe-arc278-child-entry-static-call.wat — THE DISCONFIRMING PROBE for the child-entry strike.
;;
;; THE STRIKE IT GUARDS. `defservice`'s generated child main reaches its own internals through
;; `(:wat::core::apply (:wat::core::keyword/from-string "<fqdn>::serve") …)` — a call that exists
;; BECAUSE it does not resolve statically, so no closure walk can follow it. The strike replaces
;; that with a per-service `<fqdn>::child-entry` — a REAL parent defn that names `serve` and
;; `dispatch-admin` statically — so ONE `fn-forms` over it reaches everything and the
;; hand-enumerated `service-forms` manifest can die.
;;
;; TWO load-bearing claims, and this file fails on EXACTLY them if either is false:
;;
;;   A. A parent defn can hand the process tier's `(Peer' :- [Status Admin])` to `serve`, whose `self`
;;      is declared `(ThreadSelfPeer' :- [Status Admin])`. This is what the 293.W derive edge bought
;;      (`Peer' derives ThreadSelfPeer'`, wat/spawn.wat). Before that edge this was a located
;;      TypeMismatch — i.e. the strike was IMPOSSIBLE, not merely unwritten.
;;
;;   B. `fn-forms` rooted at that defn REACHES the service internals — `serve` and
;;      `dispatch-admin` must appear among the shipped declarations. If the walk stops at the
;;      defn itself, the one-entry model does not work and the manifest cannot be retired.
;;
;; ⚠ NON-VACUITY. Claim B is asserted by NAME MEMBERSHIP over the shipped forms, and a
;; membership check that finds nothing looks identical to one that was never run. So the probe
;; also prints the full declared-name set and its size: a reader can see the walk produced a
;; real closure, not an empty vector that trivially "contains" nothing.

(:wat::core::defsurface :probe::CE :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::CE::PingRequest [])
   (:wat::core::defenum :probe::CE::PingResponse :wat::enum::Pure
     :Ok               [ok <- :wat::core::bool]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(ping [self <- :probe::CE  req <- :probe::CE::PingRequest] -> :probe::CE::PingResponse :max-request-bytes 524288)])

(:wat::service::defservice :probe::ce
  :satisfies :probe::CE
  :durable   [n <- :wat::core::i64]
  :ephemeral []
  :init (:wat::core::fn [record <- :probe::ce::Record] -> :probe::ce::State
          (:probe::ce::State :durable record))
  :impls
  [(ping [s ctx req]
     (:wat::service::Outcome::Reply s (:probe::CE::PingResponse::Ok true)))])

;; ── CLAIM A — the STATIC call, with a wire-safe `Peer'` in the `self` slot ────────────────────
;; This is the shape `<fqdn>::child-entry` will have. Every argument is spelled exactly as the
;; generated `serve` declares it (service.wat:1478 `serve-params`), so a rider copying this is
;; copying a form the checker has already accepted.
;;
;; ★ The `self` here is `(Peer' :- [Status Admin])` — what `:wat::program::self-peer` returns in the
;; forked child — NOT the `ThreadSelfPeer'` serve declares. It type-checks only via the derive
;; edge. Flip either head and this file goes red, which is the disconfirmation.
;; Mirrors the generated child main's real flow — dispatch-admin THEN serve — so the closure
;; walk below is rooted at something with the same callee set the strike will have.
(:wat::core::defn :probe::ce::child-entry-shape
  [self <- (:wat::kernel::Peer :- [:probe::ce::Status :probe::ce::Admin])
   l    <- (:wat::kernel::Listener :- [:probe::CE::Op :probe::CE::Reply])
   ship <- :probe::ce::Admin]
  -> :wat::core::nil
  (:wat::core::let
    [state (:probe::ce::dispatch-admin ship)]
  (:probe::ce::serve self l
    ;; the selectables slot: `(Vector :- [(Tuple :- [i64 (Peer :- [Reply Op])])])` — the id travels WITH its peer
    ;; (arc 278 the call context). The element type is ONE tuple type-keyword, exactly as
    ;; `selectable-entry-ty` builds it (service.wat:979).
    (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 (:wat::kernel::Peer :- [:probe::CE::Reply :probe::ce::Op])])])
    0
    state)))

;; ── CLAIM B — does a closure walk rooted HERE reach the service internals? ────────────────────
(:wat::core::defn :user::declared-names
  [forms <- (:wat::core::Vector :- [:wat::WatAST])
   i     <- :wat::core::i64
   acc   <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::if (:wat::i64::>= i (:wat::core::length forms))
    acc
    (:wat::core::let
      [form (:wat::core::nth forms i)
       ch   (:wat::core::ast->children form)
       head (:wat::core::ast-name (:wat::core::first ch))
       nm   (:wat::core::if (:wat::core::= head ":wat::core::do")
              (:wat::core::ast-name
                (:wat::core::first (:wat::core::rest (:wat::core::ast->children
                  (:wat::core::first (:wat::core::rest ch))))))
              (:wat::core::ast-name (:wat::core::first (:wat::core::rest ch))))]
      (:user::declared-names forms (:wat::i64::+ i 1) (:wat::core::conj acc nm)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [forms (:wat::kernel::fn-forms :probe::ce::child-entry-shape :user::root-entry)
     names (:user::declared-names forms 0 (:wat::core::Vector :- [:wat::core::String]))
     _n    (:wat::kernel::println
             (:wat::string::concat "closure forms="
               (:wat::i64::to-string (:wat::core::length forms))))
     _d    (:wat::kernel::println names)
     ;; CLAIM B, asserted by membership — with the full set printed above so an empty walk
     ;; cannot masquerade as a pass.
     hit-s (:wat::fix::str-in? ":probe::ce::serve" names)
     hit-d (:wat::fix::str-in? ":probe::ce::dispatch-admin" names)]
    (:wat::core::if (:wat::core::and hit-s hit-d)
      (:wat::kernel::println "CLAIM-B PASS — the walk reaches serve AND dispatch-admin")
      (:wat::kernel::println "CLAIM-B FAIL — the walk does NOT reach both; read the name set above"))))
