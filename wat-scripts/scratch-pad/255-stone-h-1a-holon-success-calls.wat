;; wat-scripts/scratch-pad/255-stone-h-1a-holon-success-calls.wat — arc 255
;; Stone H-1a. row 1's SUCCESS half: exercise every constructible one of the
;; 35 holon verbs at its real arity and print a result, so before/after can
;; be diffed byte-for-byte. Four `:wat::holon::Engram/*` verbs (`name`,
;; `eigenvalue-signature`, `n`, `residual`) are documented UNREACHABLE FROM
;; WAT TODAY (no constructor anywhere hands a wat program a bare
;; `Value::Engram` — see `engram.rs`'s doc comments) — they are exercised in
;; the sibling wrong-arity script instead, not here. Scratch, per
;; holon/CLAUDE.md's .wat scratch convention.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::core::let
      [s (:wat::holon::OnlineSubspace/new 10000 8)
       v (:wat::holon::encode (:wat::holon::to-holon "x"))]
      (:wat::core::do
        (:wat::kernel::println "── :wat::holon::OnlineSubspace/new ──")
        (:wat::kernel::pprintln s)
        (:wat::kernel::println "── :wat::holon::OnlineSubspace/dim ──")
        (:wat::kernel::pprintln (:wat::holon::OnlineSubspace/dim s))
        (:wat::kernel::println "── :wat::holon::OnlineSubspace/k ──")
        (:wat::kernel::pprintln (:wat::holon::OnlineSubspace/k s))
        (:wat::kernel::println "── :wat::holon::OnlineSubspace/n ──")
        (:wat::kernel::pprintln (:wat::holon::OnlineSubspace/n s))
        (:wat::kernel::println "── :wat::holon::OnlineSubspace/threshold ──")
        (:wat::kernel::pprintln (:wat::holon::OnlineSubspace/threshold s))
        (:wat::kernel::println "── :wat::holon::OnlineSubspace/eigenvalues ──")
        (:wat::kernel::pprintln (:wat::core::length (:wat::holon::OnlineSubspace/eigenvalues s)))
        (:wat::kernel::println "── :wat::holon::OnlineSubspace/update ──")
        (:wat::kernel::pprintln (:wat::holon::OnlineSubspace/update s v))
        (:wat::kernel::println "── :wat::holon::OnlineSubspace/residual ──")
        (:wat::kernel::pprintln (:wat::holon::OnlineSubspace/residual s v))
        (:wat::kernel::println "── :wat::holon::OnlineSubspace/project ──")
        (:wat::kernel::pprintln (:wat::core::length (:wat::holon::OnlineSubspace/project s v)))
        (:wat::kernel::println "── :wat::holon::OnlineSubspace/reconstruct ──")
        (:wat::kernel::pprintln (:wat::core::length (:wat::holon::OnlineSubspace/reconstruct s v)))))

    (:wat::core::let
      [lib (:wat::holon::EngramLibrary/new 10000)
       sub (:wat::holon::OnlineSubspace/new 10000 4)
       v   (:wat::holon::encode (:wat::holon::to-holon "x"))
       r0  (:wat::holon::OnlineSubspace/update sub v)
       u   (:wat::holon::EngramLibrary/add lib "pattern-a" sub)]
      (:wat::core::do
        (:wat::kernel::println "── :wat::holon::EngramLibrary/new ──")
        (:wat::kernel::pprintln lib)
        (:wat::kernel::println "── :wat::holon::EngramLibrary/add ──")
        (:wat::kernel::pprintln u)
        (:wat::kernel::println "── :wat::holon::EngramLibrary/len ──")
        (:wat::kernel::pprintln (:wat::holon::EngramLibrary/len lib))
        (:wat::kernel::println "── :wat::holon::EngramLibrary/contains ──")
        (:wat::kernel::pprintln (:wat::holon::EngramLibrary/contains lib "pattern-a"))
        (:wat::kernel::println "── :wat::holon::EngramLibrary/names ──")
        (:wat::kernel::pprintln (:wat::holon::EngramLibrary/names lib))
        (:wat::kernel::println "── :wat::holon::EngramLibrary/match-vec ──")
        (:wat::kernel::pprintln (:wat::core::length (:wat::holon::EngramLibrary/match-vec lib v 5 5)))))

    (:wat::core::let
      [labels
        (:wat::core::Vector :wat::holon::HolonAST
          (:wat::holon::to-holon "up")
          (:wat::holon::to-holon "down"))
       r (:wat::holon::Reckoner/new-discrete "test-rec" 10000 1 labels)
       v (:wat::holon::encode (:wat::holon::to-holon "x"))
       u1 (:wat::holon::Reckoner/observe r v 0 1.0)
       u2 (:wat::holon::Reckoner/observe r v 1 1.0)
       pred (:wat::holon::Reckoner/predict r v)
       conviction (:wat::core::third pred)
       resolved (:wat::holon::Reckoner/resolve r conviction true)
       curve (:wat::holon::Reckoner/curve r)
       rc (:wat::holon::Reckoner/new-continuous "cont" 10000 100 0.0 16)]
      (:wat::core::do
        (:wat::kernel::println "── :wat::holon::Reckoner/new-discrete ──")
        (:wat::kernel::pprintln r)
        (:wat::kernel::println "── :wat::holon::Reckoner/observe ──")
        (:wat::kernel::pprintln u1)
        (:wat::kernel::println "── :wat::holon::Reckoner/predict ──")
        (:wat::kernel::pprintln conviction)
        (:wat::kernel::println "── :wat::holon::Reckoner/resolve ──")
        (:wat::kernel::pprintln resolved)
        (:wat::kernel::println "── :wat::holon::Reckoner/curve ──")
        (:wat::kernel::pprintln curve)
        (:wat::kernel::println "── :wat::holon::Reckoner/labels ──")
        (:wat::kernel::pprintln (:wat::core::length (:wat::holon::Reckoner/labels r)))
        (:wat::kernel::println "── :wat::holon::Reckoner/dims ──")
        (:wat::kernel::pprintln (:wat::holon::Reckoner/dims r))
        (:wat::kernel::println "── :wat::holon::Reckoner/new-continuous ──")
        (:wat::kernel::pprintln (:wat::holon::Reckoner/dims rc))))

    (:wat::core::let
      [store (:wat::holon::Hologram/make (:wat::holon::filter-accept-any))
       k (:wat::holon::leaf :alpha)
       val (:wat::holon::leaf :beta)
       putres (:wat::holon::Hologram/put store k val)]
      (:wat::core::do
        (:wat::kernel::println "── :wat::holon::Hologram/make ──")
        (:wat::kernel::pprintln store)
        (:wat::kernel::println "── :wat::holon::Hologram/put ──")
        (:wat::kernel::pprintln putres)
        (:wat::kernel::println "── :wat::holon::Hologram/len ──")
        (:wat::kernel::pprintln (:wat::holon::Hologram/len store))
        (:wat::kernel::println "── :wat::holon::Hologram/capacity ──")
        (:wat::kernel::pprintln (:wat::holon::Hologram/capacity store))
        (:wat::kernel::println "── :wat::holon::Hologram/get ──")
        (:wat::kernel::pprintln (:wat::holon::Hologram/get store k))
        (:wat::kernel::println "── :wat::holon::Hologram/find ──")
        (:wat::kernel::pprintln (:wat::holon::Hologram/find store k))
        (:wat::kernel::println "── :wat::holon::Hologram/remove ──")
        (:wat::kernel::pprintln (:wat::holon::Hologram/remove store k))))))
