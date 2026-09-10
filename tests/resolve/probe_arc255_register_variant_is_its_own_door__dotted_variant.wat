;; Room ⑤a/⑤b, end to end from the surface: a `defenum` variant whose OWN name carries
;; a dot is refused at --check, via the variant-type registration door
;; (`TypeEnv::register_variant_type`) — the precondition on `register_variant`'s
;; `variant_leaf` argument fires before the composed name ever reaches `gate`.
(:wat::core::defenum :my::app::Foo :wat::enum::Pure :Bar.Baz :Quux)
