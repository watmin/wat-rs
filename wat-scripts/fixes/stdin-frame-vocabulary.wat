;; wat-scripts/fixes/stdin-frame-vocabulary.wat — arc 170 closure #4, a ratified intueri rename.
;;
;; `stdio.wat`'s Layer-2 stdin op calls `IOReader/read-frame` and relabels the result a "line" —
;; a frame is one-or-more physical lines accumulated until the buffer forms a complete EDN value,
;; not a single line. Four Level-1 names LIE about this and are renamed here:
;;
;;   :wat::kernel::StdIn/read-line               -> :wat::kernel::StdIn/read-frame
;;   :wat::kernel::StdIn::ReadLineRequest         -> :wat::kernel::StdIn::ReadFrameRequest
;;   :wat::kernel::StdIn::ReadLineResponse        -> :wat::kernel::StdIn::ReadFrameResponse
;;   :wat::kernel::StdIn::ReadLineResponse::Line  -> :wat::kernel::StdIn::ReadFrameResponse::Frame
;;
;; PLUS a fifth, structurally required, pair: the `defenum`'s own variant DECLARATION spells the
;; variant as the bare (unqualified) keyword `:Line` — `(:wat::core::defenum …ReadLineResponse …
;; :Line [line <- …] :Eof [] …)`. That bare keyword shares NO substring with the fully-qualified
;; `…ReadLineResponse::Line` construction/match sites above (its ast-name is just "Line", 4 chars —
;; too short for pair 1's ~45-char old-bare to match at all), so it needs its OWN pair:
;;   :Line -> :Frame
;; Verified narrow (grep, both files): the ONLY bare `:Line` keyword token in either file in scope
;; is this one declaration site (`stdio.wat:88`); the probe `.wat` has none (only the fully-qualified
;; form, already caught by pair 1). Caught by the type checker on the first real dry-run + gate pass
;; (a missing arm: the enum still declared `Line` while every call site now matched `Frame`) — kept
;; visible here rather than silently folded away.
;;
;; ANCHORS — ruled correct, NOT touched by any pair below: `:wat::io::IOReader/read-line`
;; (a genuinely-different verb — reads exactly one physical line), `:wat::io::IOReader::
;; ReadFrameOutcome`, `:wat::kernel::ReadFrameOutcome`, and `readln`/`println`/`eprintln`. The
;; `Eof`/`Stopped`/`RequestTooLarge`/`RequestMalformed` variants keep their names — only the
;; `::Line` variant lied.
;;
;; ⚠ PREFIX, NOT EXACT — `rename-keyword-exact` keys on the FULL ast-name, so a sub-path use
;; like `…ReadLineResponse::Eof` would be left byte-identical (only a standalone `ReadLineResponse`
;; token would match). `rename-keyword-prefix` is boundary-aware: it matches a qualified name
;; anchored at the START of the keyword token (or at a type-arg embedding), so a single pair
;; for the ENCLOSING type reaches every one of its accessor/variant continuations
;; (`::ReadLineResponse::Eof`, `::ReadLineResponse::Stopped`, …) in one pass.
;;
;; ORDER — most-specific pair FIRST: `::Line` -> `::Frame` is a single compound rewrite of the
;; WHOLE `…ReadLineResponse::Line` token straight to the final `…ReadFrameResponse::Frame` form,
;; and it MUST run before the enclosing `ReadLineResponse` -> `ReadFrameResponse` pair — once that
;; pair has run, the token reads `…ReadFrameResponse::Line` and the compound pair's `old` string
;; (`…ReadLineResponse::Line`) no longer exists in the source to match. The remaining three pairs
;; (`ReadLineRequest`, `StdIn/read-line`, bare `:Line`) are disjoint substrings and interact with
;; nothing else — in particular the bare `:Line` pair's old-bare ("Line", 4 chars) cannot match
;; ANYWHERE inside the ~45-char compound token from pair 1, so its position in the list is free.
;;
;; Idempotent by construction: every pair's `new` no longer contains any `old` substring, so a
;; re-run matches nothing.
;;
;; Closed: `wat/fix.wat` now carries `:wat::fix::rename-symbol-exact` (the symbol-kind sibling
;; of `rename-keyword-exact`, purely additive) — the SYMBOL sixth pair below (`read-line` ->
;; `read-frame`) reaches stdio.wat's `defsurface :features` member (:101) and `defservice
;; :impls` arm (:112), the two op-HEAD sites the five keyword pairs above cannot touch.
;;
;; Usage:
;;   printf '["wat/kernel/services/stdio.wat" "tests/services/probe_arc170_stdio_prime.wat"]\n' \
;;     | ./target/release/wat ./wat-scripts/fixes/stdin-frame-vocabulary.wat

;; The migration as DATA — one line per pair, most-specific first (see ORDER above).
(:wat::core::defn :user::renames [] -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])])
  (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::String :wat::core::String])
    ;; compound — the actual lie, rewritten straight to its final form
    (:wat::core::Tuple ":wat::kernel::StdIn::ReadLineResponse::Line" ":wat::kernel::StdIn::ReadFrameResponse::Frame")
    ;; the enclosing response enum — catches ::Eof/::Stopped/::RequestTooLarge/::RequestMalformed
    ;; accessors and bare type refs (defenum head, -> annotations)
    (:wat::core::Tuple ":wat::kernel::StdIn::ReadLineResponse"       ":wat::kernel::StdIn::ReadFrameResponse")
    ;; the request record — bare type refs and the ::max-buffer-bytes accessor
    (:wat::core::Tuple ":wat::kernel::StdIn::ReadLineRequest"        ":wat::kernel::StdIn::ReadFrameRequest")
    ;; the op invocation keyword
    (:wat::core::Tuple ":wat::kernel::StdIn/read-line"                ":wat::kernel::StdIn/read-frame")
    ;; the defenum's own bare variant declaration (structurally required — see header)
    (:wat::core::Tuple ":Line"                                        ":Frame")))

(:wat::core::defn :user::migrate
  [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let [kw-migrated
                     (:wat::core::foldl
                       (:wat::core::fn [acc <- :wat::core::String
                                        pr  <- (:wat::core::Tuple :- [:wat::core::String :wat::core::String])] -> :wat::core::String
                         (:wat::fix::rename-keyword-prefix (:wat::core::first pr) (:wat::core::second pr) acc))
                       src
                       (:user::renames))]
    ;; sixth pair, SYMBOL-kind not keyword — the defsurface/defservice op HEAD (`read-line`,
    ;; e.g. stdio.wat:101/:112) is a bare symbol, unreachable by rename-keyword-prefix/-exact.
    (:wat::fix::rename-symbol-exact "read-line" "read-frame" kw-migrated)))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path
          (:user::migrate (:wat::io::read-file path)))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
