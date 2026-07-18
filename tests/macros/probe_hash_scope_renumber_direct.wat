;; tests/macros/probe_hash_scope_renumber_direct.wat — co-located fixture for
;; probe_hash_scope_renumber.rs's macro_alias_expands_to_same_hash_as_direct_primitive.
;;
;; Program B: the same primitive call as the companion
;; probe_hash_scope_renumber_alias.wat's expansion, written directly — no macro
;; involved.
(:my::prim 42 99 1 -1)
