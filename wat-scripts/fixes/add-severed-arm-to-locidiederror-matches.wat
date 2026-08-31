;; wat-scripts/fixes/add-severed-arm-to-locidiederror-matches.wat
;;
;; Self-hosted, comment-faithful fix-wat codemod — NO hand-editing of .wat files, use the tool.
;;
;; THE MIGRATION: `:wat::kernel::LociDiedError` gains a `Severed` unit variant (the service's owner
;; released its handle, so its serve loop exited — distinct from `Disconnected`, which is the
;; catch-all "it died, you are not told why"). Every EXHAUSTIVE match over the enum therefore needs
;; one more arm, and the checker names each site to the byte:
;;
;;   non-exhaustive: enum :wat::kernel::LociDiedError missing arm(s) for variant(s): Severed
;;
;; Census at the time of writing — FOUR files, and the floor's four RED arms are exactly these
;; (the census and the floor agree, which is why neither is being trusted alone):
;;   tests/comms/probe_arc209_structured_peer_death.wat
;;   tests/comms/wat_arc113_raise_round_trip.wat
;;   tests/diagnostics/probe_runtime_error_produces_structured_edn.wat
;;   tests/diagnostics/probe_plain_panic_produces_structured_edn.wat
;;
;; All four carry the SAME shape: a `WRONG:<Variant>` negative oracle, one arm per variant, so a
;; wrong-arm RED names exactly which death arrived instead of just failing. The new arm must join
;; that oracle rather than be swallowed by a `_` wildcard — a wildcard would make every future
;; variant silently pass these tests, which is the opposite of what they exist to do.
;;
;; So the arm inserted is the `Stopped` arm COPIED with its label renamed, which is why one pass
;; handles both payload shapes present in the corpus:
;;   (1) `(… ::Stopped "WRONG:Stopped")`                     -> + `(… ::Severed "WRONG:Severed")`
;;   (2) `(… ::Stopped (:wat::core::Some "WRONG:Stopped"))`  -> + the `Some`-wrapped twin
;; Neither pattern is a substring of the other (their payloads differ), so the two passes are
;; disjoint and order between them does not matter.
;;
;; IDEMPOTENCY IS BY GUARD, NOT BY CONSTRUCTION — and that is the one way this codemod differs from
;; its precedent, `rename-locidiederror-shutdown-to-stopped.wat`. A RENAME is self-limiting: after it
;; runs, the old token is gone and a re-run finds nothing. An INSERTION is not: the anchor it keys on
;; (`::Stopped`) survives the edit, so a second run would append a SECOND `Severed` arm and the
;; checker would then reject the file for a duplicate arm. `migrate` therefore returns `src`
;; untouched the moment the text already names `LociDiedError::Severed`.
;;
;; Usage (one EDN vector of paths on stdin — list EVERY path):
;;   printf '["pathA" "pathB" …]\n' \
;;     | ./target/release/wat ./wat-scripts/fixes/add-severed-arm-to-locidiederror-matches.wat

;; literal-replace — substring replace via split+join (no dedicated string::replace primitive exists
;; in wat core). Transcribed from the precedent codemod named above.
(:wat::core::defn :user::literal-replace
  [src <- :wat::core::String  old <- :wat::core::String  new <- :wat::core::String] -> :wat::core::String
  (:wat::string::join new (:wat::string::split src old)))

(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  ;; the guard: already migrated -> return untouched (see IDEMPOTENCY above).
  (:wat::core::if (:wat::string::contains? src "LociDiedError::Severed")
    src
    (:wat::core::let
      [bare   "          (:wat::kernel::LociDiedError::Stopped \"WRONG:Stopped\")"
       bare'  (:wat::string::concat bare
                "\n          (:wat::kernel::LociDiedError::Severed \"WRONG:Severed\")")
       somed  "          (:wat::kernel::LociDiedError::Stopped (:wat::core::Some \"WRONG:Stopped\"))"
       somed' (:wat::string::concat somed
                "\n          (:wat::kernel::LociDiedError::Severed (:wat::core::Some \"WRONG:Severed\"))")
       src1   (:user::literal-replace src somed somed')
       src2   (:user::literal-replace src1 bare bare')]
      src2)))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path
          (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[+severed-arm] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
