# dimensionX — Software Requirements Specification (SRS)

Version: 1.0.0  
License: MIT  
Status: Active Specification (Traced to `ARCHITECTURE.md`, `rust-core/src/lib.rs`, and `tests/`)

---

## 1. Scope & Foundational Constraints

### REQ-FOUND-001: MIT License & Zero-Dependency Native Rust + JS Implementation
- **Description:** The entire `dimensionX` codebase shall be distributed under the MIT License (`LICENSE`). All core $N$-dimensional mathematics, entity/universe state, space-time clock, cross-section slicing, and simulation systems shall be implemented natively in **Rust** (`rust-core/`, Edition 2021, zero external third-party crates—100% standard library `std` for guaranteed MIT purity and zero supply-chain risk), compiled to WebAssembly (`src/wasm/dimensionx_core.wasm`), while the 2D/3D projection and interactive visualization layer shall be implemented in **JavaScript** (`src/engine/RustWasmBridge.js`, `src/render/`, `src/ui/`) with **zero** duplicated core mathematics or simulation systems in JS.
- **Verification:** `rust-core/Cargo.toml` and `package.json` contain zero external runtime dependencies, `LICENSE` is present at repository root, and `cargo test` (`15/15`) + `npm test` (`8/8`) execute cleanly.

### REQ-FOUND-002: Zero Hardcoding & Parameterized Configuration
- **Description:** No subsystem shall hardcode a fixed number of dimensions (such as 2D or 3D), fixed entity counts, or fixed simulation constants. All simulation parameters—including ambient universe dimension $D_{\max}$, initial active dimension $D_{\text{init}}$, base tick rate $\delta t$, speed of information $c_{\text{info}}$, mutation rate $\mu$, communication radius $r_{\text{comm}}$, and folding thresholds—shall be injected via a configurable `SimulationConfig` struct (`rust-core/src/config.rs`).
- **Verification:** Unit and integration tests in `rust-core/src/lib.rs`, `tests/rust-wasm.test.mjs`, and `tests/projector.test.mjs` instantiate worlds with varying dimensions ($N = 2, 3, 4, 5, 6, 7$) and custom config parameters without code changes.

### REQ-FOUND-003: Zero Mock Code or Spurious Stubs
- **Description:** Every exported struct, class, method, system, and test shall be fully implemented. Stubs, `todo!()`, `unimplemented!()`, `TODO` placeholders, dummy return values, and mocked unit tests are strictly prohibited.
- **Verification:** Code audit and full execution of `cargo test` and `npm test` (`node --test`).

---

## 2. Core $N$-Dimensional Mathematics (`rust-core/src/math.rs`)

### REQ-MATH-001: Arbitrary $N$-Dimensional Vector Algebra (`VectorND`)
- **Description:** `VectorND` (`rust-core/src/math.rs`) shall represent vectors in $\mathbb{R}^N$ backed by `Vec<f64>` for any integer $N \ge 1$, supporting dimension-safe vector addition, subtraction, scalar multiplication, dot product, Euclidean norm, squared distance, normalization, linear interpolation (`lerp`), projection onto a $k$-dimensional subspace, and dimensional embedding/extension from $\mathbb{R}^k \to \mathbb{R}^N$ ($k \le N$).
- **Verification:** `rust-core/src/lib.rs` (`test_vector_nd_algebra_projections_and_embeddings`) verifies vector identities, Cauchy-Schwarz inequality, Parallelogram law, subspace projection, and dimensional embedding across $N \in \{2, 3, 4, 5, 7\}$.

### REQ-MATH-002: $N$-Dimensional Bivector / Plane Rotations (`RotorND`)
- **Description:** In $N$ dimensions, rotations occur in $\binom{N}{2} = \frac{N(N-1)}{2}$ independent 2D coordinate planes $(i, j)$ where $0 \le i < j < N$. `RotorND` (`rust-core/src/math.rs`) shall represent $N$-D orthogonal transformations ($SO(N)$ matrices) supporting elementary Givens/plane rotations $R_{i,j}(\theta)$, matrix composition, vector transformation, transpose/inversion, determinant ($\det(R) = +1$), and Modified Gram-Schmidt re-orthogonalization (`reorthogonalize()`).
- **Verification:** `rust-core/src/lib.rs` (`test_rotor_nd_planes_orthogonality_and_norm_preservation`) verifies that rotating any vector in $\mathbb{R}^N$ across arbitrary plane pairs $(i,j)$ preserves Euclidean norm $\|\vec{v}\|$ to within $10^{-11}$, satisfies $R^T R = I$, and has $\det(R) = +1$.

