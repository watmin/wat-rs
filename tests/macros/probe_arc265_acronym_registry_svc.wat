;; tests/macros/probe_arc265_acronym_registry_svc.wat — explicit fixture for
;; probe_arc265_acronym_registry.rs (SVC program), loaded via startup_from_file.
;;
;; Arc 278 S4c — a `:satisfies` service whose SURFACE owns its protocol (`:messages`) and whose
;; kebab surface-method carries an acronym. The surface's S1 protocol synthesis
;; (`synthesize_surface_protocol`) must consult the namespace-scoped acronym registry — keyed by
;; the surface's OWN name (`:my::aws::Waf`), EXACTLY as `defservice :impls` keys its lookup on the
;; satisfied surface (`kebab->pascal-in <surface-kw> <op>`). With `ACL` declared, the method
;; `create-web-acl` must synthesize the `::Op`/`::Reply` variant `CreateWebACL`, NOT `CreateWebAcl`.
;; `(:user::req-n)` constructs + matches that exact synthesized variant and round-trips 7 through
;; it — the program only type-checks/evals if the acronym casing carried through both the surface's
;; S1 synthesis and the service's `:impls` op-name derivation (the two paths must agree).
(:wat::string::declare-acronyms :my::aws::Waf ["ACL"])

(:wat::core::defsurface :my::aws::Waf :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :my::aws::Waf::CreateWebACLRequest  [n     <- :wat::core::i64])
   (:wat::core::defenum :my::aws::Waf::CreateWebACLResponse :wat::enum::Pure
     :Ok              [value <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(create-web-acl [self <- :my::aws::Waf  req <- :my::aws::Waf::CreateWebACLRequest]
                   -> :my::aws::Waf::CreateWebACLResponse :max-request-bytes 524288)])

(:wat::service::defservice :my::waf
  :satisfies :my::aws::Waf
  :durable   [count <- :wat::core::i64]
  :ephemeral []
  :impls
  [(create-web-acl [s ctx req]
     (:wat::service::Outcome::Continue s
       (:wat::core::Some (:my::aws::Waf::Reply::CreateWebACL (:my::aws::Waf::CreateWebACLResponse::Ok (:my::waf::Record/count (:my::waf::State/durable s))))) (:wat::core::Vector :- [(:wat::service::Directed :- [:my::aws::Waf::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:my::waf::Op])])))])

;; Prove the surface synthesized `:my::aws::Waf::Op::CreateWebACL` (acronym-cased). Constructing
;; and matching that EXACT variant type-checks + evals ONLY if S1 threaded the `ACL` acronym; with
;; the pre-fix `&[]` it would be `::Op::CreateWebAcl` and this name would not resolve.
(:wat::core::defn :user::req-n [] -> :wat::core::i64
  (:wat::core::match (:my::aws::Waf::Op::CreateWebACL (:my::aws::Waf::CreateWebACLRequest :n 7))
    
    ((:my::aws::Waf::Op::CreateWebACL req) (:my::aws::Waf::CreateWebACLRequest/n req))))
