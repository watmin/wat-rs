# Arc 295 — `signed-code-only`: you may only use signed code

> **The doctrine (the builder, verbatim — `294/REALIZATIONS.md`, 2026-06-27):** *"no....... /you may only use
> signed code/ .... there is no option. period. you sign your code. you may only sign your code."*

**Status: SCOPED — model locked 2026-06-27 (refine ON DISK, not in volatile context).** Foundation-trust arc: code
provenance becomes **structural, not convention** — the datamancy static-MCP trust model (signed manifest, pinned
key, detached sig, release chain), **rebuilt as wat's own, in EDN, with no JSON / no blobs / no KMS dependency.**

> **PATH NOTE (amend-with-recognition):** live working contract. Superseded passages get a dated `⊘ SUPERSEDED`
> note; nothing is deleted. The model below was reached through a fast co-design (the 295 chat, 2026-06-27); the
> turns are preserved in the supersession notes because the tension taught.

## The one inversion
**Signing is opt-in today (`src/load.rs`); the doctrine makes it mandatory and key-pinned.** Every wat distribution
ships as a verifiable **signed release chain**; unsigned code cannot eval.

## ⊘ SUPERSEDED — the first-draft sketch (2026-06-27 AM), and why each piece turned
The arc opened with a simpler model; co-design with the builder turned four pieces. Kept here with recognition:
- **per-file `.sig` sidecars** → **one signed manifest over all files** (file → digest). One sig, not 119. (His:
  *"the pubkey validates the sig who signed over the manifest of all signed files."*)
