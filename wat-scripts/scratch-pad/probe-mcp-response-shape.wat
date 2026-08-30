;; probe-mcp-response-shape.wat — CAN wat build a JSON-RPC response?
;;
;; Stone 2 needs to EMIT nested, HETEROGENEOUS JSON:
;;
;;   {"jsonrpc":"2.0","id":1,"result":{"content":[{"type":"text","text":"42"}],"isError":false}}
;;
;; wat's HashMap is `(HashMap :- [K V])` — homogeneous in V — so a map cannot hold a String, an i64,
;; a bool and a nested object at once. RECORDS can (each field its own type). The question is
;; what they SERIALIZE to:
;;
;;   `write-json`          → `{"#tag":"ns/Name","body":{…}}` — the round-trip sentinel. WRONG for
;;                            MCP: a harness expects bare keys, not our tag envelope.
;;   `write-json-natural`  → documented (`edn::render::eval_edn_write_json_natural`) to DROP the #tag/body wrapping "so
;;                            struct fields land at the top level of the JSON object" and to drop
;;                            the `:` prefix from keywords. That is exactly MCP's shape — IF it
;;                            holds for NESTED records and vectors too, which is the untested part.
;;
;; Briefing Stone 2 on an unproven emit path is how the last brief built a wall. So: measure.
;;
;;   PASS: bare keys at every level, nested record inlined, vector of records as a JSON array.
;;   FAIL: any `#tag`/`body` envelope, or a `:`-prefixed key, anywhere in the output.

(:wat::core::defrecord :probe::Content
  [type <- :wat::core::String
   text <- :wat::core::String])

(:wat::core::defrecord :probe::Result
  [content <- (:wat::core::Vector :- [:probe::Content])
   isError <- :wat::core::bool])

(:wat::core::defrecord :probe::Response
  [jsonrpc <- :wat::core::String
   id      <- :wat::core::i64
   result  <- :probe::Result])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [reply (:probe::Response
             :jsonrpc "2.0"
             :id      1
             :result  (:probe::Result
                        :content (:wat::core::conj
                                   (:wat::core::Vector :- [:probe::Content])
                                   (:probe::Content :type "text" :text "42"))
                        :isError false))]
    (:wat::core::do
      ;; the sentinel form — expected to carry #tag/body, shown for contrast
      (:wat::kernel::println (:wat::edn::write-json reply))
      ;; the MCP candidate
      (:wat::kernel::println (:wat::edn::write-json-natural reply)))))
