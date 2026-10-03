;; tests/program/wat_arc170_program_contracts_t3_argv.wat — main with (:wat::runtime::argv) let-bind.
(wat.core/defn user/main [] :- wat.type/nil
  (wat.core/let
    [_ (wat.runtime/argv)]
    nil))
