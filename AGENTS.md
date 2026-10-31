# dimensionX — Agentic SDLC Rules & Invariants

These rules are mandatory for the orchestrator and all subagents working on `dimensionX`:

1. **MIT License Compliance (`LICENSE`):**
   - Only use zero-dependency native implementations or strictly MIT-licensed dependencies.
   - Core simulation & math logic in `rust-core/src/` (100% pure Rust `std`, compiled to `src/wasm/dimensionx_core.wasm`) and visualization/HUD code in `src/` and `tests/` (100% native ES Modules) have zero third-party runtime dependencies.
2. **Core in Rust, Visualization in JS — Zero Duplication:**
   - All $N$-dimensional mathematics (`VectorND`, `RotorND`, `PolytopeND`, `CrossSectionND`, `SeededRandom`), configuration (`SimulationConfig`), space-time clock (`SpaceTimeClock`), and simulation systems (`DimensionXUniverse`) live exclusively in `rust-core/src/`.
   - JavaScript (`src/engine/RustWasmBridge.js`, `src/render/`, `src/ui/`, `src/main.js`) contains ONLY the WebAssembly FFI bridge, the 2D/3D viewport projection/rendering layer, and the DOM HUD controller — never duplicate core math or simulation systems in JS.
3. **Native Implementation Only — Zero Mocks or Spurious Code:**
   - Never write placeholder functions, `todo!()`, `unimplemented!()`, `TODO` stubs, fake/mock return values, or unimplemented interfaces. Either implement a feature completely with mathematical rigor or do not include it.
4. **Zero Hardcoding:**
   - Never hardcode spatial dimension limits (e.g., `x, y, z` or `N=3`). All math, physics, polytopes, and components must operate on parameterized $N$-dimensional structures configured via dynamic simulation parameters (`SimulationConfig`).
5. **Requirements-Driven Development (`REQUIREMENTS.md`):**
   - Every module and system must trace back to formal requirement IDs (`REQ-*`) in `REQUIREMENTS.md`.
6. **MiniMax `mcode-stack` Skills Discipline (`.agents/skills/`):**
   - Follow `subagent-split-check` when delegating bounded modules to subagents.
   - Follow `independent-review` and `engineering-code-review` to verify subagent outputs against requirements and real test outputs.
   - Follow `handoff-doc` and `long-session-check` by maintaining `CONTEXT_AND_LEARNINGS.md` at every milestone so context resets lose zero state.
7. **Real, Non-Mocked Tests (`cargo test` & `npm test`):**
   - Every test in `rust-core/src/lib.rs` and `tests/` must execute the real implementation and assert concrete mathematical, physical, and rendering invariants.
8. **Accurate Documentation:**
   - `README.md`, `ARCHITECTURE.md`, `REQUIREMENTS.md`, and `CONTEXT_AND_LEARNINGS.md` must reflect only what is genuinely implemented and verified by tests.
