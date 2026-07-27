//! The boot wire — how a spawned child receives its program.
//!
//! # Why this exists
//!
//! Today a forked child gets its program for free: `spawn_process_peer` hands
//! `run_forms_as_server_child` a `Vec<WatAST>` that the child can simply read,
//! because fork gave it the parent's address space. `execve` ends that — it
//! replaces the address space, so the program must arrive as **bytes on a wire**.
//!
//! This module is that wire. It ships BEFORE the exec (arc 170 step 2) so the
//! stream path is proven while the closure is still there as the control; when the
//! exec lands, exactly one variable changes.
//!
//! # The shape
//!
//! ```text
//!    substrate frames 0..N        the substrate's — Config lives here
//!    marker  SubstrateDone
//!    program frames 0..M          the user's — reassembled by concatenation
//!    marker  ProgramDone
//!    ── HANDOVER ──               the stream is the program's
//! ```
//!
//! # Mini-TCP — and why the read boundary is safe
//!
//! Every frame is acked, and **the parent blocks on the ack before writing the
//! next frame**. That is the substrate's canonical mutex-replacement pattern,
//! named in arc 089 and documented at `docs/ZERO-MUTEX.md:252`: *"bounded(1) on
//! the request pipe means a producer can't enqueue another message while the
//! previous one is in-flight."*
//!
//! It is also what makes a chunked read correct here. A pipe has no peek and no
//! pushback, so a chunked read over-reads by design — if a marker lands at byte
//! 100 of a 64 KiB read, the rest is consumed and cannot be returned. Reading one
//! byte at a time is the only way to stop exactly at a delimiter, and it was
//! measured at **91.91 ms per 512 KiB frame against 0.41 ms chunked** (224x) —
//! unaffordable next to exec's ~170 ms.
//!
//! Under mini-TCP there is nothing to over-read: the parent has not written past
//! the marker, because it is blocked on that marker's ack. The class is not made
//! recoverable, it is made **unproducible**.
//!
//! **The invariant, stated once: the parent MUST NOT write past a marker until
//! that marker is acked.** Pipelining the first message behind `ProgramDone` for
//! latency silently re-opens the class.
//!
//! # Why the ack is not redundant with the kernel
//!
//! The kernel's pipe backpressure tells the parent *bytes were accepted*. An ack
//! tells it *the child received, decoded, and accepted them*. Different facts —
//! and for a startup handshake the second is the one that matters, because the
//! failure this arc exists to kill is "the child died during startup and the
//! parent proceeds not knowing." Acks also make the death **locatable**: dying on
//! a substrate frame is a bad handoff from the parent; on a program frame, a bad
//! program. Without them both read as *the child went away*.
//!
//! # The frames are EDN, read and written only by Rust
//!
//! Boot happens before any world exists in the child, so these types are never
//! registered as wat types and no wat program ever sees them. They ride the
//! existing `Sender<String>` / `Receiver<String>` process wire as EDN text, which
//! is already newline-framed (`comms/process.rs` — `edn_bytes + b'\n'`).

use crate::runtime::{RuntimeError, RuntimeErrorKind};
use crate::span::Span;

/// The per-frame transport budget. Mirrors `edn_shim::DEFAULT_MAX_FRAME_BYTES`.
///
/// This is a limit on ONE FRAME, never on the program: a large program is chunked
/// across many program frames. The chunker below is what guarantees no single
/// frame exceeds it.
pub(crate) const MAX_BOOT_FRAME_BYTES: usize = crate::edn_shim::DEFAULT_MAX_FRAME_BYTES;

/// A frame on the boot wire.
///
/// Markers are typed values, not magic strings — an unknown or malformed marker is
/// a located decode failure rather than a string comparison that quietly does not
/// match. They carry no count: every frame is acked, so a lost frame cannot go
/// unnoticed, and a count would be a second mechanism for a guarantee the
/// handshake already gives.
#[derive(Debug, Clone, PartialEq, wat_edn::Edn)]
#[to_edn(namespace = BOOT_NS)]
pub(crate) enum BootFrame {
    /// A slice of the current section's payload. Sections are reassembled by
    /// CONCATENATION, so a chunk boundary may fall anywhere — mid-form,
    /// mid-token, mid-string — and the reader never parses content.
    Chunk { text: String },
    /// The substrate section is complete.
    SubstrateDone,
    /// The program section is complete. The next thing on this wire belongs to
    /// the program.
    ProgramDone,
}

/// The child's reply to one frame.
///
/// An enum rather than a bare acknowledgement, because a reply is an OUTCOME: the
/// child may later need to say *why* it will not take a frame, and a `Refused`
/// variant joins here without a wire change. A parent that gets no ack today
/// learns only that the child stopped talking; a parent that gets a refusal
/// learns which frame and why.
#[derive(Debug, Clone, Copy, PartialEq, wat_edn::Edn)]
#[to_edn(namespace = BOOT_NS)]
pub(crate) enum BootReply {
    /// Received, decoded, accepted. Send the next frame.
    Ack,
}

