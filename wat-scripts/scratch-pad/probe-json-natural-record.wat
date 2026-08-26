;; probe-json-natural-record.wat — does `write-json-natural` serve a RECORD, or only a STRUCT?
;;
;; MEASURED by reading (`edn::render::value_to_json_natural`): the aggregate arm is guarded
;; `sv.nature == Nature::Struct`, and the TypeEnv lookup at `:2656` gates on
;; `Nature::Struct` again. A `Nature::Record` aggregate should therefore fall through the
;; `_ =>` at `:2744` to `value_to_edn_with` — the TAGGED walker — and come back as a
;; `#ns.Type {...}` sentinel with `:`-prefixed keyword keys instead of bare JSON keys.
;;
;; This probe stops that being a claim. Same field shape, twice: once as a struct
;; (the CONTROL — must already be bare-keyed JSON), once as a record (the SUBJECT).
;;
;; Arc 300 made RECORDS the guaranteed-pure-data aggregate; a struct may hold resources.
;; A natural-JSON writer that serves the resource-capable kind and skips the pure one is
;; backwards — a JSON-RPC reply is pure data. This measures how backwards.
;;
;;   CONTROL bare-keyed + SUBJECT bare-keyed  => no gap; the read of the match arms is wrong
;;   CONTROL bare-keyed + SUBJECT #-tagged    => the gap is real; Stone 2a is the fix

(:wat::core::defstruct :probe::ContentS
  [type <- :wat::core::String
   text <- :wat::core::String])

(:wat::core::defrecord :probe::ContentR
  [type <- :wat::core::String
   text <- :wat::core::String])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    ;; CONTROL — the struct. If this is not bare-keyed JSON, nothing below means anything.
    (:wat::kernel::println
      (:wat::edn::write-json-natural
        (:probe::ContentS :type "text" :text "42")))
    ;; SUBJECT — the identical shape as a record.
    (:wat::kernel::println
      (:wat::edn::write-json-natural
        (:probe::ContentR :type "text" :text "42")))))
