;; tests/program/probe_arc213_program_edn_roundtrip.wat — the sample program from
;; BRIEF-213-SERIALIZER-BRIDGE.md, adjusted (see probe_arc213_program_edn_roundtrip.rs
;; doc comment for the two omitted `/`-in-name call forms). Read as raw text (NOT
;; startup_beside/startup_from_file — the .rs driver feeds this text straight into
;; `parse_all_with_file` / `program_to_edn` / `edn_to_program`, the WatAST<->EDN
;; bridge under test) and re-parsed by every test in the file. Still exercises every
;; collection shape: Map `{...}`, Set `#{...}`, Vector `[...]`, List `(...)`, plus
;; `:keys` destructure and multiple keyword namespaces.
(:wat::config::set-capacity-mode! :error)
(:wat::core::defstruct :myapp::Pt [x <- :wat::core::i64  y <- :wat::core::i64])
(:wat::core::defn :myapp::sum [p <- :myapp::Pt] -> :wat::core::i64
    (:wat::core::let [{:keys [x y]} p] (:wat::i64::+ x y)))
(:wat::core::defn :myapp::tags [] -> :wat::core::i64
    (:wat::core::let [m {:a 1 :b 2}  s #{:x :y :z}]
        42))
(:wat::core::defn :user::main [] -> :wat::core::nil nil)
