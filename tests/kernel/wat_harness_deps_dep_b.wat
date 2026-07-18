;; tests/kernel/wat_harness_deps_dep_b.wat — dep-B source for wat_harness_deps.rs, read via
;; include_str! into a WatSource. Stands in for what an external wat crate's wat_sources()
;; would return (arc 015 slice 3a global-install-once architecture).
(:wat::core::defn :user::test::dep-b::label [] -> :wat::core::String "B")
