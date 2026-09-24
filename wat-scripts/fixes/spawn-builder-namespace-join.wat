;; wat-scripts/fixes/spawn-builder-namespace-join.wat — arc 255 Stone 255.14,
;; "a namespace is not a type".
;; SCOPE: corpus
;; Self-hosted fix-wat codemod: no hand-editing of .wat files — use the tool.
;;
;; ⭐ THE NAMES DO NOT CHANGE. THIS IS NOT A RENAME.
;; `wat.spawn.process/post-spawn` is already correct faithful Clojure — namespace
;; `wat.spawn.process`, name `post-spawn`. Only the RUST-SCHEME DECLARATION was wrong,
;; because it spelled a NAMESPACE (`process`, `thread`) as if it were a TYPE:
;;
;;     :wat::spawn::process/post-spawn   →   :wat::spawn::process::post-spawn
;;              ^ member join (255.4: a member join is `/`, ALWAYS — and ONLY when the
;;                segment before it is a declared TYPE)
;;
;; The faithful image is BYTE-IDENTICAL either way (`wat.spawn.process/post-spawn`), which
;; is the whole point: 251.8d-ii's EIGHTH draw proved that a `/` join at a NON-TYPE parent
;; is UNSPELLABLE in the faithful surface — `wat.spawn.process/post-spawn` is the image of
;; BOTH `:wat::spawn::process/post-spawn` and `:wat::spawn::process::post-spawn`, and only
;; `freeze::env::rekey_type_member_functions` can tell them apart, which it can do only
;; when the parent names a TYPE. `process` and `thread` are not types; they are the
;; lower-case halves of `wat/spawn.wat`'s documented per-env builder-constructor
;; convention. After this migration the round trip is the identity.
;;
;; SEVEN whole-name renames, one per builder constructor (`wat/spawn.wat` :105 :110 :116
;; :130 :133 :141 :149 and every call site):
;;
;;   :wat::spawn::thread/init               -> :wat::spawn::thread::init
;;   :wat::spawn::thread/post-spawn         -> :wat::spawn::thread::post-spawn
;;   :wat::spawn::thread/runner-count       -> :wat::spawn::thread::runner-count
;;   :wat::spawn::process/post-spawn        -> :wat::spawn::process::post-spawn
;;   :wat::spawn::process/env               -> :wat::spawn::process::env
;;   :wat::spawn::process/max-message-bytes -> :wat::spawn::process::max-message-bytes
;;   :wat::spawn::process/runner-count      -> :wat::spawn::process::runner-count
;;
;; ⭐ `rename-keyword-exact`, NOT `rename-keyword-prefix`: this is a WHOLE-TOKEN rename and
;; must never bleed into a sibling. `:wat::spawn::process` (the no-argument builder, :122)
;; and `:wat::spawn::thread` (:99) keep their names, and the RETIRED heads
;; `:wat::spawn::process/grants` / `:wat::spawn::process/uses` (src/intrinsic/mod.rs's
;; retired-name rows, and `repoint-retired-heads-to-live-spellings.wat`) are NOT in this
;; class and must NOT move. Exact whole-name equality gives that for free, and makes the
;; rewrite IDEMPOTENT BY CONSTRUCTION: after it, no token equals the old name.
;;
;; ⛔ COMMENTS AND PROSE ARE NOT REWRITTEN — `rename-keyword-exact` rides `fix-text-apply`
;; over keyword LEAF spans only. `wat/spawn.wat`'s own header (lines 90–98) documents the
;; convention in prose and is updated by hand in the same commit; `docs/arc/**` is history
;; and is not rewritten at all.
;;
;; ⛔ TWO FILES CARRYING THE NAME ARE DELIBERATELY NOT IN THE PATH LIST:
;;   wat-scripts/fixes/replay/caller-to-emitted-from/before.pre
;;   wat-scripts/fixes/replay/caller-to-emitted-from/after.post
;; They are ANOTHER migration's recorded byte-exact replay fixture (`.pre` → `.post`), the
;; oracle for `every_recorded_migration_replays`. Moving either one falsifies that gate's
;; record. They are text fixtures for a text rewrite and name nothing at runtime.
;;
;; Usage (ONE EDN vector of EVERY path on stdin — list them ALL; a hand-listed subset that
;; misses one call site leaves a name that no longer exists):
;;   printf '["wat/spawn.wat" "tests/..." …]\n' \
;;     | cargo wat ./wat-scripts/fixes/spawn-builder-namespace-join.wat

(:wat::core::defn :user::migrate
  [src <- :wat::core::String] -> :wat::core::String
  (:wat::fix::rename-keyword-exact ":wat::spawn::thread/init" ":wat::spawn::thread::init"
    (:wat::fix::rename-keyword-exact ":wat::spawn::thread/post-spawn" ":wat::spawn::thread::post-spawn"
      (:wat::fix::rename-keyword-exact ":wat::spawn::thread/runner-count" ":wat::spawn::thread::runner-count"
        (:wat::fix::rename-keyword-exact ":wat::spawn::process/post-spawn" ":wat::spawn::process::post-spawn"
          (:wat::fix::rename-keyword-exact ":wat::spawn::process/env" ":wat::spawn::process::env"
            (:wat::fix::rename-keyword-exact ":wat::spawn::process/max-message-bytes" ":wat::spawn::process::max-message-bytes"
              (:wat::fix::rename-keyword-exact ":wat::spawn::process/runner-count" ":wat::spawn::process::runner-count"
                src))))))))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path
          (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[namespace-join] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln ) [:wat::kernel::ReadlnOutcome.Datum {:v __datum} __datum] [:wat::kernel::ReadlnOutcome.Eof {} (:wat::kernel::assertion-failed! :message "readln: end of input")] [:wat::kernel::ReadlnOutcome.Stopped {} (:wat::kernel::assertion-failed! :message "readln: stop requested")])))
