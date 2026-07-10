;; arc 293 — inheritance ANNIHILATED. A recordtype/aggregatetype parent MUST be a nature-root
;; (:wat::core::Value / :wat::core::Struct / :wat::core::Record / :wat::holon::Record). A USER-type
;; parent is nominal inheritance — REJECTED at registration. Reuse-of-shape is surface-splice, not a base.
;;
;; RED at HEAD: register_with_span (types.rs:457) registers :my::Child <: :my::Base for ANY existing
;; parent → startup SUCCEEDS. GREEN once the nature-root guard rejects the non-nature-root parent.
(:wat::core::defrecord :my::Base [x <- :wat::core::i64])
(:wat::core::recordtype :my::Child :my::Base [y <- :wat::core::i64])
