;; tests/kernel/wat_harness_deps_dep_a.wat — dep-A source for wat_harness_deps.rs, read via
;; include_str! into a WatSource. Stands in for what an external wat crate's wat_sources()
;; would return (arc 015 slice 3a global-install-once architecture).
(:wat::core::defn :user::test::dep-a::label [] -> :wat::core::String "A")
