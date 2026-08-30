;; probe-mcp-reply-emit.wat — the ESCAPING primitive for the JSON-RPC reply.
;;
;; FIRST ATTEMPT (kept as the finding, not the code): I tried to build the reply envelope as
;; wat DATA — nested `HashMap/assoc` — and the checker refused it in one shot:
;;
;;   :wat::core::HashMap/assoc: parameter #3 expects :wat::core::String; got :wat::core::i64
;;
;; A wat `(HashMap :- [K V])` has ONE V. The reply envelope is heterogeneous by nature (a String
;; beside an i64 beside a bool beside a Vector of maps), so it CANNOT be a HashMap. That is
;; the asymmetry worth writing down: `read-json` WALKS heterogeneous JSON happily
;; (probe-mcp-nested-json-walk.wat), but nothing can CONSTRUCT it as a map. The only wat
;; aggregate with heterogeneous fields is a record — which is precisely why the envelope-as-
;; typed-data path needs `write-json-natural` to serve records (probe-json-natural-record.wat).
;;
;; So the envelope is either RECORDS (typed, needs that two-guard fix) or an INTERPOLATED
;; skeleton (no substrate change). The skeleton is only safe if the one variable slot — the
;; EDN payload — is properly escaped, because real EDN is full of double quotes:
;;
;;   #some.ns/Rec {:field "val" :another 42}
;;
;; That is what this measures. `write-json` of a String must come back as a QUOTED, ESCAPED
;; JSON string literal, ready to drop into the skeleton verbatim.

(:wat::core::defn :probe::edn-with-quotes [] -> :wat::core::String
  "#some.ns/Rec {:field \"val\" :another 42}")

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    ;; the raw EDN, as `edn::write` would hand it back
    (:wat::kernel::println (:probe::edn-with-quotes))
    ;; the same string, escaped for JSON — the drop-in for the skeleton's text slot
    (:wat::kernel::println (:wat::edn::write-json (:probe::edn-with-quotes)))))
