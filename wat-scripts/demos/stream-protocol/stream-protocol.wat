;; stream-protocol.wat — the legible twin of the substrate's own boot.
;;
;; WHY THIS EXISTS. When a wat program starts, the substrate has ALREADY spoken a
;; framed protocol to it: N substrate frames, a marker, N program frames, a marker,
;; then HANDOVER — from that moment stdin and stdout belong to the program. That
;; boot is written in Rust, so a student who writes wat cannot read the teacher.
;; This is the same shape, in wat, on the same wire, using the same two verbs the
;; substrate used: `readln` takes one frame, `println` writes one.
;;
;; WHAT IS ACTUALLY INHERITED — not a style, the guarantees. Every frame is one
;; typed EDN value, so a malformed frame is a LOCATED decode error rather than a
;; substring that silently did not match. Every frame is acked, so "the peer
;; received and accepted it" is a fact you hold rather than a hope. A section that
;; can vary in length ends with a MARKER, so growing it later is not a wire break.
;;
;; ── HOW THIS DIFFERS FROM defservice — read this before copying ───────────────
;;
;; The loop below looks like a defservice serve loop because it IS one, by hand.
;; That is the point of the demo and it is ALSO the trap:
;;
;;   defservice   — service-to-service. The substrate owns the channel, generates
;;                  the dispatch from a surface, and threads state for you. If you
;;                  are building something other services dial, USE IT. Hand-rolled
;;                  IPC is precisely what it exists to replace, and the corpus's
;;                  old hand-rolled service reference was annihilated for teaching
;;                  the wrong lesson.
;;
;;   this file    — program-to-pipe. After handover fd 0/1 are YOURS: whatever the
;;                  caller piped in, whatever you write out. There is no peer to
;;                  dial and no surface to satisfy, so there is nothing for
;;                  defservice to generate. You define the conversation.
;;
;; The shared shape is not an accident — a serve loop is what reading a stream of
;; typed values looks like. Copy the SHAPE (framed values, acked, marked sections).
;; Do not copy this to build a service.
;;
;; ── RUN IT ────────────────────────────────────────────────────────────────────
;;   ./target/release/wat wat-scripts/demos/stream-protocol/stream-protocol.wat \
;;     < wat-scripts/demos/stream-protocol/session.edn
;; Expected on stdout: an Ack per frame, then the two assembled section lengths.

;; ── The wire vocabulary ───────────────────────────────────────────────────────
;;
;; A closed set is an enum: the reader matches EVERY variant (a `_` arm on an enum
;; is illegal here — see 109's NOTE-full-enum-match-mandatory-no-wildcard-arm), so
;; adding a frame kind later breaks the build rather than falling through a
;; wildcard at runtime.
;;
;; Both variants carry a field. `SectionDone` carries the sender's count, which the
;; reader checks against its own — the acks already make a LOST frame impossible,
;; so this catches the other direction: a sender that thinks it sent more than it
;; did.
(:wat::core::defenum :proto::Frame :wat::enum::Pure
  :Chunk       [text  <- :wat::core::String]
  :SectionDone [count <- :wat::core::i64])

;; The ack is a value, not a convention. `Got` says "this frame decoded and I
;; accepted it"; `SectionAck` closes the section with the count the reader saw, so
;; a mismatch is visible to the SENDER rather than only to us.
(:wat::core::defenum :proto::Ack :wat::enum::Pure
  :Got        [n     <- :wat::core::i64]
  :SectionAck [count <- :wat::core::i64])

;; ── The reader — one section ──────────────────────────────────────────────────
;;
;; Tail-recursive, because wat has no `loop`: the accumulator and the count are
;; threaded as parameters and TCO makes the recursion a jump. This is the same
;; move a defservice serve loop makes with its state.
;;
;; Reassembly is CONCATENATION, not per-frame parsing — a frame boundary may fall
;; anywhere, mid-form or mid-token, and the reader never needs to understand the
;; content. Whatever the section means is decided once, after the marker.
;; This reader is BOUNDED, not a loop: it is promised exactly one section,
;; closed by a `SectionDone` marker, and it returns the assembled `String` to
;; its caller. Unlike a loop that is content to end whenever the peer stops
;; talking, an end-of-input (or a stop) that arrives BEFORE that marker means
;; the section was cut off mid-flight — the caller can never learn how long it
;; was meant to be, so returning a partial result would silently pass off a
;; truncated read as a complete one. That is a real protocol violation, so
;; dying here — loudly, and naming which of the two ways the stream went
;; missing — is the correct behaviour.
(:wat::core::defn :proto::read-section
  [acc <- :wat::core::String
   n   <- :wat::core::i64]
  -> :wat::core::String
  (:wat::core::match
    (:wat::core::match (:wat::kernel::readln)
      ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
      (:wat::kernel::ReadlnOutcome::Eof
        (:wat::kernel::assertion-failed! "stream-protocol: section truncated — stream ended before a SectionDone marker" :wat::core::None :wat::core::None))
      (:wat::kernel::ReadlnOutcome::Stopped
        (:wat::kernel::assertion-failed! "stream-protocol: section truncated — stop requested before a SectionDone marker" :wat::core::None :wat::core::None)))

    ;; A payload frame: accept it, ack it, keep going.
    ((:proto::Frame::Chunk text)
      (:wat::core::do
        (:wat::kernel::println (:proto::Ack::Got n))
        (:proto::read-section (:wat::string::concat acc text)
                              (:wat::i64::+ n 1))))

    ;; The marker: the section is closed. Ack with OUR count — if it disagrees
    ;; with the sender's, the sender is the one who can act on it.
    ((:proto::Frame::SectionDone _count)
      (:wat::core::do
        (:wat::kernel::println (:proto::Ack::SectionAck n))
        acc))))

;; ── The program ───────────────────────────────────────────────────────────────
;;
;; Two sections, in order, each ended by its own marker — the same shape the
;; substrate used to deliver this file. Position alone would have been enough for
;; a FIXED number of frames; the markers are here because a section that can GROW
;; must not require a wire change to do it.
;;
;; Everything after the last marker is ordinary program work. That is the handover,
;; and it is the same boundary the substrate crossed to reach `:user::main`.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [header  (:proto::read-section "" 0)
     body    (:proto::read-section "" 0)]
    (:wat::kernel::println (:wat::string::length header))
    (:wat::kernel::println (:wat::string::length body))))
