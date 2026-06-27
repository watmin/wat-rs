# Arc 295 — `signed-code-only`: you may only use signed code

> **The doctrine (the builder, verbatim — `294/REALIZATIONS.md`, 2026-06-27):** *"no....... /you may only use
> signed code/ .... there is no option. period. you sign your code. you may only sign your code."* Plus the
> mechanism he laid out: a `wat sign` command (private key **piped in, pipe-only**), `.wat` paired with `.sig`,
> signed eval referencing the **pubkey at the root**, and a **Rust hard-hook** so our key is callable only by the
> binary build system.

**Status: SCOPED (2026-06-27).** This is a foundation-trust arc: code provenance becomes **structural, not
convention** — the same move the datamancy grimoire makes on its spells (signed, verified, not spoofable), now
turned on wat's OWN code. The substrate forces the idealized state: unsigned code cannot eval.

## The one inversion (what flips)
**Signing is opt-in today; the doctrine makes it mandatory and our-key-only.** Everything below is one consequence.

## The foundation — what ALREADY EXISTS (grounded against the disk this session)
wat is NOT greenfield here. `src/load.rs` already carries a full signed-load path:
- **`(:wat::signed-load! "path" :wat::verify::signed-ed25519 <sig-iface> <pubkey-iface>)`** — an opt-in signed
  load form (`load.rs:27`). Verifies **POST-PARSE against the SHA-256 of the canonical-EDN** (`load.rs:60`) — so a
  signature **survives comment/whitespace edits**; the *AST* is what's signed, not the bytes.
- **Sidecar payloads already supported** — `:wat::verify::file-path "sidecar.sig"` resolves a `.sig` file next to
  the source (`PayloadInterface::FilePath`, `load.rs:120`). The `.wat`+`.sig` pairing the doctrine names is the
  EXISTING sidecar path.
- **Crypto deps present** — `ed25519-dalek = "2"` + `sha2 = "0.10"` (root `Cargo.toml:62-63`). `sign_source_ed25519`
  / `SigningKey` / `VerifyingKey` are exercised in `load.rs` tests.
- **`LoadSpec { source, verification: Option<VerificationSpec> }`** (`load.rs:154`) — verification is `Option` =
  the opt-in. The enforcement seam is **`FsLoader`** (`load.rs:936`), the production disk reader.

So ~70% exists. The arc is **flip opt-in → mandatory + our-key-only trust + the `wat sign` tool + the hard-hook +
retrofit 119 files** — not invent signing.

## The target architecture
1. **Mandatory verification at the loader.** `FsLoader` verifies every `.wat` read against its `.sig` sidecar, using
   the **trusted root pubkey** — an unsigned/unverifiable file is a hard load error, no unsigned branch. (`LoadSpec`
   verification stops being `Option` for the Fs path; or `FsLoader` injects the signed-load requirement.)
2. **Our-key-only trust (the hard-hook).** The pubkey the loader trusts is **fixed**, not supplied per-load — so no
   one can sign with their own key and name their own pubkey. (Today `signed-load!` carries the pubkey inline; the
   doctrine pins it.) Candidate: the pubkey is **compiled into the binary** (build-embedded) so a swapped root file
   can't fool it; the root `.pubkey` file is the reference copy.
3. **`wat sign` command** (in `wat-cli`, sibling of `cargo-wat`). Private key **read from stdin only** (pipe):
   `cat priv.key | wat sign <files…>`. Signs the canonical-EDN SHA-256 with ed25519 → writes `<file>.sig`. With no
   path: walks the default positions (`wat/ wat-tests/ wat-scripts/`) and signs all `.wat`.
4. **Retrofit** — sign all 119 `.wat` in `wat/ wat-tests/ wat-scripts/`; commit the `.sig` sidecars.

## Open questions — the genuine forks (for the builder)
- **Q1 — the trusted pubkey: embedded-in-binary, root-`.pubkey`-file, or both?** "Hard hook … callable only by us"
  reads as **compile-time-embedded pubkey** (a swapped file can't downgrade trust); "ref'ing the pubkey at the
  root" reads as a **root file**. My read: **embed the pubkey in the binary (the hard-hook), root file is the
  reference copy** — but it's your call.
- **Q2 — the "key callable only by us" mechanism.** Possession of the private key IS the signing capability — a
  binary can't refuse a key-holder. So the hard-hook is necessarily on the **trust side** (embedded pubkey) +
  **operational** on the private key (we hold it; pipe-only input keeps it out of argv/env/history). Confirm that's
  the intent, or there's a stronger gate you have in mind (e.g. the priv key never leaves a KMS/HSM and `wat sign`
  calls out to it — mirroring datamancy's KMS model).
- **Q3 — inline-source (the doctrine's hard edge).** 5 `load-string!`/inline sites in `src/` + the universe
  bootstrap + in-Rust-string test fixtures are **not files** → can't carry a `.sig` sidecar. Options: **(a)** the
  doctrine governs FILE loads only (inline source is build-trusted, compiled into the binary we already sign as a
  whole); **(b)** inline source carries an embedded sig; **(c)** inline source is forbidden (everything becomes a
  signed file). My lean: **(a)** — the binary itself is the build artifact; inline wat IS the binary, already
  inside the trust boundary. Your call.
- **Q4 — sign the canonical-EDN AST (existing) vs raw bytes (digest).** AST-canonical (existing `signed-load!`)
  survives formatting edits and is the stronger semantic ("authored by us"). Recommend **AST-canonical**; raw-byte
  digest is the weaker sibling. Confirm.

## Decomposition (provisional — after the forks settle)
- **295.0** — the disconfirming probe: an unsigned `.wat` loaded via `FsLoader` must be REJECTED (RED at HEAD —
  today it loads fine). Commit RED.
- **295.1** — `wat sign` command (pipe-only key) + the canonical-EDN signing, producing `.sig` sidecars. (Reuses the
  existing `sign_source_ed25519`.)
- **295.2** — the trusted-pubkey resolution + the Rust hard-hook (embedded pubkey).
- **295.3** — mandatory verification at `FsLoader` (unsigned = hard error). The probe flips GREEN.
- **295.4** — retrofit: sign all 119 `.wat` in default positions; gate the suite green (every test-loaded file now
  carries a `.sig`).
- **295.5** — resolve inline-source per Q3; close.

## Census (grounded 2026-06-27)
- Retrofit corpus: **119 `.wat`** (`wat/ wat-tests/ wat-scripts/`). Existing `.sig`: **0**. Inline-source sites in
  `src/`: **5** (+ the bootstrap + test fixtures).
- Foundation: `src/load.rs` (the signed-load machinery + `FsLoader` seam) · `Cargo.toml` (ed25519-dalek + sha2) ·
  `crates/wat-cli` (`wat sign` home, beside `cargo-wat`).

## Pairs
`294/REALIZATIONS.md` ("you may only sign your code" — the doctrine, verbatim) · `src/load.rs` (the signed-load
foundation) · the datamancy grimoire trust model (signed/verified/KMS — the prior art this mirrors) ·
`project_signed_code_only_doctrine` (memory) · `feedback_substrate_forces_idealized_state`.