### REQ-MATH-003: Procedural $N$-Dimensional Polytope Generation (`PolytopeND`)
- **Description:** `PolytopeND` (`rust-core/src/math.rs`) shall procedurally generate exact vertex coordinates in $\mathbb{R}^k$, 1-skeleton edges, and $(k-1)$-dimensional facet vertex index lists (`facets`) for the three universal regular polytope families in any dimension $k \ge 1$, plus arbitrary $p$-gons in $k=2$:
  1. **2D Regular Polygons $\{p\}$ ($p \ge 3$):** $p$ vertices, $p$ edges.
  2. **$k$-Simplex ($\alpha_k$):** $k+1$ equidistant vertices in $\mathbb{R}^k$, $\binom{k+1}{2}$ edges, $k+1$ facets (each a $(k-1)$-simplex).
  3. **$k$-Hypercube ($\gamma_k$):** $2^k$ vertices $(\pm r, \dots, \pm r)$, $k \cdot 2^{k-1}$ edges, $2k$ facets (each a $(k-1)$-hypercube).
  4. **$k$-Orthoplex / Cross-Polytope ($\beta_k$):** $2k$ axis-aligned vertices, $2k(k-1)$ edges, $2^k$ facets (each a $(k-1)$-simplex).
- **Verification:** `rust-core/src/lib.rs` (`test_polytope_nd_families_across_dimensions`) verifies exact vertex counts, edge counts, facet counts, and edge-length regularity for $k = 2, 3, 4, 5, 6$.

### REQ-MATH-004: Hyperplane Cross-Section & Slicing Engine (`CrossSectionND`)
- **Description:** Given a $k$-dimensional polytope embedded in $\mathbb{R}^N$ and an orthogonal hyperplane slice defined by $x_d = s$ along dimension axis $d$, `CrossSectionND::slice_polytope` (`rust-core/src/math.rs`) shall compute the exact intersection points along all edges crossing $x_d = s$, returning the $(k-1)$-dimensional cross-section vertices, centroid, bounding radius, connected slice segments, and deduplicated 2D polygon contour (`ordered_polygon_2d`).
- **Verification:** `rust-core/src/lib.rs` (`test_cross_section_nd_slicing_3d_and_4d_polytopes`) and `tests/rust-wasm.test.mjs` verify that slicing a 3D Cube or Tetrahedron at $z=0$ yields a valid 2D polygon cross-section, slicing a 4D Tesseract at $w=0$ yields a valid cross-section, and slicing outside the polytope's extent yields an empty intersection.

---

## 3. Universe State, Space-Time Clock & Telemetry (`rust-core/src/spacetime.rs` & `rust-core/src/universe.rs`)

### REQ-CORE-001: Composable $N$-Dimensional Entity & Universe State (`DimensionXUniverse`)
- **Description:** `DimensionXUniverse` (`rust-core/src/universe.rs`) shall manage entity lifecycle (`BeingEntity`, `InformationPulseEntity`), deterministic ID allocation, component state (`position`, `velocity`, `orientation`, `polytope`, `active_dimension`, `max_dimension`, `phase_state`, `known_axioms`, `inherited_axioms`, `collaboration_links`, `energy`, `generation`, `worldline`), and deterministic system execution ordering on every fixed tick.
- **Verification:** `rust-core/src/lib.rs` and `tests/rust-wasm.test.mjs` verify entity spawning, seeding across 2D/3D/4D/5D/6D, and state snapshot serialization.

### REQ-CORE-002: Universal Space-Time Clock, Slow-Motion & Hyper-Warp Time Dilation (`SpaceTimeClock`)
- **Description:** `SpaceTimeClock` (`rust-core/src/spacetime.rs`, bridged via `wasm_api.rs` and `RustWasmBridge.js`) shall maintain Universal Proper Time $T$ (in seconds) with a base rate of $1.0\text{ simulation second per }1.0\text{ real second}$ (`time_scale = 1.0`) and fixed simulation quantum $\delta t$ (default $\frac{1}{60}\text{ s}$). It shall support:
  - Pausing (`time_scale = 0.0`), single-stepping (`step_single_tick()`), **slowing down proper time below $1\times$** (sub-real-time slow-motion: $0.05\times, 0.1\times, 0.25\times, 0.5\times$), and **accelerating proper time above $1\times$** ($2\times, 5\times, 10\times, 20\times, 50\times, 100\times, 250\times, 500\times, 1000\times$).
  - Dedicated **`-` (Slow Down Proper Time)** and **`+` (Speed Up Proper Time)** step controls (`slow_down_time()` / `speed_up_time()`) across `TIME_SCALE_LADDER` at the top of the UI and in the Rust core engine.
  - Deterministic accumulator stepping (`advance_real_time(real_delta_seconds)`) that returns the exact number of fixed $\delta t$ ticks executed and fractional interpolation alpha, advancing $T$ by $n \cdot \delta t$.
