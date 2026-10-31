# dimensionX — Persistent Context, Architecture State & Learnings

**Updated:** 2026-10-31  
**Project Root:** `/Users/ajaychakravarthy/code/dimensionX`  
**License:** MIT (`LICENSE`)  
**Rust Core Verification:** `export PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$PATH" && cd rust-core && cargo test` (`15/15` tests passing in ~80 ms)  
**JS & WASM Verification:** `npm test` (`8/8` WASM & visualizer tests passing)  
**Interactive Server Command:** `npm run serve` (`node scripts/serve.mjs` -> `http://127.0.0.1:4173`)

---

## 1. Goal & Ground Rules (Strictly Enforced)
1. **MIT License Only (`REQ-FOUND-001`):** `LICENSE` contains the full MIT License. Both `rust-core/Cargo.toml` and `package.json` have **zero external dependencies** (`std` only in Rust, standard ES2024 in JS).
2. **Core in Rust + Visualization in JS (Zero Duplication):**
   - **Rust Core (`rust-core/`):** Implements 100% of the $N$-dimensional mathematics (`math.rs`), parameterized configuration (`config.rs`), universal proper-time clock with slow-down (`-`) and speed-up (`+`) (`spacetime.rs`), all 6 simulation systems & ECS universe (`universe.rs`), and 16 WebAssembly C-ABI exports (`wasm_api.rs`). Compiled to `src/wasm/dimensionx_core.wasm`.
   - **JS Visualization (`src/`):** Contains **zero** duplicated math, ECS, clock, or simulation systems (`src/core/`, `src/components/`, `src/systems/`, and `src/engine/DimensionXUniverse.js` were completely removed). Loads `src/wasm/dimensionx_core.wasm` via `RustWasmUniverse` (`src/engine/RustWasmBridge.js`) and drives the dual-viewport 3D Space-Time & 2D Flatland Cross-Section visualizer (`SpaceTimeVisualizer.js`, `ProjectorND.js`) and top-bar Space-Time HUD (`ControlHUD.js`, `index.html`, `styles.css`).
3. **Zero Hardcoding (`REQ-FOUND-002`):** No subsystem hardcodes 2D/3D limits or magic simulation constants. Everything is parameterized via `SimulationConfig`, supporting arbitrary integer universe dimensions $N \ge 2$ and procedural axiom generation `get_axioms_for_dimension(target_dim)`.
4. **Zero Mock / Spurious Code (`REQ-FOUND-003`):** Every struct, class, function, and test executes real mathematical and simulation logic.
5. **MiniMax `mcode-stack` Skills Installed (`.agents/skills/` & `.agents/checklists/`):**
   - Installed from `https://github.com/MiniMax-AI/mcode-stack` (`stacks/ronny` and `stacks/jiachen/skills/lifting-loops`).
   - Applied `subagent-split-check`, `independent-review`, `engineering-code-review`, `long-session-check`, and `handoff-doc`.

---

## 2. Architecture & Implemented Module Map

### 2.1. Rust Core Engine (`rust-core/src/`)
- [`rust-core/src/math.rs`](./rust-core/src/math.rs): `SeededRandom` (Mulberry32 + Marsaglia polar Gaussian), `VectorND` ($\mathbb{R}^N$ Euclidean algebra, projections, embeddings), `RotorND` ($SO(N)$ plane rotations across $\binom{N}{2}$ planes, `determinant`, `inverse`, Modified Gram-Schmidt `reorthogonalize`), `PolytopeND` (2D polygons $\{p\}$, $k$-Simplexes $\alpha_k$, $k$-Hypercubes $\gamma_k$, $k$-Orthoplexes $\beta_k$), and `CrossSectionND` (`slice_polytope` with deduplicated `ordered_polygon_2d`).
- [`rust-core/src/config.rs`](./rust-core/src/config.rs): `BoundaryMode` (`Reflective`, `Toroidal`) and `SimulationConfig` with `get_axioms_for_dimension(target_dim)`.
- [`rust-core/src/spacetime.rs`](./rust-core/src/spacetime.rs): `TIME_SCALE_LADDER = &[0.0, 0.05, 0.1, 0.25, 0.5, 1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 100.0, 250.0, 500.0, 1000.0]` and `SpaceTimeClock` supporting base $1.0\text{ s/s}$, sub-real-time slow-motion (`0.05x..0.5x`), hyper-warp (`2x..1000x`), `slow_down_time()` (`-` button), and `speed_up_time()` (`+` button).
- [`rust-core/src/universe.rs`](./rust-core/src/universe.rs): Complete `DimensionXUniverse` implementing all 6 systems (`step_kinematics_nd`, `step_higher_dim_injection`, `step_peer_exchange`, `step_inheritance_mutation`, `step_collaborative_folding`, `step_dimensional_phasing`), `sample_worldlines()`, bounded $O(V)$ `compute_being_cross_section()`, in-place `upsert_collaboration_link()`, and pure-Rust JSON serialization `to_snapshot_json()`.
- [`rust-core/src/wasm_api.rs`](./rust-core/src/wasm_api.rs): 16 `#[no_mangle] pub extern "C"` WebAssembly FFI exports (including `universe_set_include_worldlines`).
- [`rust-core/src/lib.rs`](./rust-core/src/lib.rs): 15 deterministic unit, integration & sustained-performance tests covering math, space-time slow-mo/hyper-warp, all 6 simulation systems, WASM FFI, and 600-tick proper-time frame budget (`REQ-PERF-001`).