- **JSON manifest** (imported from datamancy's web wire) → **EDN manifest.** (His: *"there is no json — we vend edn
  — i will not be misunderstood. wat is edn."*) Datamancy's JSON is *their* wire; wat vends EDN, period.
- **P-256/KMS as the trust root** → **no runtime KMS dependency.** Algorithm is a pluggable `:sig` support; KMS/HSM
  is a *sign-side backend only.* (His: *"we do not dep on kms — we provide different load signatures."*)
- **datamancy's `blobs/sha256/` store** → **dropped.** Content-addressed blobs are datamancy's *web-vending*
  concern; a wat distribution is the source tree + the chain, files referenced by **path + hash.** (His: *"a
  distribution of wat is just a signed release chain."*)

## The model (locked)

### EDN, throughout. No JSON, no blobs.
The manifest, the chain links, the version, the key registry — all EDN. A wat distribution is **`(manifest,
enumerate-files ["src/" "wat/" "wat-tests/" "wat-scripts/" …])`** — the source tree as it lives, plus the signed
manifest. Files are referenced by **path + canonical-EDN hash** (no blob indirection; the tree IS the content).

### The signature target — canonical-EDN hash (Q4, decided)
Sign the **SHA-256 of the canonical-EDN** of each file (the existing `signed-load!` semantics, `load.rs:60`) — so a
signature survives comment/whitespace edits; the *AST* is what's signed, not raw bytes.

### `:sig` — pluggable signature support (decided; renamed from `:alg`)
A key's signatures are verified by a named **support**, not a bare algorithm — because the two real custody modes
differ in the *whole* path (encoding + production + verification), not just the curve. **Two honest supports today,
named not hidden, extensible (*"whatever else we add later"*):**
- **PEM-on-disk** — local keypair (ed25519, the existing dep; raw sig). `wat sign` takes the key by **pipe**.
- **remote-HSM** — non-exportable key (P-256, DER sig). `wat sign` calls the HSM; the private key never lands.

**The runtime never deps KMS:** verification needs only `(pubkey, support)` — pure crypto, no network. The HSM lives
only in the `wat sign` tool's sign-side backend.

### The manifest is MULTI-KEY (key rotation / loss, decided)
A real trust config is plural. The manifest carries:
- **All pubkeys, accreted, never deleted** — so every historical release stays verifiable forever.
- **Per file: `(path, digest, signing-pubkey)`** — provenance recorded, not assumed.
- **The chain: each `:previous` paired with the pubkey that signed THAT release** — every link carries the key to
  verify it.
- **Rotation/loss falls out:** lose the primary key → ship a new release signed by a **new** key that extends the
  chain → old files **keep their old-key signatures** (you can't re-sign — the old private key is gone — and needn't,
  the old *public* key is retained). New content rides the new key. *"We handle releases with many keys."* The chain
  can be **forked by any held key.**

### The release chain — timestamped, prunable
- **Version = ISO8601 UTC timestamp** (`2026-06-27T14-32-08Z`) — monotonic, self-ordering, signed *into* the
  manifest (so the version is unforgeable, not a label beside it).
- **`:previous sha256:…`** — append-only, tamper-evident log; walk it backward to audit.
- **The head is the trust entry; the chain back is the audit log.** At load, verify against the *current* manifest;
  the chain is for `wat verify` (trivial: walk `:previous`, check each sig under its paired key).
- **Prunable by the signer (forking is trivial):** stop appending, truncate to a known-good release, branch a new
  direction. The abandoned forward tail is **forgotten** (unreferenced by the published head). Tamper-evidence is
  against *outsiders* (no key → no splice/forge); *we* hold the key, so the chain is ours to manage. A consumer
  pinned to a truncated release finds their pin **no longer chains to the head** — that's the loud detection, not a
  failure.
- Honest scope: the ISO8601 version is a **self-signed** timestamp (we attest "this release is T"), not a TSA. The
  chain gives ordering without an external authority; RFC3161 is a later additive `:sig`-style support if ever needed.

### Surface (decided)
```
(load "path" :label)              ; verify "path" against the trust-config bound to :label (default label if omitted)
(load "path")                     ; default label → our embedded key (the build-only root)
(load-key <manifest> :label)      ; register a whole trust manifest under :label. WRITE-ONCE (collision → denied).
```
- The **manifest** is the unit `load-key` consumes — many pubkeys + file→(digest,key) + the chain — NOT a bare key.
- **Write-once registry:** first `load-key` for a label wins; a second binding of the same label is a hard error
  (*"first loader wins, second is denied"*) — no relabel/rebind mid-run.
- The **default label** is pre-bound at build to our **embedded pubkey** (string literal, the hard-hook). External
  callers can neither bind it (taken → collision) nor satisfy it (they lack our private key) → *"illegal outside the
  binary build system,"* enforced by math.
- **Convenience loaders are wat `defn` wrappers** over the contract (e.g. `(defn load-key-file [p l] (load-key
  (wat.io/read p) l))`) — distinct named functions, not a defclause, not macros (ordinary runtime values).
  *"however we want to construct helpful key loaders are just wrappers on that contract."*

### Distributions are signed compositions (the Battery pattern, decided)
A wat distribution is a **composition**: the wat core + N extension crates ("drivers" / optimization layers at the
Rust layer) + their wat code, under one signed manifest, **namespaced — *"you may not share names."*** This is the
EXISTING **Battery** mechanism (`wat-cli::run_with_args(batteries: &[Battery], …)`; the `wat` binary already composes
`wat_telemetry` + `wat_sqlite` + `wat_lru` + `wat_holon_lru` + `wat_telemetry_sqlite`, each a `(register,
wat_sources)` pair). **`foobar-wat` = wat + the `foo` battery + the `bar` battery.** A plain wat program needs no
drivers; heavier distributions compose several. Arc 295's job: put the **signed manifest over the composed
distribution**, so any composition ships as one verifiable signed-release-chain unit.

## The foundation — what ALREADY EXISTS (grounded 2026-06-27)
NOT greenfield:
- **`src/load.rs`** — `(:wat::signed-load! "path" :wat::verify::signed-ed25519 <sig> <pubkey>)`, verifying the
  canonical-EDN SHA-256, sidecar payloads, `LoadSpec{ verification: Option<…> }` (the opt-in to flip), `FsLoader`
  (the enforcement seam). `sign_source_ed25519` already in tests.
- **`Cargo.toml`** — `ed25519-dalek = "2"` + `sha2 = "0.10"`. (P-256 verify is the one new dep, for the HSM support.)
- **`crates/wat-cli`** — `run_with_args(&[Battery])` (the composition mechanism) + `wat sign` home (beside `cargo-wat`).
- **Prior art mirrored (architecture only):** datamancy static-MCP — `pinned-pubkey.ts` (embedded const root),
  `signature.ts` (detached-sig verify, fail→reject), the chained manifest (`:version`/`:previous`/`:resources`).
  wat takes the *shape*, not the JSON / blobs / KMS.

### The manifest measures the RUST source too (decided)
A distribution is built AND vended as `src/` + `wat/` — and **the manifest measures the Rust files as well**, not
just `.wat`. *"That path ensures the files are correct before compilation."* So the seal covers the whole
supply chain: the Rust drivers and the wat code are both hash-pinned in the manifest, verified before `cargo build`,
not just at wat-load. (His: *"we measure the rust files in the manifest too."*)

### Q-CHAIN — RESOLVED → (a) least authority (decided 2026-06-27)
At verify time, an old file is anchored to **its origin key**, not re-vouched by the head. `wat/core.wat` signed
under `:key/2026-01` is verified against the retained `:key/2026-01` pubkey via the chain release that signed it —
**a compromise of a later key cannot forge an earlier file.** Each key vouches only for what it actually signed.
(Common single-key case: head-only, identical to (b). Rotation: the old-key files consult their chain release,
cacheable.) This is the literal reading of *"all prior files stay signed with the lost key."*

### Q-COMPOSE — RESOLVED → a distribution is a user composition, signed as a unit (decided 2026-06-27)
*"users can distribute their own wat compositions with whatever collection of crates … the crates just extend wat
beyond its core … they ship their own code with their distribution as well — that's the composition."* A
**distribution** = a user-assembled composition (wat core + chosen extension crates + the user's own code), built
and vended as `src/` + `wat/`, signed as ONE unit by the distributor (whose `:label` is the trust anchor; the
manifest measures every file, Rust + wat). Crates are upstream building blocks (the Battery mechanism); a distributor
**vouches for their whole composition** (reviewed + measured before inclusion). The runtime **label registry** is
what composes trust roots *across* distributors at load time — each its own `:label`/manifest. So: composite *per
distribution* (one signer per distro), plural *across* distributions (the registry).

## Open questions — remaining
- **Q-VERIFY-DEPTH — head-only vs full-chain at load?** Head-manifest verification is the hot path; the full-chain
  walk is `wat verify` (audit). Likely head-only at load (+ the chain release for any old-key file per Q-CHAIN);
  full walk is opt-in audit. Confirm at the strike.
- **Q-DEFAULT-LABEL name** — the reserved default keyword (`:wat` / `:wat::self` / …) users can't bind. `intueri` at
  the strike.

## Decomposition (provisional — after Q-COMPOSE settles)
- **295.0** — RED probe: an unsigned file via `FsLoader` is rejected; a tampered file (hash≠manifest), a wrong label,
  a wrong `:sig`, a broken chain, and a write-once collision are each rejected. Commit RED.
- **295.1** — the EDN manifest format + `wat verify` (parse, chain-walk, hash + sig check). Pure, testable.
- **295.2** — `wat sign` (manifest generation + canonical-EDN signing; PEM-pipe backend first, HSM backend stub).
- **295.3** — the embedded default pubkey + the label registry (`load-key <manifest>`, write-once) + the `:sig`
  verifier dispatch.
- **295.4** — mandatory verification at `FsLoader` (unsigned/unverified = hard error). The probe flips GREEN.
- **295.5** — retrofit: sign all 119 `.wat` (default positions) → the suite's loaded files all chain-verify.
- **295.6** — the composition/Battery manifest (Q-COMPOSE); inline-source policy; close.

## Census (grounded 2026-06-27)
Retrofit corpus **119 `.wat`** (`wat/ wat-tests/ wat-scripts/`); existing `.sig`/manifests **0**; inline-source
sites in `src/` **5** (+ bootstrap + test fixtures). Foundation: `src/load.rs`, `Cargo.toml` (ed25519+sha2),
`crates/wat-cli` (Battery + sign home).

## Pairs
`294/REALIZATIONS.md` (the doctrine, verbatim) · `src/load.rs` (the signed-load foundation) · datamancy
`src/{pinned-pubkey,signature,manifest}.ts` (the architecture mirrored — EDN not JSON, no blobs, no KMS) ·
`project_signed_code_only_doctrine` (memory) · `feedback_substrate_forces_idealized_state`.