- **Verification:** `rust-core/src/lib.rs` (`test_spacetime_clock_base_slow_motion_hyper_warp_and_ladder`) and `tests/rust-wasm.test.mjs` verify that $1.0\text{ s}$ of real time at $0.25\times$ advances $T$ by $0.25\text{ s}$ (15 ticks), at $1\times$ advances $T$ by $1.0\text{ s}$ (60 ticks), at $10\times$ advances $T$ by $10.0\text{ s}$ (600 ticks), and that `+` / `-` controls step smoothly across slow-motion and hyper-warp scales.

### REQ-CORE-003: Space-Time Worldline Recording (`sample_worldlines`)
- **Description:** Every being's spatial trajectory through space-time `(universal_time, position, active_dimension)` shall be recorded at configurable proper-time intervals (`worldline_sample_interval`) into a bounded ring buffer (`worldline`, capped at `worldline_max_samples`), enabling space-time worldline visualization ($2\text{D space} + 1\text{D time} \to 3\text{D trajectory}$).
- **Verification:** `rust-core/src/lib.rs` and `tests/rust-wasm.test.mjs` verify chronological worldline sampling, ring-buffer bounding, and space-time coordinate projection.

### REQ-CORE-004: Domain Event Recording & Telemetry Snapshot (`SimulationEvent`, `to_snapshot_json`)
- **Description:** `DimensionXUniverse` (`rust-core/src/universe.rs`) shall record structured `SimulationEvent` entries (`INFO_INJECTED`, `AXIOM_ABSORBED`, `AXIOM_ACQUIRED`, `PEER_EXCHANGE`, `BEING_REPRODUCED`, `BEING_MUTATED`, `DIMENSIONAL_ASCENSION`, `DIMENSIONAL_PHASE_JUMP`) and serialize full JSON telemetry snapshots into WebAssembly linear memory via `wasm_api.rs`.
- **Verification:** `rust-core/src/lib.rs` (`test_snapshot_json_and_wasm_api_end_to_end`) and `tests/rust-wasm.test.mjs` verify event emission and WASM linear-memory snapshot decoding.

---

## 4. Simulation Systems: Beings, Information, Mutation & Ascension (`rust-core/src/universe.rs`)

### REQ-SYS-001: $N$-D Kinematics & Dimensional Confinement (`step_kinematics_nd`)
- **Description:** Beings with `active_dimension = k` (initially $k = 2$) shall move and rotate strictly within their active $k$-dimensional subspace (coordinates $x_0 \dots x_{k-1}$ active; coordinates $x_k \dots x_{N-1}$ held at their phase slice coordinate, $0$ for native $k$-D beings) with configurable boundary reflection or toroidal wrapping and angular velocity rotation across the $\binom{k}{2}$ active planes.
- **Verification:** `rust-core/src/lib.rs` (`test_system_1_kinematics_nd_confinement_and_boundaries`) verifies that a 2D being never drifts into $x_2, x_3, \dots$ until it ascends, reflects/wraps at boundaries, and preserves $SO(N)$ rotor orthogonality.

### REQ-SYS-002: Stochastic Higher-Dimensional Information Injection (`step_higher_dim_injection`)
- **Description:** Higher-dimensional beings ($k$-D, where $k > 2$) observing or intersecting a lower dimension $(k-1)$-D shall stochastically (at configurable interval `higher_dim_injection_interval` driven by a seeded PRNG) or on command (`trigger_random_injection`) inject an **Information Pulse** into the $(k-1)$-D hyperplane. The pulse expands radially at causal speed `pulse_wave_speed` in space-time and imparts a higher-dimensional geometric axiom fragment to lower-dimensional beings intersected by the wavefront.
- **Verification:** `rust-core/src/lib.rs` (`test_system_2_higher_dim_injection_and_wavefront_absorption`) verifies pulse emission from a 3D being, causal wavefront expansion over proper time $T$, and fragment acquisition by 2D beings upon wavefront contact.

