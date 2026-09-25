# BRIEF — STONE M: `wat__holon__Vector` renders as its data, and can be read back

**Drawn 2026-09-25.** Builder: *"why is Value::Vector not just [...] — [1 2 3] is a vec of i64 ?"*
(stone I's amendment). A VSA hypervector is `Arc<holon::Vector>`, backed by `&[i8]` — pure numeric
data, hashed from `data()`. It has an honest EDN representation; `{:dim N}` is neither data nor nil.

## Measured at `974b67387`

- **Purity:** `:wat::holon::Vector` is already in the pure-scalar list (`src/check.rs:13638`), so the
  checker admits it onto a wire. The ONLY defect is the writer/reader pair.
- **Writer:** `src/edn/render.rs` renders `#wat.holon/Vector {:dim N}` (the `wat__holon__Vector` arm,
  ~`:4770`).
- **Reader:** `tagged_to_value`'s `if ns == "wat.holon"` branch (~`:3521`) decodes only the
  `Thermometer`/`SlotMarker` directives via `decode_holon_directive_tag`; `Vector` falls through to
  "unknown tag". Stone H measured it arriving as `Lost: … unknown tag #wat.holon/Vector`.
- **Constructor:** `holon::Vector::from_data(Vec<i8>)` (`holon-rs/src/kernel/vector.rs:26`).

## The work

1. **Writer:** `#wat.holon/Vector [i8 …]` — the tag says which type (a bare `[…]` would read back as a
   wat `Vec`, not a hypervector); the body is the components, in order.
2. **Reader:** in the `wat.holon` branch, `Vector` with a vector body of integers → `from_data`. A body
   that is not a vector, or an element outside `i8`, is a decode ERROR that names what was wrong —
   never a clamp, never a silent truncation.
3. **Typed decode:** confirm a `#wat.holon/Vector […]` arriving in a `:wat::holon::Vector`-typed slot
   (the wire `recv` path, `edn_to_typed_value`) decodes to the hypervector. Report the path you found.
4. **Two stale comments:** the Vector arm's comment still cites *"the same 'preserve real data' call
   294.i made for HandlePool's name"* — stone I reversed that call; and `src/value/value.rs:~745`
   still lists `Vector` for what is now `wat__holon__Vector`.

## Prove it

- **Round trip:** a hypervector written then read back is EQUAL (`PartialEq` compares data slices).
- **Over a process wire:** a `Box` holding a hypervector crosses and arrives equal (model on stone G/H's
  probes).
- **Refusals:** `#wat.holon/Vector {:dim 4}` (the OLD form) and `#wat.holon/Vector [1 300]` are decode
  errors naming the problem.
- **Mutation:** restore the `{:dim N}` writer → the round-trip and wire cases red.

## Size, stated rather than discovered

A hypervector is typically 4096 components, so a `println` of one prints ~4096 integers. That is the
cost of an honest, round-trippable form, and it is accepted. Report the byte size of one default-dim
vector's EDN.

## STOP triggers

1. Something consumes `{:dim}` for behaviour — report it before removing.
2. The typed-decode path cannot reach a new reader arm without a wider change — report the path.
3. Any gate reddens that you did not add — capture whole, name the arm. ⛔ Do not re-run first.

## Mechanics — ⛔ read

- Floor: `nohup scripts/floor.sh > <file> 2>&1 &`, then repeated foreground
  `until grep -qE '^ *Summary' <that file>; do sleep 30; done` blocks. Do not end your turn while it
  runs. Never `pgrep -f 'cargo …'`. Read `Summary` lines, never piped exit codes.
- No `cargo fmt` / `rustfmt`. Stage explicit paths. A test that compares against an EDN string literal
  trips `no_inlined_edn` — use an `.edn` golden with `assert_edn_matches_file!` (stone I hit this).
