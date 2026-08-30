;; wat-scripts/scratch-pad/255-stone-layer-2-vector-codec-roundtrip.wat
;; arc 255 Stone layer-2 — row 4 evidence (BYTE-IDENTICAL ROUND TRIP) for the
;; atom.rs -> src/holon/codec.rs split of `:wat::holon::vector-bytes` /
;; `:wat::holon::bytes-vector`. Exercises the real registered intrinsics
;; end-to-end (not just the pure codec functions) so the delegate wiring
;; itself — require_vector, program_dim, the outcome-constructor mapping —
;; is proven, not just the extracted algorithm.
;;
;; Three things proven, in order:
;;   1. round trip: encode -> vector-bytes -> bytes-vector -> Decoded, and
;;      the decoded Vector `=` the original, AND re-encoding it reproduces
;;      byte-identical bytes.
;;   2. failure path 1 — TruncatedHeader: 3 raw bytes (< the 4-byte header).
;;   3. failure path 2 — LengthMismatch: a well-formed dim=8 header (needs 2
;;      data bytes) followed by only 1.

(:wat::core::defn :probe::run [] -> :wat::core::nil
  (:wat::core::let
    [v (:wat::holon::encode (:wat::holon::to-holon "layer-2-roundtrip-atom"))
     bs (:wat::holon::vector-bytes v)
     decoded (:wat::holon::bytes-vector bs)]
    (:wat::core::do
      (:wat::kernel::println "── round trip ──")
      (:wat::core::match decoded
        ((:wat::holon::VectorDecodeOutcome::Decoded v2)
          (:wat::core::let
            [bs2 (:wat::holon::vector-bytes v2)]
            (:wat::core::do
              (:wat::kernel::println "bytes-len:")
              (:wat::kernel::println (:wat::core::count bs))
              (:wat::kernel::println "decoded-vector-equal-original:")
              (:wat::kernel::println (:wat::core::= v v2))
              (:wat::kernel::println "re-encoded-bytes-equal-original:")
              (:wat::kernel::println (:wat::core::= bs bs2)))))
        (_
          (:wat::core::do
            (:wat::kernel::println "UNEXPECTED:")
            (:wat::kernel::println decoded))))

      (:wat::kernel::println "── failure path 1: TruncatedHeader (3 bytes) ──")
      (:wat::core::let
        [outcome1
          (:wat::holon::bytes-vector
            (:wat::core::Vector :- [:wat::core::u8] (:wat::core::u8 1) (:wat::core::u8 2) (:wat::core::u8 3)))]
        (:wat::core::match outcome1
          ((:wat::holon::VectorDecodeOutcome::TruncatedHeader got)
            (:wat::core::do
              (:wat::kernel::println "TruncatedHeader got:")
              (:wat::kernel::println got)))
          (_
            (:wat::core::do
              (:wat::kernel::println "UNEXPECTED:")
              (:wat::kernel::println outcome1)))))

      (:wat::kernel::println "── failure path 2: LengthMismatch (dim=8 header, 1 data byte instead of 2) ──")
      (:wat::core::let
        [outcome2
          (:wat::holon::bytes-vector
            (:wat::core::Vector :- [:wat::core::u8]
              (:wat::core::u8 8) (:wat::core::u8 0) (:wat::core::u8 0) (:wat::core::u8 0)
              (:wat::core::u8 0)))]
        (:wat::core::match outcome2
          ((:wat::holon::VectorDecodeOutcome::LengthMismatch expected got)
            (:wat::core::do
              (:wat::kernel::println "LengthMismatch expected:")
              (:wat::kernel::println expected)
              (:wat::kernel::println "LengthMismatch got:")
              (:wat::kernel::println got)))
          (_
            (:wat::core::do
              (:wat::kernel::println "UNEXPECTED:")
              (:wat::kernel::println outcome2))))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:probe::run))
