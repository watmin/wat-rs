//! Arc 272 — the capability subsystem (a warded home). Three concerns: what may cross a boundary as a
//! capability, how it serializes (the `wat-edn.cap` **narrow waist**), and — v4 — who may receive it
//! (the comms policy / **powerbox**).
//!
//! - [`registry`] — the frozen WAIST: a [`registry::CapCodec`] registry + the generic
//!   [`encode_capability`] / [`decode_capability`] dispatch over the `wat-edn.cap` wire. Adding a
//!   capability is a registry row; the dispatch never changes — the hourglass / narrow-waist law, run
//!   inward (a rigid core enabling unbounded capabilities above it).
//!
//! The object-capability trust boundary sits AT the waist: a capability reconstructs ONLY off the
//! trusted door (`edn_shim::decode_trusted_wire`) — handed over a lineage channel, never forged from
//! parsed data. `edn_shim` owns the EDN read/write dispatch + that door and DELEGATES here; this home
//! owns the registry + codecs (and, v4, the policy). See `docs/arc/2026/06/272-…/` REALIZATIONS.md for
//! the ocap / narrow-waist / end-to-end synthesis this home embodies.

pub mod registry;

pub use registry::{decode_capability, encode_capability, CapCodec};
