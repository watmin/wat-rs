;; STONE 251.8c mechanism reference — loadable, not the acceptance test.
;; The hole was NOT infer_list skipping Symbol heads on residue. Normalize
;; already rewrites residue to Keyword. check_program type-checks
;; FunctionBody snapshots taken at register_defines, before that rewrite.
;; Acceptance: check::tests::stone_251_8c_slash_and_colon_heads_share_the_call_path

(:wat::core::defn :user::f [n <- :wat::core::i64] -> :wat::core::i64 n)
