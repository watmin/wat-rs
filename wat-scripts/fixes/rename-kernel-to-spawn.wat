;; wat-scripts/fixes/rename-kernel-to-spawn.wat — the arc-269 spawn-coherence move,
;; run over real wat source files IN WAT, through the wat CLI. The migration tool,
;; self-hosted: no Rust harness, no hand-edit of wat source (use-the-tool, not hand-fix).
;;
;; Relocates the three spawn-layer types misfiled in the kernel junk-drawer
;; (intueri: kernel fidelity 4/10) to their honest home `:wat::spawn::`:
;;   :wat::kernel::Bound        -> :wat::spawn::Bound        (+ /listener /address accessors)
;;   :wat::kernel::Spawned      -> :wat::spawn::Spawned      (the derive marker)
;;   :wat::kernel::ServiceEvent -> :wat::spawn::ServiceEvent (+ ::Shutdown/::Connection/… variants)
;;
;; THREE separate full-name PREFIX renames, NOT one `kernel::` blanket: kernel:: is a
;; shared prefix (Thread'/Process'/Peer'/Listener'/… stay put), so the prefix is the
;; full name — which still catches the accessor (`/listener`) and variant (`::Shutdown`)
;; suffixes that share it. The three prefixes are disjoint, so order is irrelevant.
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["wat/spawn.wat" "wat/service.wat"]\n' | cargo wat ./wat-scripts/fixes/rename-kernel-to-spawn.wat
;;
;; The rewrite is comment-faithful and idempotent (re-running yields zero changes — the
;; old prefix is gone), so it is safe to run over a clean tree.

(:wat::core::defn :user::migrate
  [src <- :wat::core::String] -> :wat::core::String
  (:wat::fix::rename-keyword-prefix ":wat::kernel::ServiceEvent" ":wat::spawn::ServiceEvent"
    (:wat::fix::rename-keyword-prefix ":wat::kernel::Spawned" ":wat::spawn::Spawned"
      (:wat::fix::rename-keyword-prefix ":wat::kernel::Bound" ":wat::spawn::Bound"
        src))))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path
          (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::core::string::concat "[renamed] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