### 2.2. WebAssembly Binary & JS Bridge (`src/wasm/` & `src/engine/`)
- [`src/wasm/dimensionx_core.wasm`](./src/wasm/dimensionx_core.wasm): Compiled `wasm32-unknown-unknown` release binary.
- [`src/engine/RustWasmBridge.js`](./src/engine/RustWasmBridge.js): Zero-dependency `RustWasmUniverse` loading `dimensionx_core.wasm`, invoking C-ABI exports (`universe_init`, `universe_seed_default`, `universe_step`, `universe_step_ticks`, `universe_set_time_scale`, `universe_slow_down_time`, `universe_speed_up_time`, `universe_set_slice_z`, `universe_set_include_worldlines`, `universe_trigger_injection`, `universe_jump_being_to_dimension`, `universe_spawn_being`, `universe_refresh_snapshot`), and decoding UTF-8 JSON snapshots from WASM linear memory.

### 2.3. Visualization & Space-Time HUD (`src/render/` & `src/ui/`)
- [`ProjectorND.js`](./src/render/ProjectorND.js): Cascaded $N\text{D} \to 3\text{D} \to 2\text{D}$ perspective/oblique projector with camera orbit, zoom, hyperplane slice offsets, and Space-Time Worldline Mode ($Z = \Delta T$).
- [`SpaceTimeVisualizer.js`](./src/render/SpaceTimeVisualizer.js): Dual-canvas renderer driving both the 3D/$N$-D Omniscient Space-Time Viewport and the 2D Flatland Cross-Section Slice Viewport, with deduplicated bidirectional peer-link rendering and facet cap for 5D/6D polytopes.
- [`ControlHUD.js`](./src/ui/ControlHUD.js), [`index.html`](./index.html), [`styles.css`](./styles.css), [`src/main.js`](./src/main.js), [`scripts/serve.mjs`](./scripts/serve.mjs): Includes prominent **`− Slow`** (`#btn-time-slower`) and **`+ Fast`** (`#btn-time-faster`) buttons in the top Space-Time Control Bar, slow-motion preset (`0.25×`), keyboard shortcuts (`-`/`+`), and diff-guarded DOM updates in `_updateInspectorPanel` and `_updateEventFeed`.

---

## 3. Key Technical Learnings & Fixes Applied During SDLC
1. **Rust Toolchain PATH on macOS Homebrew:**
   - Homebrew installs `rustup` as keg-only in `/opt/homebrew/opt/rustup/bin`. Prefix shell commands with `export PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$PATH"`.
2. **Direct World-Vertex, Facet & Cross-Section Snapshot Rendering (`Rust WASM` $\to$ `JS Visualizer`):**
   - In `rust-core/src/universe.rs`, `to_snapshot_json()` outputs world-transformed `vertices = pos + orientation * local_v`, `edges`, `facets`, and cascaded 2D hyperplane cross-sections `sliceAtZ` (`Self::compute_being_cross_section(b, 2, self.viewer_slice_z)`). `SpaceTimeVisualizer.js` renders directly from this snapshot with zero duplicated math or ECS state in JS.
3. **32-bit Unsigned State Coercion in `SeededRandom`:**
   - In `rust-core/src/math.rs`, `self.state = self.state.wrapping_add(0x6D2B_79F5)` enforces wrapping 32-bit unsigned arithmetic for full-period Mulberry32 determinism.
4. **Sustained Proper-Time 60 FPS Performance (`REQ-PERF-001`):**
   - Eliminated the $O(V^{2^{M-3}})$ complete-graph edge explosion in `compute_being_cross_section` by adding an $O(1)$ bounding-sphere pre-check, deduplicating intermediate slice vertices within `1e-3` (capped at 24), and constructing an $O(V)$ cyclic + antipodal-chord edge graph (`<= 2*n` edges) at each intermediate dimension step.
   - Added `BeingEntity::upsert_collaboration_link` (capping at 8 links per being) and moved the `target_pop >= max_pop_per_dim` check to the start of `step_collaborative_folding`, eliminating 60 Hz link spam and cutting 600-tick test execution time by $48\times$ ($3{,}847\text{ ms} \to 80\text{ ms}$).
   - Added `universe_set_include_worldlines` so worldline arrays are only serialized into the JSON snapshot when 4D Worldline trails are toggled on, and diff-guarded `ControlHUD._updateInspectorPanel` / `_updateEventFeed` so DOM nodes and phase-jump button listeners aren't rebuilt unless structural state changes.

---

## 4. Documentation Suite (`docs/`)
- [`docs/USER_GUIDE.md`](./docs/USER_GUIDE.md): Complete user manual with 8 real screenshots (`docs/images/screenshot-*.png`) explaining every viewport element, button, slider, inspector field, phase-jump control, and keyboard shortcut.
- [`docs/TECHNICAL_DOCUMENTATION.md`](./docs/TECHNICAL_DOCUMENTATION.md): Detailed engineering & mathematical reference with 4 vector architectural diagrams (`docs/images/*.svg`) covering `math.rs`, `config.rs`, `spacetime.rs`, `universe.rs`, `wasm_api.rs`, `RustWasmBridge.js`, `ProjectorND.js`, `SpaceTimeVisualizer.js`, and `ControlHUD.js`.
- [`docs/README.md`](./docs/README.md): Documentation & image catalog index.

---

## 5. How to Resume in a Future Session
1. Read `CONTEXT_AND_LEARNINGS.md`, `AGENTS.md`, `REQUIREMENTS.md`, `ARCHITECTURE.md`, and `docs/TECHNICAL_DOCUMENTATION.md`.
2. Run `export PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$PATH" && cd rust-core && cargo test && cd .. && npm test` to verify all 15 Rust tests and 8 JS/WASM tests pass.