/// The wire namespace every boot tag lives under.
///
/// `#[derive(Edn)]` emits the write impl AND submits an `EdnSchema` into the
/// link-time inventory that `register_builtin_types` drains — so these frames are
/// REGISTERED wat types, readable by `edn::read`, not a private Rust dialect. That
/// is deliberate: a boot frame a wat program cannot name is a stringly holdout of
/// exactly the kind arc 296 exists to delete, and the handshake is testable from
/// wat because of it.
pub(crate) const BOOT_NS: &str = "wat.boot";
const TAG_CHUNK: &str = "Chunk";
const TAG_SUBSTRATE_DONE: &str = "SubstrateDone";
const TAG_PROGRAM_DONE: &str = "ProgramDone";
const TAG_ACK: &str = "Ack";

fn boot_err(reason: String) -> RuntimeError {
    RuntimeError {
        span: crate::rust_caller_span!(),
        kind: RuntimeErrorKind::MalformedForm {
            head: ":wat::process::boot".into(),
            reason,
        },
    }
}

impl BootFrame {
    /// Encode to the EDN line this frame rides as.
    ///
    /// The shape comes from `#[derive(Edn)]` — there is no hand-written encoder to
    /// drift from the type. This is only the bridge to `comms`'s string wire.
    pub(crate) fn to_wire(&self) -> String {
        wat_edn::write(&wat_edn::ToEdn::to_edn(self))
    }

    /// Decode one frame.
    ///
    /// **This reader is hand-written, and that asymmetry is deliberate.** The
    /// derive registers an `EdnSchema` so `edn::read` can reconstruct these types
    /// — but `reconstruct_record` needs a `TypeEnv`, and the child has no world
    /// when it reads these frames; the frames are what BUILD the world. So the
    /// boot decoder cannot use the registry it populates.
    ///
    /// Kept small and total for that reason: three tags, structural comparison,
    /// and an unknown tag is a LOCATED refusal rather than a guess. A child must
    /// never improvise about a frame it does not recognise.
    pub(crate) fn from_wire(line: &str) -> Result<BootFrame, RuntimeError> {
        let parsed = wat_edn::parse_owned(line.trim())
            .map_err(|e| boot_err(format!("boot frame is not EDN: {e}")))?;
        // Compare the tag STRUCTURALLY (namespace + name), never through a
        // rendered string — a Display form is a second encoding to get wrong, and
        // this decoder is the thing standing between a child and a program it
        // does not understand.
        let (tag, body) = match &parsed {
            wat_edn::OwnedValue::Tagged(tag, body) => (tag, body.as_ref()),
            other => {
                return Err(boot_err(format!(
                    "boot frame must be a tagged value; got {other:?}"
                )))
            }
        };
        if tag.namespace() != BOOT_NS {
            return Err(boot_err(format!(
                "boot frame tag is not in the {BOOT_NS} namespace: #{}/{}",
                tag.namespace(),
                tag.name()
            )));
        }
        match tag.name() {
            TAG_CHUNK => {
                let text = named_string_field(body, "text")?;
                Ok(BootFrame::Chunk { text })
            }
            TAG_SUBSTRATE_DONE => Ok(BootFrame::SubstrateDone),
            TAG_PROGRAM_DONE => Ok(BootFrame::ProgramDone),
            other => Err(boot_err(format!(
                "unknown boot frame tag #{BOOT_NS}/{other} — the child refuses to guess"
            ))),
        }
    }
}

/// Pull one named String field out of a derived variant's map body.
fn named_string_field(body: &wat_edn::OwnedValue, key: &str) -> Result<String, RuntimeError> {
    let entries = match body {
        wat_edn::OwnedValue::Map(entries) => entries,
        other => {
            return Err(boot_err(format!(
                "expected a map body carrying :{key}; got {other:?}"
            )))
        }
    };
    for (k, v) in entries {
        let matches = matches!(k, wat_edn::OwnedValue::Keyword(kw) if kw.name() == key);
        if matches {
            return match v {
                wat_edn::OwnedValue::String(s) => Ok(s.to_string()),
                other => Err(boot_err(format!(
                    ":{key} must be a String; got {other:?}"
                ))),
            };
        }
    }
    Err(boot_err(format!("boot frame body is missing :{key}")))
}

impl BootReply {
    pub(crate) fn to_wire(&self) -> String {
        wat_edn::write(&wat_edn::ToEdn::to_edn(self))
    }

