;; tests/macros/probe_arc265_acronym_registry_svc.wat — explicit fixture for
;; probe_arc265_acronym_registry.rs (SVC program), loaded via startup_from_file.
;;
;; defservice consults its namespace acronyms at expand time.
;; arc 291 4b-ii: State is now a defstruct; :durable mints ::Record.
(:wat::core::string::declare-acronyms :my::aws ["ACL"])
(:wat::service::defservice :my::aws
  :durable [count <- :wat::core::i64]
  :ephemeral []
  :ops
  [(:CreateWebACL [s <- :State n <- :wat::core::i64]
                  -> [value <- :wat::core::i64]
     (:wat::service::Outcome::Reply s (:my::aws::CreateWebACLResponse (:my::aws::Record/count (:my::aws::State/durable s)))))])

(:wat::core::defn :user::req-n [] -> :wat::core::i64
  (:my::aws::CreateWebACLRequest/n (:my::aws/create-web-acl-request 7)))
