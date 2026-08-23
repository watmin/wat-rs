;; probe-arc278-74-literal-rtl-ctor.wat — #74, the ONE unproven mechanism.
;;
;; #74's ruled design ("convention is law — enforce `<Op>Response`") deletes the
;; RESPONSE-TYPE runtime constant and both EDN-decode branches, and lets
;; `wat/service.wat` go back to splicing a LITERAL constructor keyword:
;;
;;     (<proto-base>::<variant-pascal>Response::RequestTooLarge n cap)
;;
;; Every other part of that design is a DELETION and needs no proof. This is the
;; one CONSTRUCTION, and it has one live risk: a PARAMETRIC response enum.
;;
;; `proto-base` is the surface name with its type args STRIPPED (service.wat:268),
;; and `variant-pascal` is `kebab->pascal-in surface-kw op-str` (service.wat:890).
;; So for a surface `:probe::PCtor<K,V>` whose op `get` returns
;; `:probe::PCtor::GetResponse<V>`, the macro would splice the BARE base name
;; `:probe::PCtor::GetResponse::RequestTooLarge` — with no `<K,V>` anywhere.
;;
;; `wat-tests/service-parametric-messages.wat`'s header CLAIMS this is fine
;; ("Name positions (ctors, accessors, patterns …) stay at the BASE, because a
;; runtime `type_path` has no params to carry"). That is a claim in a doc header
;; about name positions in general. This probe makes it a RUN, about this exact
;; call, before a rider is briefed on it.
;;
;; PASS = this file type-checks. FAIL = #74's deletion cannot restore the literal
;; ctor for parametric responses, and the stone needs a parametric arm.
;;
;; (Two form-corrections the checker taught while writing this, kept visible:
;;  `defrecord` not `recordtype`; and a parametric surface's `:messages` must be
;;  declared parametric in EXACTLY the surface's params, in order — so the
;;  response is `GetResponse<V>` — it names only V, which is all its fields use (rule retired 2026-08-21).)

;; ── the SUBJECT: a parametric serviceable surface, params load-bearing in BOTH
;;    the request and the response payload (the shape service-parametric-messages
;;    proved on the wire).
(:wat::core::defsurface :probe::PCtor :- [K V] :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::PCtor::GetRequest :- [K]
     [probes <- (:wat::core::Vector :- [K])
      limit  <- :wat::core::i64])
   (:wat::core::defenum :probe::PCtor::GetResponse :- [V] :wat::enum::Pure
     :Ok               [results <- (:wat::core::Vector :- [V])]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path     <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String
                        got      <- :wat::core::String])]
  :features
  [(get [self <- (:probe::PCtor :- [K V])  req <- (:probe::PCtor::GetRequest :- [K])]
     -> (:probe::PCtor::GetResponse :- [V]) :max-request-bytes 1024)])

;; ★ THE CLAIM UNDER TEST — a LITERAL ctor call naming the BARE base of a
;;   PARAMETRIC response enum, in exactly the position the macro will splice it.
(:wat::core::defn :probe::mk-rtl-parametric []
    -> (:probe::PCtor::GetResponse :- [:wat::core::i64])
  (:probe::PCtor::GetResponse::RequestTooLarge 9999 1024))

;; The RequestMalformed twin — the same strike lands on it, so it is under test too.
(:wat::core::defn :probe::mk-rm-parametric []
    -> (:probe::PCtor::GetResponse :- [:wat::core::i64])
  (:probe::PCtor::GetResponse::RequestMalformed
    (:wat::core::Vector :wat::core::String "limit") "i64" "String"))

;; ── NON-VACUITY CONTROL: the MONOMORPHIC case, which the pre-#72 concatenation
;;    already built literally for the whole corpus. If the parametric arms above
;;    pass but this fails, the probe is measuring something other than its subject.
(:wat::core::defsurface :probe::MCtor :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::MCtor::PutRequest [k <- :wat::core::String])
   (:wat::core::defenum :probe::MCtor::PutResponse :wat::enum::Pure
     :Ok               [n <- :wat::core::i64]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path     <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String
                        got      <- :wat::core::String])]
  :features
  [(put [self <- :probe::MCtor  req <- :probe::MCtor::PutRequest]
     -> :probe::MCtor::PutResponse :max-request-bytes 1024)])

(:wat::core::defn :probe::mk-rtl-mono [] -> :probe::MCtor::PutResponse
  (:probe::MCtor::PutResponse::RequestTooLarge 9999 1024))
