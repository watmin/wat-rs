# DESIGN STONE — 294.k · a fabricated home is a lie; make it RAISE

**Builder's standing ruling:** `#wat-edn.*` is annihilated — only `#wat.*` survives. 294.i took
`.opaque`; 294.j took `.holon`. **Eight sites remain, in four families. This stone takes five.**

```
wat-edn.local     3   edn_shim.rs:2660, 3966, 3968      0 tests, 0 goldens   ← this stone
wat-edn.opaque    2   edn_shim.rs:3964, 3969            0 tests, 0 goldens   ← this stone
wat-edn.cap       2   edn_shim.rs:2839, registry.rs:71  2 test files         ← builder's ruling
wat-edn.float     1   crates/wat-edn/parser.rs:361      2 test files         ← builder's ruling
```

## The defect, verbatim from the code

```rust
fn tag_from_type_path(path: &str) -> Tag {
    let stripped = path.strip_prefix(':').unwrap_or(path);
    if let Some(idx) = stripped.rfind("::") {
        let ns = stripped[..idx].replace("::", ".");
        let name = &stripped[idx + 2..];
        Tag::try_ns(&ns, name).unwrap_or_else(|_| Tag::ns("wat-edn.opaque", "unnamed"))   // ← 3964
    } else {
        // No namespace separator — fabricate a "wat-edn.local" namespace
        // so wat-edn's spec-required namespace constraint is met.
        Tag::try_ns("wat-edn.local", stripped)                                            // ← 3968
            .unwrap_or_else(|_| Tag::ns("wat-edn.opaque", "unnamed"))                     // ← 3969
    }
}
```

Two lies, stacked:

1. **`.local` FABRICATES a home** for a type whose path has no `::`. The comment admits it — *"fabricate
   … so wat-edn's spec-required namespace constraint is met."* A tag namespace is supposed to say where
   a type LIVES; this one says only *"we needed to write something here."*
2. **`.opaque/unnamed` DISCARDS the identity** when `try_ns` rejects the split, replacing a type's name
   with the literal word `"unnamed"` — **and raises nothing.** A value crosses with its identity erased
   and no one is told.

`struct_tag_for` (`edn_shim.rs:2653`) is the **decode-side mirror** and carries the identical `.local`
fallback. ★ **Two implementations of one concept, which is the pattern that has bitten this arc three
times** (294.j's `holon_to_watast` vs `from_holon_item`; task #102's `watast_to_holon` vs
`to_holon_inner`). They must move together, and a differential between them is the honest gate.

## ★ MEASURED: nothing takes the fabricating branch

Every `type_path` in the tree is `::`-separated:

```
:probe::Point                :test::Token              :wat::edn::Validation
:wat::eval::StepResult       :wat::kernel::RunResult   :wat::kernel::LociDiedError
:wat::kernel::ReadlnOutcome  :wat::sqlite::Cell        :wat::io::IOReader::ReadFrameOutcome
```

`rfind("::")` succeeds on all of them, so the `else` branch — every `.local` site — is **unreached by
any observed value.** And **zero tests, zero goldens** carry `wat-edn.local` or `wat-edn.opaque`, so
nothing has ever confirmed they fire at all.

⚠ **That is a measurement of the OBSERVED set, not a proof of the reachable set.** Which is exactly why
this stone does not delete the arms on the strength of it — see the method below.

## The design — the wall 294.j already ruled

294.j established it for holons: *a value that is neither data nor a directive **RAISES**; there is no
fallback rendering.* The same reasoning applies verbatim here. **A type whose home cannot be derived has
no honest tag. Fabricating one, or erasing the name to `"unnamed"`, are both lies — and the lie is
SILENT, which is the part that matters.**

```
tag_from_type_path(path):
    ns::name derivable  →  Tag::ns(ns, name)
    otherwise           →  RAISE, naming the offending path
```

## ★ THE METHOD: impose the check, read the screams — do NOT survey first