### REQ-SYS-003: Peer-to-Peer Information Exchange (`step_peer_exchange`)
- **Description:** When two beings operating in the same active dimension $k$ (or when a higher-D being is phased into dimension $k$) come within communication radius `communication_radius`, they exchange geometric axioms that the other lacks, recording an active `collaboration_links` entry in space-time and gaining evolutionary energy.
- **Verification:** `rust-core/src/lib.rs` (`test_system_3_peer_exchange_and_links`) verifies that two beings holding disjoint axiom sets synchronize their axioms upon entering `communication_radius`.

### REQ-SYS-004: Reproduction, Inheritance & Geometric Mutation (`step_inheritance_mutation`)
- **Description:** Beings accumulate evolutionary energy over space-time and when exchanging information. Upon reaching `reproduction_threshold`, a being divides its energy and spawns an offspring in the same dimension that **inherits** the parent's `known_axioms` (`inherited_axioms`) and genetic traits. With configurable mutation probabilities (`mutation_probability`, `spontaneous_axiom_mutation_probability`), the offspring mutates its geometric structure (e.g., polygon sides/symmetry order) or spontaneously synthesizes a novel $(k+1)$-D geometric axiom.
- **Verification:** `rust-core/src/lib.rs` (`test_system_4_inheritance_and_mutation`) verifies parent-to-offspring axiom inheritance, lineage tracking, and deterministic geometric/epistemic mutation under a seeded PRNG.

### REQ-SYS-005: Collaborative Discovery & Polytope Folding Ascension (`step_collaborative_folding`)
- **Description:** When a group of collaborating beings within `folding_radius` in dimension $k$ collectively possesses the complete set of required geometric axioms for dimension $k+1$ (or when a solo mutated being holds all required axioms for $k+1$), they form a **Folding Constellation**, synthesize the $(k+1)$-dimensional polytope geometry (`PolytopeND` in $\mathbb{R}^{k+1}$), unlock dimension $k+1$ (`max_dimension = k + 1`, `active_dimension = k + 1`), and lift off along the newly unlocked orthogonal axis $x_k$.
- **Verification:** `rust-core/src/lib.rs` (`test_system_5_collaborative_folding_2d_to_3d_to_4d`) verifies that collaborating 2D beings fold into a 3D polyhedron and 3D beings fold into a 4D polychoron, updating their polytope geometry, active dimension, and orthogonal coordinates.

### REQ-SYS-006: Dimensional Phasing — Observing & Jumping Down (`step_dimensional_phasing`)
- **Description:** Any being that has unlocked dimension $M > 2$ (`max_dimension = M`) can:
  1. **Observe Lower Dimensions:** Hover at $x_{M-1} \ne 0$ above lower-dimensional hyperplanes, perceiving lower-dimensional beings within its projection cone (`PhaseState::Observing`).
  2. **Jump Down to Any Lower Dimension ($m < M$):** Transition its `active_dimension` to $m$ (`jump_being_to_dimension`), where it manifests as its mathematical $m$-D cross-section (`CrossSectionND::slice_polytope`) and can directly exchange information with native $m$-D beings (`PhaseState::PhasedDown`).
  3. **Ascend Back Up ($m \to M$):** Return from any lower dimension $m$ back to its native higher dimension $M$ after `phase_residence_time` or on command.
- **Verification:** `rust-core/src/lib.rs` (`test_system_6_dimensional_phasing_and_live_cross_section`) and `tests/rust-wasm.test.mjs` verify jumping from 3D/4D down to 2D, computing the live cross-section slice, interacting with 2D beings, and jumping back to 3D/4D.

---

## 5. Visualization & Space-Time Control Interface (`src/render/` & `src/ui/`)

### REQ-VIS-001: Cascaded $N\text{D} \to 3\text{D} \to 2\text{D}$ Projection Pipeline (`ProjectorND`)
- **Description:** `ProjectorND` (`src/render/ProjectorND.js`) shall project any $N$-dimensional vertex array $\vec{v} \in \mathbb{R}^N$ ($N \ge 2$) produced by the Rust WASM core down to 3D world coordinates $(X, Y, Z)$ and 2D screen coordinates $(u, v)$ via cascaded perspective/oblique projection, preserving exact 2D and 3D coordinates when $N \le 3$ and projecting $w, v, u \dots$ ($x_3 \dots x_{N-1}$) smoothly for $N \ge 4$.
- **Verification:** `tests/projector.test.mjs` verifies finite, deterministic projection coordinates for $N = 2, 3, 4, 5, 6$ (using vertices generated by the Rust WASM core) and camera orbit/zoom transformations.

