;; :wat::telemetry::ConsoleLogger — direct-to-stdio structured
;; logger.
;;
;; Producer-side recorder bound to a Console destination. Closure
;; over (caller-id, clock, con-tx, ack-tx, ack-rx, format). Per
;; emission: stamp the current time, identify the caller, render
;; `[time level caller entry]` as one EDN/JSON line, write through
;; the Console driver, BLOCK on ack until the bytes are durable.
;; No Service queue between producer and Console driver — direct
;; render-and-send in the producer's thread, with the in-memory
;; TCP discipline arc 089 slice 5 made universal.
;;
;; Why direct (no Service<E,G> shell): for dev/debug logging at
;; reasonable volume, queue-fronted indirection adds latency without
;; eliminating anything. High-throughput producers wanting decoupling
;; reach for the explicit Service shell + Console/dispatcher (arc
;; 081's factory). ConsoleLogger is the simple-and-honest path.
;;
;; Level routing — debug/info land on stdout (Console/out); warn/error
;; on stderr (Console/err). Custom levels (e.g. :trace, :fatal) fall
;; through to stdout. Levels are bare keywords, not an enum, so the
;; rendered line stays compact (`:info` vs the verbose tagged form
;; an enum unit-variant would render as).
;;
;; Output line shape:
;;   [#inst "<iso8601>" :<level> :<caller> <rendered-entry>]
;;
;; Built once per producer; passed by reference to the producer's
;; hot path. Substrate ships :debug / :info / :warn / :error
;; convenience methods so the call site reads `(/info logger entry)`
;; instead of `(/log logger :info entry)` — the universal /log form
;; is there for callers that want to compute the level dynamically.

;; The line shape — named-field struct so EDN/JSON renderers emit a
;; map keyed by field names. Fields in producer-eyes order
;; (time, level, caller, data).
(:wat::core::struct :wat::telemetry::LogLine<E>
  (time :wat::time::Instant)
  (level :wat::core::keyword)
  (caller :wat::core::keyword)
  (data :E))


(:wat::core::struct :wat::telemetry::ConsoleLogger
  (con-handle :wat::std::service::Console::Handle)
  (caller :wat::core::keyword)
  (now-fn :fn(())->wat::time::Instant)
  (format :wat::telemetry::Console::Format))


;; Internal — build the LogLine struct + render it via the format-
;; selected wat-edn primitive. Format dispatch mirrors arc 081's
;; Console/dispatcher.
(:wat::core::define
  (:wat::telemetry::ConsoleLogger::render-line<E>
    (logger :wat::telemetry::ConsoleLogger)
    (now :wat::time::Instant)
    (level :wat::core::keyword)
    (entry :E)
    -> :String)
  (:wat::core::let*
    (((caller :wat::core::keyword)
      (:wat::telemetry::ConsoleLogger/caller logger))
     ((format :wat::telemetry::Console::Format)
      (:wat::telemetry::ConsoleLogger/format logger))
     ((line-struct :wat::telemetry::LogLine<E>)
      (:wat::telemetry::LogLine/new now level caller entry))
     ((line :String)
      (:wat::core::match format -> :String
        (:wat::telemetry::Console::Format::Edn
          (:wat::edn::write line-struct))
        (:wat::telemetry::Console::Format::Json
          (:wat::edn::write-json line-struct))
        (:wat::telemetry::Console::Format::Pretty
          (:wat::edn::write-pretty line-struct))
        (:wat::telemetry::Console::Format::NoTagEdn
          (:wat::edn::write-notag line-struct))
        (:wat::telemetry::Console::Format::NoTagJson
          (:wat::edn::write-json-natural line-struct)))))
    (:wat::core::string::concat line "\n")))


;; Internal — pick stdout vs stderr from the level keyword.
;; :debug + :info → stdout. :warn + :error → stderr. Other levels
;; default to stdout (per the convention "structured signals go
;; through stdout; only WARN/ERROR break to stderr").
(:wat::core::define
  (:wat::telemetry::ConsoleLogger::route-by-level
    (logger :wat::telemetry::ConsoleLogger)
    (level :wat::core::keyword)
    (line :String)
    -> :())
  (:wat::core::let*
    (((handle :wat::std::service::Console::Handle)
      (:wat::telemetry::ConsoleLogger/con-handle logger))
     ((to-stderr :bool)
      (:wat::core::or
        (:wat::core::= level :warn)
        (:wat::core::= level :error))))
    (:wat::core::if to-stderr -> :()
      (:wat::std::service::Console/err handle line)
      (:wat::std::service::Console/out handle line))))


;; Universal log form. Caller passes the level explicitly. Use this
;; when the level is computed; otherwise prefer the convenience
;; methods.
(:wat::core::define
  (:wat::telemetry::ConsoleLogger/log<E>
    (logger :wat::telemetry::ConsoleLogger)
    (level :wat::core::keyword)
    (entry :E)
    -> :())
  (:wat::core::let*
    (((now-fn :fn(())->wat::time::Instant)
      (:wat::telemetry::ConsoleLogger/now-fn logger))
     ((now :wat::time::Instant) (now-fn ()))
     ((line :String)
      (:wat::telemetry::ConsoleLogger::render-line
        logger now level entry)))
    (:wat::telemetry::ConsoleLogger::route-by-level
      logger level line)))


;; Convenience — debug / info → stdout; warn / error → stderr.
;; Pure sugar over /log with the level baked in.

(:wat::core::define
  (:wat::telemetry::ConsoleLogger/debug<E>
    (logger :wat::telemetry::ConsoleLogger)
    (entry :E)
    -> :())
  (:wat::telemetry::ConsoleLogger/log logger :debug entry))

(:wat::core::define
  (:wat::telemetry::ConsoleLogger/info<E>
    (logger :wat::telemetry::ConsoleLogger)
    (entry :E)
    -> :())
  (:wat::telemetry::ConsoleLogger/log logger :info entry))

(:wat::core::define
  (:wat::telemetry::ConsoleLogger/warn<E>
    (logger :wat::telemetry::ConsoleLogger)
    (entry :E)
    -> :())
  (:wat::telemetry::ConsoleLogger/log logger :warn entry))

(:wat::core::define
  (:wat::telemetry::ConsoleLogger/error<E>
    (logger :wat::telemetry::ConsoleLogger)
    (entry :E)
    -> :())
  (:wat::telemetry::ConsoleLogger/log logger :error entry))
