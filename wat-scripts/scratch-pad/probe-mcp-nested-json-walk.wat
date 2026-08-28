;; probe-mcp-nested-json-walk.wat — can wat walk a NESTED JSON object?
;;
;; Stone 1's gate proved the FLAT case: `{"edn":"42"}` → `HashMap/get m "edn"` → `Some "42"`.
;; A real MCP request is not flat — the payload sits three levels down:
;;
;;   {"jsonrpc":"2.0","id":1,"method":"tools/call",
;;    "params":{"name":"eval","arguments":{"edn":"(:wat::core::+ 2 2)"}}}
;;
;; and the sibling values at each level are HETEROGENEOUS (a String beside an i64 beside a
;; nested object). A wat `(HashMap :- [K V])` has ONE V. So this is the question that decides where
;; the JSON-RPC envelope is parsed:
;;
;;   WALKS  => wat can own the envelope; `wat/mcp.wat` reads the request end to end.
;;   REFUSED => the envelope belongs to the CLI shim (Rust/serde), and wat's surface is
;;              exactly what the builder ruled — an EDN string in, an EDN string out.
;;
;; Either answer is fine; guessing between them is not. Run it and read what the checker says.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::match
      (:wat::edn::read-json
        "{\"id\":1,\"method\":\"tools/call\",\"params\":{\"name\":\"eval\",\"arguments\":{\"edn\":\"(:wat::core::+ 2 2)\"}}}")

    ((:wat::edn::ReadJsonOutcome::Value top)
      (:wat::core::match (:wat::hashmap::get top "params")
        ((:wat::core::Some params)
          (:wat::core::match (:wat::hashmap::get params "arguments")
            ((:wat::core::Some args)
              (:wat::core::match (:wat::hashmap::get args "edn")
                ((:wat::core::Some s)
                  (:wat::kernel::println (:wat::string::concat "WALKS -> " s)))
                (:wat::core::None (:wat::kernel::println "MISS at edn"))))
            (:wat::core::None (:wat::kernel::println "MISS at arguments"))))
        (:wat::core::None (:wat::kernel::println "MISS at params"))))

    ((:wat::edn::ReadJsonOutcome::Malformed cause)
      (:wat::kernel::println cause))))
