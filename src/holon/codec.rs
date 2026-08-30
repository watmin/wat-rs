//! The vector wire format: `dim:u32-LE ++ packed-cells` (each byte packs up
//! to 4 ternary cells, 2 bits/cell — `0b00`=0, `0b01`=+1, `0b10`=-1, `0b11`
//! reserved/invalid). Pure codec, lifted out of
//! `src/intrinsic/holon/atom.rs`'s `holon_vector_bytes` /
//! `eval_holon_bytes_vector` per Stone layer-2 — see `src/holon/mod.rs` for
//! the two-layer doctrine this module is held to a stricter version of:
//! **no `WatAST`, `Value`, `RuntimeError`, `Span`, `Environment`, or
//! `SymbolTable` anywhere in a signature here.** `Vec<u8>` /
//! `holon::Vector` / plain Rust enums only — the door (`atom.rs`) converts
//! wat values in, calls in here, and adapts the domain outcome back through
//! `src/holon/outcome.rs`'s `vector_decode_outcome_*` constructors.
//!
//! Decoding is split into `parse_vector_header` + `decode_vector_cells`
//! rather than one function, because the door's cross-check of the header's
//! `dim` against the program's ambient dim-count needs the `SymbolTable`
//! (`program_dim`, in `runtime.rs`) — a door concern this module cannot
//! take a dependency on. Splitting at that point lets the door fetch the
//! ambient dim-count only once the header/length checks have already
//! passed, exactly where the pre-split body fetched it — so a call with no
//! `EncodingCtx` attached and a truncated/malformed header still resolves
//! to `TruncatedHeader`/`LengthMismatch` without ever touching the ctx,
//! same as before the split.

/// The outcome of `parse_vector_header`: either a validated `dim` (the
/// header parsed and the byte count matches it), or one of the two
/// structural failures that don't need the program's dim-count to detect.
pub(crate) enum VectorHeader {
    Ok { dim: usize },
    /// Fewer than the 4-byte dim header. No `expected` field — 4 is a
    /// protocol constant, not a per-call datum.
    TruncatedHeader { got: usize },
    /// The header's dim parsed fine, but the data bytes don't match
    /// `ceil(dim/4)`.
    LengthMismatch { expected: usize, got: usize },
}

/// Parse `bytes`' 4-byte little-endian dim header and validate the overall
/// length against it. Does not know or care what dim the program expects —
/// that cross-check is the caller's job (it needs the `SymbolTable`).
pub(crate) fn parse_vector_header(bytes: &[u8]) -> VectorHeader {
    if bytes.len() < 4 {
        return VectorHeader::TruncatedHeader { got: bytes.len() };
    }
    let header = [bytes[0], bytes[1], bytes[2], bytes[3]];
    let dim = u32::from_le_bytes(header) as usize;
    let expected_data_len = dim.div_ceil(4);
    if bytes.len() != 4 + expected_data_len {
        return VectorHeader::LengthMismatch {
            expected: 4 + expected_data_len,
            got: bytes.len(),
        };
    }
    VectorHeader::Ok { dim }
}

/// The outcome of `decode_vector_cells`.
pub(crate) enum VectorCells {
    Decoded(holon::Vector),
    /// A 2-bit cell decoded to the reserved `0b11` pattern at cell index
    /// `at`.
    InvalidCell { at: usize },
}

/// Decode `bytes[4..]`'s packed ternary cells into a `dim`-dimensional
/// `Vector`. `dim` must already be validated (by `parse_vector_header` plus
/// the caller's dim-count cross-check) — this function trusts it and does
/// not re-derive it from `bytes`.
pub(crate) fn decode_vector_cells(bytes: &[u8], dim: usize) -> VectorCells {
    let mut cells: Vec<i8> = Vec::with_capacity(dim);
    for byte in &bytes[4..] {
        for shift in 0..4 {
            if cells.len() == dim {
                break;
            }
            let bits = (byte >> (shift * 2)) & 0b11;
            let cell: i8 = match bits {
                0b00 => 0,
                0b01 => 1,
                0b10 => -1,
                _ => return VectorCells::InvalidCell { at: cells.len() },
            };
            cells.push(cell);
        }
    }
    // arc 278 STOP-6 (grounded, not assumed): `cells.len() != dim` here is
    // UNREACHABLE and was deleted rather than mapped to a variant. The length
    // check in `parse_vector_header` guarantees `bytes[4..].len() ==
    // dim.div_ceil(4)`, so this loop has `4 * dim.div_ceil(4) >= dim`
    // decodable bit-pairs available — always enough to reach `cells.len() ==
    // dim` (the `break` never lets it exceed dim, and an early return
    // already fires above on any invalid cell). There is no byte-length
    // value that reaches this point with `cells.len() != dim`.
    VectorCells::Decoded(holon::Vector::from_data(cells))
}

/// The failure outcomes of `encode_vector` — raised as wat errors by the
/// door (`vector-bytes` has no matchable `VectorEncodeOutcome` enum on the
/// wat side, unlike decode's `VectorDecodeOutcome`).
pub(crate) enum VectorEncodeError {
    /// `v`'s dimension count doesn't fit in the wire format's `u32` header.
    DimTooLarge,
    /// A cell outside `{-1, 0, +1}` — the ternary wire format has no
    /// representation for it.
    InvalidCell { value: i8 },
}

/// Encode `v`'s ternary cells into the wire format `parse_vector_header` /
/// `decode_vector_cells` decode back: a 4-byte little-endian dim header
/// followed by the cells packed 4-per-byte, 2 bits/cell.
pub(crate) fn encode_vector(v: &holon::Vector) -> Result<Vec<u8>, VectorEncodeError> {
    let dim = v.dimensions();
    let dim_u32 = u32::try_from(dim).map_err(|_| VectorEncodeError::DimTooLarge)?;
    let data_len = dim.div_ceil(4);
    let mut out: Vec<u8> = Vec::with_capacity(4 + data_len);
    out.extend_from_slice(&dim_u32.to_le_bytes());
    let data = v.data();
    for chunk in data.chunks(4) {
        let mut byte: u8 = 0;
        for (i, &cell) in chunk.iter().enumerate() {
            let bits: u8 = match cell {
                0 => 0b00,
                1 => 0b01,
                -1 => 0b10,
                other => return Err(VectorEncodeError::InvalidCell { value: other }),
            };
            byte |= bits << (i * 2);
        }
        out.push(byte);
    }
    Ok(out)
}
