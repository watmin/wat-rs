;; Co-located fixture for probe_arc278_emitted_from.rs — arc 278 "caller.2" acceptance gate.
;;
;; caller.2 flips the telemetry `caller` field (a forgeable hand-typed keyword) to
;; `emitted-from <- :wat::kernel::Frame` (the real captured call-site, via native
;; `(:wat::kernel::call-site)` — arc 278 "caller.1", tests/kernel/probe_arc278_call_site.wat).
;;
;; RED at HEAD (pre-flip): two type errors —
;;   `unknown field :emitted-from for aggregate :wat::telemetry::Log`
;;   `unknown callee: :wat::telemetry::Log/emitted-from`
;; GREEN after: startup succeeds; the deftest' body RETURNS (not raises) — the Log's
;; `emitted-from` Frame has a `Some` :file.
;;
;; Asserts `file`: `Frame/file` is ALWAYS a real wat source location, anon or named, so it is the
;; robust, portable gate — it proves `emitted-from` carries the caller's real WHERE, the whole point
;; of caller.2. (Arc 109 — Frame's fields are now concrete/non-Option; an anon fn's `symbol` is the
;; Fn TYPE `:wat::core::Fn`, no longer `None`. Still not symbol-content-checked here: the immediate
;; caller is `deftest'`'s dispatch machinery, not this file — the file-names-a-.wat check is portable.)
;;
;; Log construction mirrors tests/services/probe_arc278_journal_logs_on_process.wat's l1/l2
;; (full Scope fields + own fields, kwargs ctor).

(:wat::core::defrecord :probe::Note [text <- :wat::core::String])

(:wat::test::deftest :user::emitted-from-round-trips 
  (:wat::core::let
    [tags   (:wat::core::HashMap :- [:wat::core::keyword :wat::core::String])
     log    (:wat::telemetry::Log :namespace "probe-ns" :uuid (:wat::uuid::nil) :tags tags
              :time-ns 1000000000 :emitted-from (:wat::kernel::call-site)
              :level :wat::telemetry::Level::Info
              :message (:wat::edn::write (:probe::Note :text "emitted-from")))
     frame  (:wat::telemetry::Log/emitted-from log)
     ;; Arc 109 — Frame/file is a concrete (non-Option) String, always present;
     ;; assert it names a wat source location (the caller's real WHERE).
     file   (:wat::kernel::Frame/file frame)
     file-ok (:wat::string::contains? file ".wat")]
    (:wat::test::assert-true file-ok)))
