;; probe-mcp-wire.wat — the two assumptions `wat --mcp` would rest on.
;;
;; THE DESIGN (builder): the wire is JSON, the PAYLOAD is opaque EDN text —
;;
;;     {"edn":"#some.edn/Thing {:whatever 42}"}
;;
;; JSON never represents a wat value, so nothing is lost in translation (keywords,
;; rationals, bigints all survive as text). Same call as `Log.message` being an opaque
;; EDN-text String (arc 278 Stone B) — the carrier does not decode, the consumer does.
;;
;; ASSUMPTION 1 — `read-frame` can read a JSON line INTACT.
;;   read-frame's scanner is EDN-AWARE: repl.wat documents that it "continues only while the
;;   prefix is INCOMPLETE EDN and terminates on MALFORMED". A JSON object opens `{` exactly as
;;   an EDN map does, so whether `{"edn":"…"}` comes back whole or gets mis-framed is a
;;   MEASUREMENT. Everything else rests on it.
;;
;; ASSUMPTION 2 — `write-json` on a String-keyed map yields BARE keys.
;;   The reply must be `{"edn":"…"}`, not `{":edn":"…"}`. wat-edn's reader treats a JSON
;;   string opening with `:` as a keyword; the WRITE direction's key rendering is untested here.
;;
;; Run:  printf '{"edn":"(:wat::core::i64::+ 1 2)"}\n' | ./target/release/wat <this file>

(:wat::core::defn :probe::show-frame [] -> :wat::core::nil
  (:wat::core::match (:wat::kernel::read-frame )
    ;; ASSUMPTION 1: echo the raw text back. If it is the whole JSON line, read-frame frames
    ;; JSON fine. If it is truncated at the first `:` or `"`, the EDN-aware scanner mis-frames
    ;; it and the mcp reader needs a different door.
    ((:wat::kernel::ReadFrameOutcome::Frame text)
      (:wat::kernel::println text))
    (:wat::kernel::ReadFrameOutcome::Eof nil)
    (:wat::kernel::ReadFrameOutcome::Stopped nil)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    ;; ASSUMPTION 2: a String-keyed map through write-json.
    (:wat::kernel::println
      (:wat::edn::write-json
        (:wat::hashmap::assoc
          (:wat::core::HashMap :- [:wat::core::String :wat::core::String])
          "edn" "#some.edn/Thing {:whatever 42}")
        (:wat::edn::opts)))
    (:probe::show-frame)))
