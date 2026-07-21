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
;; Asserts `file`, NOT `symbol`: inside an anonymous fn body (e.g. a service `:impls` closure, or
;; — as here — `deftest'`'s own anonymous test-body wrapper, invoked via wat/spawn.wat machinery),
;; `Frame/symbol` is `None` (a known arc-109 wart — anon fns lack a structured symbol) while
;; `Frame/file` is ALWAYS populated, anon or named. `file` is therefore the robust, meaningful
;; gate — it proves `emitted-from` carries the caller's real WHERE, the whole point of caller.2.
;; (Not content-checked against this fixture's own filename: the immediate caller here is
;; `deftest'`'s dispatch machinery, not this file — `Some` presence is the portable assertion.)
;;
;; Log construction mirrors tests/services/probe_arc278_journal_logs_on_process.wat's l1/l2
;; (full Scope fields + own fields, kwargs ctor).

(:wat::core::defrecord :probe::Note [text <- :wat::core::String])

(:wat::test::deftest' :user::emitted-from-round-trips ()
  (:wat::core::let
    [tags   (:wat::core::HashMap :wat::core::keyword :wat::core::String)
     log    (:wat::telemetry::Log :namespace "probe-ns" :uuid (:wat::core::Uuid/nil) :tags tags
              :time-ns 1000000000 :emitted-from (:wat::kernel::call-site)
              :level :wat::telemetry::Level::Info
              :message (:wat::edn::write (:probe::Note :text "emitted-from")))
     frame  (:wat::telemetry::Log/emitted-from log)
     file   (:wat::kernel::Frame/file frame)
     file-ok (:wat::core::match file -> :wat::core::bool
               ((:wat::core::Some _) true)
               (:wat::core::None     false))]
    (:wat::test::assert-true file-ok)))