    /// Decode the child's reply. Anything that is not an `Ack` means the child did
    /// NOT accept the frame — the parent must never read silence or a surprise as
    /// success.
    pub(crate) fn from_wire(line: &str) -> Result<BootReply, RuntimeError> {
        let parsed = wat_edn::parse_owned(line.trim())
            .map_err(|e| boot_err(format!("boot reply is not EDN: {e}")))?;
        match &parsed {
            wat_edn::OwnedValue::Tagged(tag, _)
                if tag.namespace() == BOOT_NS && tag.name() == TAG_ACK =>
            {
                Ok(BootReply::Ack)
            }
            other => Err(boot_err(format!(
                "expected #{BOOT_NS}/{TAG_ACK}; got {other:?} — the child did not accept the frame"
            ))),
        }
    }
}

/// Split `payload` into `Chunk` frames, none exceeding the per-frame budget.
///
/// The split is on BYTES, not on form boundaries — reassembly is concatenation, so
/// a boundary may fall anywhere. It respects UTF-8 character boundaries only
/// because a `String` cannot be split mid-codepoint; that is a property of the
/// type, not a guarantee this wire needs.
///
/// Returns a located error rather than a silent truncation if a single chunk would
/// somehow exceed the budget — that would be a bug in this function, and STOP-2
/// says it must be loud.
pub(crate) fn chunk_payload(payload: &str) -> Result<Vec<BootFrame>, RuntimeError> {
    // The budget applies to the ENCODED frame, which is larger than the payload
    // (the tag, the brackets, and EDN string escaping). Leave headroom rather than
    // encode-and-hope: escaping can in the worst case double a payload's size.
    let slice_budget = MAX_BOOT_FRAME_BYTES / 2;

    let mut frames = Vec::new();
    let mut rest = payload;
    while !rest.is_empty() {
        let mut cut = slice_budget.min(rest.len());
        // Do not split a codepoint.
        while cut > 0 && !rest.is_char_boundary(cut) {
            cut -= 1;
        }
        if cut == 0 {
            return Err(boot_err(
                "boot chunker: a single codepoint exceeds the frame budget".into(),
            ));
        }
        let (head, tail) = rest.split_at(cut);
        let frame = BootFrame::Chunk { text: head.to_string() };
        let encoded_len = frame.to_wire().len();
        if encoded_len > MAX_BOOT_FRAME_BYTES {
            return Err(boot_err(format!(
                "boot chunker produced a {encoded_len}-byte frame, over the \
                 {MAX_BOOT_FRAME_BYTES}-byte budget — this is a chunker bug, not a caller error"
            )));
        }
        frames.push(frame);
        rest = tail;
    }
    Ok(frames)
}

#[allow(dead_code)]
fn _span_type_is_used(_: &Span) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_round_trip_through_the_wire() {
        for f in [
            BootFrame::Chunk { text: "(:user::main)".into() },
            BootFrame::Chunk { text: "with \"quotes\" and\nnewlines".into() },
            BootFrame::SubstrateDone,
            BootFrame::ProgramDone,
        ] {
            let wire = f.to_wire();
            assert!(
                !wire.contains('\n'),
                "an encoded frame must carry no RAW newline — the wire is \
                 newline-framed, so a raw newline would split one frame into two: {wire:?}"
            );
            assert_eq!(BootFrame::from_wire(&wire).expect("decode"), f);
        }
    }

    #[test]
    fn ack_round_trips() {
        assert_eq!(BootReply::from_wire(&BootReply::Ack.to_wire()).expect("decode"), BootReply::Ack);
    }

    #[test]
    fn an_unknown_tag_is_refused_not_guessed() {
        let err = BootFrame::from_wire("#wat.boot/Sideways []").unwrap_err();
        let msg = format!("{err:?}");
        assert!(
            msg.contains("unknown boot frame tag"),
            "an unrecognised frame must be a located refusal; got {msg}"
        );
    }

    #[test]
    fn an_ack_that_is_not_an_ack_is_refused() {
        // The child answering a frame with anything else means it did not accept
        // it — the parent must not read that as success.
        assert!(BootReply::from_wire("#wat.boot/ProgramDone {}").is_err());
    }

    #[test]
    fn chunking_never_exceeds_the_frame_budget() {
        // A payload well past one frame, with multibyte characters at the split
        // points to prove the codepoint guard holds.
        let payload: String = "λ(:demo::x 1)".repeat(200_000);
        let frames = chunk_payload(&payload).expect("chunk");
        assert!(frames.len() > 1, "a large payload must span frames");
        for f in &frames {
            assert!(
                f.to_wire().len() <= MAX_BOOT_FRAME_BYTES,
                "every encoded frame must fit the budget"
            );
        }
        // Reassembly is concatenation — the whole point.
        let rejoined: String = frames
            .iter()
            .map(|f| match f {
                BootFrame::Chunk { text } => text.as_str(),
                _ => panic!("chunk_payload emits only Chunks"),
            })
            .collect();
        assert_eq!(rejoined, payload, "concatenation must reproduce the payload exactly");
    }

    #[test]
    fn an_empty_payload_produces_no_frames() {
        assert!(chunk_payload("").expect("chunk").is_empty());
    }
}