### REQ-VIS-002: Dual 3D Space-Time & 2D Flatland Cross-Section Renderer (`SpaceTimeVisualizer`, `ControlHUD`)
- **Description:** The visualization layer (`src/render/SpaceTimeVisualizer.js`, `src/ui/ControlHUD.js`, `src/engine/RustWasmBridge.js`) shall render directly from the Rust WASM snapshot:
  1. **Primary 3D / $N$-D Omniscient Viewport (`canvas3D`):** Renders the 2D Flatland hyperplane ($z=0$), ascended 3D polyhedra ($z > 0$), 4D/ND projected hyper-polytopes (with translucent facets and $w$-depth spectral edge shading), expanding information injection light-cones, peer exchange beams, and **3D Space-Time Worldline corkscrew trails**.
  2. **2D Flatland Slice Viewport (`canvas2D`):** Renders the exact 2D hyperplane cross-section at slice coordinate $z_{\text{slice}}$ (`being.sliceAtZ` computed natively in Rust), showing native 2D polygons alongside the morphing 2D cross-sections of higher-dimensional beings intersecting the slice.
  3. **Space-Time & Dimensional HUD (`ControlHUD.js`):** Provides top-bar **`− Slow` (Slow Down Proper Time)** and **`+ Fast` (Speed Up Proper Time)** buttons alongside preset controls ($0\times$, $0.25\times$, $1\times$ [1 sec/sec], $5\times$, $20\times$, $100\times$, $500\times$), keyboard shortcuts (`-`/`+`), active dimension inspection ($2\text{D} \dots 6\text{D}$), manual Higher-D Information Injection, Dimensional Jump Up/Down triggers, and live population/event telemetry.
- **Verification:** `tests/projector.test.mjs` and `tests/rust-wasm.test.mjs` verify full 3D & 2D canvas rendering (`visualizer.render()`), HUD DOM updates (`hud.update()`), and end-to-end Rust WASM FFI integration.

### REQ-PERF-001: Bounded-Complexity Cross-Section Slicing, Link Deduplication & 60 FPS Frame Budget
- **Description:** As beings evolve from 2D through 6D over sustained proper-time runs, the simulation and visualization pipeline shall maintain bounded per-frame CPU and memory complexity:
  1. **Bounded $N$-D Cross-Section Slicing (`compute_being_cross_section`):** Slicing a $k$-D polytope ($k \ge 3$) down to a 2D hyperplane slice $x_2 = z_{\text{slice}}$ shall never construct iterated all-pairs complete graphs $K_V$ across intermediate dimensions. Instead, it shall either slice edges whose higher-dimensional coordinates $(x_3 \dots x_{k-1})$ lie within the slab thickness around $0$ or deduplicate intersection vertices and bound intermediate slice topology to $O(V)$ cyclic/nearest-neighbor hull edges, ensuring per-being slice time remains $< 0.1\text{ ms}$ even for 6D hypercubes ($64$ vertices, $192$ edges). Furthermore, `sliceAtZ` shall only be computed for beings whose $x_2$ bounding sphere actually intersects $z_{\text{slice}}$.
  2. **Deduplicated Collaboration Links (`collaboration_links`):** Both `step_peer_exchange` and `step_collaborative_folding` shall deduplicate `(partner_id, axiom)` entries by refreshing the timestamp of existing links rather than pushing duplicate links on every 60 Hz tick, and `step_collaborative_folding` shall check `target_pop >= max_pop_per_dim` before emitting folding links.
  3. **Bounded Telemetry & On-Demand Snapshot Payload (`to_snapshot_json`):** `ascension_log` shall be capped (last 64 entries), redundant duplicate JSON keys (`collaborationLinks`, `activeCrossSection`, `activePulses`) shall be eliminated in favor of canonical keys (`activeLinks`, `crossSection`, `pulses`), and full `worldline` history serialization shall be gated by an on-demand flag (`universe_set_include_worldlines`) so 60 FPS snapshots stay compact when 4D Worldline trails are toggled off.
  4. **Diff-Guarded HUD DOM Updates (`ControlHUD`):** `_updateInspectorPanel` and `_updateEventFeed` shall guard `innerHTML` rebuilders with state-signature checks so DOM subtrees and button event listeners are only rebuilt when the selected entity's state or latest event timestamp actually changes.
- **Verification:** `rust-core/src/lib.rs` (`test_sustained_proper_time_performance_and_bounded_complexity`) and `tests/rust-wasm.test.mjs` verify that running 600+ ticks ($10+\text{ s}$ of proper time with 4D/5D/6D beings) completes in $< 150\text{ ms}$ total with bounded link counts, bounded cross-section vertices, and compact snapshot JSON size.