`[[feedback_impose_the_check_and_read_the_screams]]` — my census has been wrong five times when I
surveyed for a worklist, and right every time I made the wall and let the corpus scream.

**Replace both fallbacks with a raise, then run the floor.** Two outcomes, both good:

- **Floor silent** → the arms were dead. Five sites to zero, and a wall stands where a silent lie did.
- **Floor screams** → the screams ARE the worklist, and each names the exact type path that has no
  derivable home. That is a fact worth having regardless of this stone, and it is not obtainable by
  reading.

Do not pre-enumerate. Do not add a temporary log and survey. The raise IS the instrument.

## The four questions — flat

**Obvious? YES.** *A type whose home cannot be derived raises.* One sentence; the alternative requires
explaining what `.local` and `"unnamed"` mean, and neither has an answer that isn't "we had to write
something."

**Simple? YES.** Two fallback arms become one raise. The stone deletes more than it adds and introduces
no new concept — 294.j already minted this exact wall.

**Honest? YES**, and it is the whole point. Today's encode path can emit a tag whose namespace is
invented and whose name is the word `"unnamed"`, with nothing raised. That is a value crossing a wire
having quietly lost its identity — the same class as `.opaque`'s death warrant, which is how this
family got condemned in the first place.

**Good UX? YES.** A raise names the offending path at the moment of the defect. `#wat-edn.opaque/unnamed`
names nothing and surfaces at whatever later point the receiver notices it cannot reconstruct.

## The gate

| # | assertion |
|---|---|
| 1 | `grep -rn 'wat-edn\.local' src/ crates/ tests/ wat/ wat-scripts/ wat-tests/` → **0** |
| 2 | `grep -rn 'wat-edn\.opaque' src/ crates/ tests/ wat/ wat-scripts/ wat-tests/` → **0** |
| 3 | `tag_from_type_path` and `struct_tag_for` agree — a **differential test** feeding both the same paths and asserting identical (ns, name), including that both raise on the same input |
| 4 | a path with no derivable home **RAISES**, and the error **names the path** (not a generic message) |
| 5 | floor GREEN via `scripts/floor.sh` — the **Summary line**, never a piped exit code |
| 6 | `cargo clippy --release --all-targets` → **0** |
| 7 | `#[ignore]` count **13**, unmoved |
| 8 | the raise is covered by a kept test — `[[feedback_a_negative_control_that_can_be_kept_must_be_kept]]` |

Row 3 is the one nobody has ever run, and it is the row that would have caught this class in
`watast_to_holon` vs `to_holon_inner` (task #102) the day they diverged.

## Out of scope — RULED, not deferred

- **`.cap`** (2 sites) — a **SECURITY BOUNDARY**. `edn_shim.rs:2839` refuses a forged capability by
  matching the namespace STRING; `capability/registry.rs:71` emits it. Renaming moves a refusal
  predicate and the two must move atomically or a forge window opens. Builder: *"we need to discuss
  those… i do not trust your judgement on them."* **His ruling, not this stone's.**
- **`.float`** (1 site) — `crates/wat-edn`'s OWN sentinel for NaN/±Inf, values EDN cannot express. The
  crate IS named `wat-edn`, so this is arguably the library naming itself rather than the substrate
  leaking. It has Clojure interop tests, so a rename is a wire-format change for external readers.
  **His ruling.**
- **Structs on the wire** — checked this session and it is **already law**, not a gap. `Nature::Struct`
  is impure alongside `Nature::Peer` (`types.rs:178`); the wall is enforced at wire-peer PRODUCERS at
  compile time; the §7 runtime backstop was deliberately retired (293.W.2d) as subsumed; and it is
  proven by `tests/comms/probe_arc293_W2a_struct_no_cross.rs` among others. The shim's struct arms
  serve **local rendering** (`str`, diagnostics, chain envelopes) — `value_to_edn_with` is not
  wire-only — so they stay. **Nothing to build.**
