//! N-Dimensional Space-Time ECS Universe & 6 Core Simulation Systems (`DimensionXUniverse`).
//!
//! Implements:
//! - `REQ-SYS-001`: N-D Kinematics, `SO(k)` Bivector Plane Rotations & Boundary Confinement (`step_kinematics_nd`)
//! - `REQ-SYS-002`: Stochastic Higher-Dimensional Information Injection & Causal Wavefront Pulses (`step_higher_dim_injection`)
//! - `REQ-SYS-003`: Peer-to-Peer Epistemic Axiom Exchange & Space-Time Link Recording (`step_peer_exchange`)
//! - `REQ-SYS-004`: Evolutionary Energy, Reproduction, Inheritance & Geometric/Epistemic Mutation (`step_inheritance_mutation`)
//! - `REQ-SYS-005`: Collaborative Discovery & Polytope Folding Ascension (`step_collaborative_folding`)
//! - `REQ-SYS-006`: Dimensional Phasing — Observing, Live Hyperplane Slicing & Jumping Down/Up (`step_dimensional_phasing`)
//! - Worldline history sampling & zero-dependency JSON snapshot serialization (`to_snapshot_json`).
//!
//! 100% pure Rust standard library (`std`), zero external dependencies, MIT License.

use std::collections::{BTreeMap, BTreeSet};
use std::f64::consts::PI;

use crate::config::{BoundaryMode, SimulationConfig};
use crate::math::{
    CrossSectionData, CrossSectionND, PolytopeData, PolytopeFamily, PolytopeND, RotorND,
    SeededRandom, VectorND,
};
use crate::spacetime::{SpaceTimeClock, StepResult};

/// Dimensional phase state of a being in N-dimensional space-time (`REQ-SYS-006`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhaseState {
    Native,
    Observing,
    PhasedDown,
}

impl PhaseState {
    pub fn as_str(&self) -> &'static str {
        match self {
            PhaseState::Native => "NATIVE",
            PhaseState::Observing => "OBSERVING",
            PhaseState::PhasedDown => "PHASED_DOWN",
        }
    }
}

/// Complete N-dimensional being entity combining spatial, geometric, epistemic, genetic, phase, and worldline state.
#[derive(Clone, Debug)]
pub struct BeingEntity {
    pub id: u32,
    pub position: VectorND,
    pub velocity: VectorND,
    pub orientation: RotorND,
    pub angular_velocities: Vec<(usize, usize, f64)>,
    pub polytope: PolytopeData,
    pub radius: f64,
    pub color_hue: u32,
    pub sides: usize,
    pub active_dimension: usize,
    pub max_dimension: usize,
    pub phase_state: PhaseState,
    pub target_slice_dimension: usize,
    pub slice_coordinate: f64,
    pub phase_timer: f64,
    pub observed_entities: Vec<u32>,
    pub known_axioms: Vec<String>,
    pub inherited_axioms: Vec<String>,
    pub collaboration_links: Vec<(u32, String, f64)>,
    pub exchange_cooldown: f64,
    pub last_injection_time: f64,
    pub energy: f64,
    pub generation: u32,
    pub parent_id: Option<u32>,
    pub mutation_count: u32,
    pub symmetry_order: usize,
    pub speed_multiplier: f64,
    pub sensor_radius_multiplier: f64,
    pub fold_progress: f64,
    pub last_cross_section: Option<CrossSectionData>,
    pub worldline: Vec<(f64, Vec<f64>, usize)>,
    pub last_worldline_sample_time: f64,
    pub reorth_counter: u8,
}

impl BeingEntity {
    /// Updates an existing collaboration link with `partner_id` in-place, or appends a new link if `< 8` links exist.
    pub fn upsert_collaboration_link(&mut self, partner_id: u32, axiom: String, now: f64) {
        if let Some(entry) = self
            .collaboration_links
            .iter_mut()
            .find(|l| l.0 == partner_id)
        {
            entry.1 = axiom;
            entry.2 = now;
        } else if self.collaboration_links.len() < 8 {
            self.collaboration_links.push((partner_id, axiom, now));
        }
    }
}

/// Causal information wavefront pulse injected from a higher dimension into a lower hyperplane (`REQ-SYS-002`).
#[derive(Clone, Debug)]
pub struct InformationPulseEntity {
    pub id: u32,
    pub source_entity_id: Option<u32>,
    pub origin: VectorND,
    pub source_dimension: usize,
    pub target_dimension: usize,
    pub axiom_payload: String,
    pub wave_speed: f64,
    pub current_radius: f64,
    pub max_radius: f64,
    pub intensity: f64,
    pub hit_entities: Vec<u32>,
}

/// Structured space-time causal event emitted during simulation ticks.
#[derive(Clone, Debug)]
pub struct SimulationEvent {
    pub event_type: String,
    pub universal_time: f64,
    pub tick_count: u64,
    pub summary_json: String,
}

/// Complete N-Dimensional Space-Time Simulation Universe (`DimensionXUniverse`).
#[derive(Clone, Debug)]
pub struct DimensionXUniverse {
    pub config: SimulationConfig,
    pub clock: SpaceTimeClock,
    pub rng: SeededRandom,
    pub beings: Vec<BeingEntity>,
    pub pulses: Vec<InformationPulseEntity>,
    pub recent_events: Vec<SimulationEvent>,
    pub ascension_log: Vec<String>,
    pub max_unlocked_dimension: usize,
    pub next_entity_id: u32,
    pub viewer_slice_z: f64,
    pub include_worldlines_in_snapshot: bool,
}

impl Default for DimensionXUniverse {
    fn default() -> Self {
        Self::new(SimulationConfig::default())
    }
}

impl DimensionXUniverse {
    /// Creates a new `DimensionXUniverse` configured with `config`.
    pub fn new(config: SimulationConfig) -> Self {
        let clock = SpaceTimeClock::new(
            config.fixed_delta_time,
            config.initial_time_scale,
            config.max_sub_steps_per_frame,
        );
        let rng = SeededRandom::new(config.random_seed);
        let max_unlocked_dimension = config.initial_dimension.max(2);

        Self {
            config,
            clock,
            rng,
            beings: Vec::new(),
            pulses: Vec::new(),
            recent_events: Vec::new(),
            ascension_log: Vec::new(),
            max_unlocked_dimension,
            next_entity_id: 1,
            viewer_slice_z: 0.0,
            include_worldlines_in_snapshot: true,
        }
    }

    /// Sets the hyperplane slice Z coordinate (`x_2 = slice_z`) for 2D cross-section computation.
    pub fn set_viewer_slice_z(&mut self, slice_z: f64) {
        self.viewer_slice_z = if slice_z.is_finite() { slice_z } else { 0.0 };
    }

    /// Enables or disables serializing per-being worldline arrays in `to_snapshot_json()`.
    pub fn set_include_worldlines(&mut self, include: bool) {
        self.include_worldlines_in_snapshot = include;
    }

    fn allocate_entity_id(&mut self) -> u32 {
        let id = self.next_entity_id;
        self.next_entity_id = self.next_entity_id.wrapping_add(1).max(1);
        id
    }

    fn record_event(&mut self, event_type: &str, summary_json: String) {
        if self.recent_events.len() >= 120 {
            self.recent_events.remove(0);
        }
        self.recent_events.push(SimulationEvent {
            event_type: event_type.to_string(),
            universal_time: self.clock.universal_time,
            tick_count: self.clock.tick_count,
            summary_json,
        });
    }

    /// Spawns an N-dimensional being entity in dimension `dimension` (`2 <= dimension <= max_universe_dimension`).
    #[allow(clippy::too_many_arguments)]
    pub fn spawn_being(
        &mut self,
        dimension: usize,
        family: Option<PolytopeFamily>,
        sides: usize,
        position: Option<VectorND>,
        velocity: Option<VectorND>,
        known_axioms: Vec<String>,
        generation: u32,
        parent_id: Option<u32>,
        energy: f64,
    ) -> u32 {
        let ambient_dim = self.config.max_universe_dimension.max(2);
        let effective_dim = dimension.clamp(2, ambient_dim);
        let symmetry_order = sides.max(3);

        let resolved_family = match family {
            Some(PolytopeFamily::Polygon) if effective_dim > 2 => {
                if symmetry_order == 4 {
                    PolytopeFamily::Hypercube
                } else if symmetry_order >= 5 {
                    PolytopeFamily::Orthoplex
                } else {
                    PolytopeFamily::Simplex
                }
            }
            Some(f) => f,
            None => {
                if effective_dim == 2 {
                    PolytopeFamily::Polygon
                } else if symmetry_order == 4 {
                    PolytopeFamily::Hypercube
                } else if symmetry_order >= 5 {
                    PolytopeFamily::Orthoplex
                } else {
                    PolytopeFamily::Simplex
                }
            }
        };

        let base_radius = if effective_dim == 2 { 1.2 } else { 1.45 };
        let polytope = PolytopeND::generate_by_family(
            resolved_family,
            effective_dim,
            symmetry_order,
            base_radius,
            ambient_dim,
        );

        let mut pos_vec = VectorND::zero(ambient_dim);
        if let Some(p) = position {
            let limit = effective_dim.min(p.dim()).min(ambient_dim);
            for d in 0..limit {
                pos_vec.set(d, p.get(d));
            }
        } else {
            let spawn_rad = (self.config.world_bounds_radius * 0.3).min(9.5);
            let rand_dir = VectorND::random_unit(ambient_dim, &mut self.rng, effective_dim);
            let r = self.rng.range(1.0, spawn_rad);
            for d in 0..effective_dim {
                pos_vec.set(d, rand_dir.get(d) * r);
            }
            if effective_dim > 2 {
                pos_vec.set(effective_dim - 1, self.rng.range(3.5, 7.5));
            }
        }

        let mut vel_vec = VectorND::zero(ambient_dim);
        if let Some(v) = velocity {
            let limit = effective_dim.min(v.dim()).min(ambient_dim);
            for d in 0..limit {
                vel_vec.set(d, v.get(d));
            }
        } else {
            let vel_dir = VectorND::random_unit(ambient_dim, &mut self.rng, effective_dim);
            let speed = self.rng.range(2.8, 5.8);
            for d in 0..effective_dim {
                vel_vec.set(d, vel_dir.get(d) * speed);
            }
        }

        for d in effective_dim..ambient_dim {
            pos_vec.set(d, 0.0);
            vel_vec.set(d, 0.0);
        }

        let mut angular_velocities = Vec::new();
        for i in 0..effective_dim {
            for j in (i + 1)..effective_dim {
                if i == 0 || j == effective_dim - 1 {
                    let mut omega = self.rng.range(-1.1, 1.1);
                    if omega.abs() < 0.05 {
                        omega = 0.65;
                    }
                    angular_velocities.push((i, j, omega));
                }
            }
        }

        let mut dedup_axioms: Vec<String> = Vec::new();
        for ax in known_axioms {
            if !dedup_axioms.contains(&ax) {
                dedup_axioms.push(ax);
            }
        }

        if effective_dim > 2 && dedup_axioms.is_empty() {
            for d in 3..=effective_dim {
                for ax in self.config.get_axioms_for_dimension(d) {
                    if !dedup_axioms.contains(&ax) {
                        dedup_axioms.push(ax);
                    }
                }
            }
            if effective_dim + 1 <= ambient_dim {
                let next_axioms = self.config.get_axioms_for_dimension(effective_dim + 1);
                if !next_axioms.is_empty() {
                    let idx = self.rng.int(0, (next_axioms.len() - 1) as i32) as usize;
                    dedup_axioms.push(next_axioms[idx].clone());
                }
            }
        }

        let id = self.allocate_entity_id();
        let color_hue =
            ((effective_dim as u32) * 68 + (symmetry_order as u32) * 23 + id * 17) % 360;
        let initial_jitter = self
            .rng
            .range(0.0, self.config.higher_dim_injection_interval * 0.15);
        let now = self.clock.universal_time;
        let poly_sides = polytope.sides;
        let poly_family_str = polytope.family.as_str();

        if effective_dim > self.max_unlocked_dimension {
            self.max_unlocked_dimension = effective_dim;
        }

        let inherited_axioms = if parent_id.is_some() {
            dedup_axioms.clone()
        } else {
            Vec::new()
        };

        let being = BeingEntity {
            id,
            position: pos_vec.clone(),
            velocity: vel_vec,
            orientation: RotorND::identity(ambient_dim),
            angular_velocities,
            polytope,
            radius: base_radius,
            color_hue,
            sides: poly_sides,
            active_dimension: effective_dim,
            max_dimension: effective_dim,
            phase_state: PhaseState::Native,
            target_slice_dimension: if effective_dim > 2 {
                effective_dim - 1
            } else {
                2
            },
            slice_coordinate: 0.0,
            phase_timer: self.config.phasing_interval,
            observed_entities: Vec::new(),
            known_axioms: dedup_axioms,
            inherited_axioms,
            collaboration_links: Vec::new(),
            exchange_cooldown: 0.0,
            last_injection_time: now - initial_jitter,
            energy,
            generation,
            parent_id,
            mutation_count: 0,
            symmetry_order,
            speed_multiplier: 1.0,
            sensor_radius_multiplier: 1.0,
            fold_progress: 0.0,
            last_cross_section: None,
            worldline: vec![(now, pos_vec.data.clone(), effective_dim)],
            last_worldline_sample_time: now,
            reorth_counter: 0,
        };

        self.beings.push(being);

        let parent_json = match parent_id {
            Some(pid) => pid.to_string(),
            None => "null".to_string(),
        };
        self.record_event(
            "BEING_SPAWNED",
            format!(
                "{{\"entityId\":{},\"dimension\":{},\"family\":\"{}\",\"sides\":{},\"generation\":{},\"parentId\":{}}}",
                id, effective_dim, poly_family_str, poly_sides, generation, parent_json
            ),
        );

        id
    }

    /// Seeds the primordial universe population across 2D, 3D, and 4D.
    pub fn seed_initial_population(
        &mut self,
        count_2d: usize,
        count_3d: usize,
        count_4d: usize,
    ) -> Vec<u32> {
        let mut spawned = Vec::new();
        let ambient_dim = self.config.max_universe_dimension.max(2);
        let dim3_axioms = if ambient_dim >= 3 {
            self.config.get_axioms_for_dimension(3)
        } else {
            Vec::new()
        };
        let dim4_axioms = if ambient_dim >= 4 {
            self.config.get_axioms_for_dimension(4)
        } else {
            Vec::new()
        };

        let ring_radius = (self.config.world_bounds_radius * 0.25).min(7.5);
        for i in 0..count_2d {
            let sides = 3 + (i % 4);
            let angle = (2.0 * PI * (i as f64)) / (count_2d.max(1) as f64);
            let r = self.rng.range(1.5, ring_radius);
            let mut pos = VectorND::zero(ambient_dim);
            pos.set(0, r * angle.cos());
            pos.set(1, r * angle.sin());

            let mut vel = VectorND::zero(ambient_dim);
            let speed = self.rng.range(3.0, 5.5);
            vel.set(
                0,
                -angle.cos() * speed * 0.65 - angle.sin() * speed * 0.35,
            );
            vel.set(
                1,
                -angle.sin() * speed * 0.65 + angle.cos() * speed * 0.35,
            );

            let seed_axioms = if !dim3_axioms.is_empty() {
                vec![dim3_axioms[i % dim3_axioms.len()].clone()]
            } else {
                Vec::new()
            };
            let energy = self.rng.range(35.0, 75.0);
            let id = self.spawn_being(
                2,
                Some(PolytopeFamily::Polygon),
                sides,
                Some(pos),
                Some(vel),
                seed_axioms,
                1,
                None,
                energy,
            );
            spawned.push(id);
        }

        let families = [
            PolytopeFamily::Simplex,
            PolytopeFamily::Hypercube,
            PolytopeFamily::Orthoplex,
        ];

        if ambient_dim >= 3 {
            for i in 0..count_3d {
                let family = families[i % families.len()];
                let mut seed_axioms = dim3_axioms.clone();
                if !dim4_axioms.is_empty() {
                    seed_axioms.push(dim4_axioms[i % dim4_axioms.len()].clone());
                }
                let id = self.spawn_being(
                    3,
                    Some(family),
                    4 + (i % 3),
                    None,
                    None,
                    seed_axioms,
                    1,
                    None,
                    50.0,
                );
                spawned.push(id);
            }
        }

        if ambient_dim >= 4 {
            for i in 0..count_4d {
                let family = families[i % families.len()];
                let id = self.spawn_being(
                    4,
                    Some(family),
                    5,
                    None,
                    None,
                    Vec::new(),
                    1,
                    None,
                    60.0,
                );
                spawned.push(id);
            }
        }

        spawned
    }

    /// Executes a single fixed simulation tick (`dt = fixed_delta_time`) across all 6 core N-D systems.
    fn execute_single_tick(&mut self, dt: f64) {
        self.step_kinematics_nd(dt);
        self.step_higher_dim_injection(dt);
        self.step_peer_exchange(dt);
        self.step_inheritance_mutation(dt);
        self.step_collaborative_folding(dt);
        self.step_dimensional_phasing(dt);
        self.sample_worldlines();
    }

    /// Advances the simulation clock by `real_dt_seconds` (scaled by `clock.time_scale`) and runs all resulting sub-ticks.
    pub fn step(&mut self, real_dt_seconds: f64) -> StepResult {
        if !real_dt_seconds.is_finite()
            || real_dt_seconds <= 0.0
            || self.clock.paused
            || self.clock.time_scale <= 0.0
        {
            return self.clock.advance_real_time(0.0, |_, _, _| {});
        }

        self.clock.accumulator += real_dt_seconds * self.clock.time_scale;
        let raw_ticks =
            ((self.clock.accumulator + 1e-11) / self.clock.fixed_delta_time).floor() as usize;
        let ticks_to_run = raw_ticks.min(self.clock.max_sub_steps_per_frame);

        if ticks_to_run > 0 {
            self.clock.accumulator = (self.clock.accumulator
                - (ticks_to_run as f64) * self.clock.fixed_delta_time)
                .max(0.0);
            let dt = self.clock.fixed_delta_time;
            for _ in 0..ticks_to_run {
                self.clock.tick_count += 1;
                self.clock.universal_time = (self.clock.tick_count as f64) * dt;
                self.execute_single_tick(dt);
            }
        }

        let interpolation_alpha =
            (self.clock.accumulator / self.clock.fixed_delta_time).clamp(0.0, 1.0);
        StepResult {
            ticks_executed: ticks_to_run,
            delta_time: self.clock.fixed_delta_time,
            simulated_time_advanced: (ticks_to_run as f64) * self.clock.fixed_delta_time,
            universal_time: self.clock.universal_time,
            tick_count: self.clock.tick_count,
            interpolation_alpha,
        }
    }

    /// Synchronously executes `num_ticks` fixed simulation ticks regardless of whether the clock is paused.
    pub fn step_ticks(&mut self, num_ticks: usize) -> StepResult {
        let dt = self.clock.fixed_delta_time;
        for _ in 0..num_ticks {
            self.clock.tick_count += 1;
            self.clock.universal_time = (self.clock.tick_count as f64) * dt;
            self.execute_single_tick(dt);
        }
        let interpolation_alpha = (self.clock.accumulator / dt).clamp(0.0, 1.0);
        StepResult {
            ticks_executed: num_ticks,
            delta_time: dt,
            simulated_time_advanced: (num_ticks as f64) * dt,
            universal_time: self.clock.universal_time,
            tick_count: self.clock.tick_count,
            interpolation_alpha,
        }
    }

    /// Sets the universal space-time speed multiplier (`0.0` = paused, `< 1.0` = slow-motion, `1.0` = 1s/s, `> 1.0` = hyper-warp).
    pub fn set_time_scale(&mut self, scale: f64) -> f64 {
        self.clock.set_time_scale(scale);
        let ts = self.clock.time_scale;
        let paused = self.clock.paused;
        self.record_event(
            "TIME_SCALE_CHANGED",
            format!("{{\"timeScale\":{:.4},\"paused\":{}}}", ts, paused),
        );
        ts
    }

    /// **`-` control:** Slows down Universal Proper Time to the next lower step in `TIME_SCALE_LADDER`.
    pub fn slow_down_time(&mut self) -> f64 {
        let next = self.clock.slow_down_time();
        let paused = self.clock.paused;
        self.record_event(
            "TIME_SCALE_CHANGED",
            format!(
                "{{\"timeScale\":{:.4},\"paused\":{},\"direction\":\"SLOW_DOWN\"}}",
                next, paused
            ),
        );
        next
    }

    /// **`+` control:** Speeds up Universal Proper Time to the next higher step in `TIME_SCALE_LADDER`.
    pub fn speed_up_time(&mut self) -> f64 {
        let next = self.clock.speed_up_time();
        let paused = self.clock.paused;
        self.record_event(
            "TIME_SCALE_CHANGED",
            format!(
                "{{\"timeScale\":{:.4},\"paused\":{},\"direction\":\"SPEED_UP\"}}",
                next, paused
            ),
        );
        next
    }

    // ========================================================================
    // System 1: REQ-SYS-001 — N-D Kinematics, SO(k) Rotations & Confinement
    // ========================================================================

    /// Integrates N-dimensional translation, `SO(k)` plane rotations, Gram-Schmidt reorthogonalization,
    /// dimensional subspace confinement, and spherical reflective/toroidal boundary enforcement.
    pub fn step_kinematics_nd(&mut self, dt: f64) {
        if dt <= 0.0 {
            return;
        }
        let max_dim = self.config.max_universe_dimension.max(2);
        let bounds_radius = self.config.world_bounds_radius;
        let bounds_radius_sq = bounds_radius * bounds_radius;
        let boundary_mode = self.config.boundary_mode;

        for being in &mut self.beings {
            if being.position.dim() < max_dim {
                being.position = being.position.embed_in_dimension(max_dim);
            }
            if being.velocity.dim() < max_dim {
                being.velocity = being.velocity.embed_in_dimension(max_dim);
            }
            if being.orientation.dim < max_dim {
                being.orientation = being.orientation.embed_in_dimension(max_dim);
            }

            let vec_dim = being.position.dim();
            let active_dim = being.active_dimension.clamp(1, vec_dim);
            let effective_dt = dt * being.speed_multiplier;

            // 1. Integrate active coordinates 0 .. active_dim
            for axis in 0..active_dim {
                let next_coord =
                    being.position.get(axis) + being.velocity.get(axis) * effective_dt;
                being.position.set(axis, next_coord);
            }

            // 2. Enforce dimensional confinement on orthogonal axes >= active_dim
            let slice_coord = if being.phase_state == PhaseState::PhasedDown
                && being.slice_coordinate.is_finite()
            {
                being.slice_coordinate
            } else {
                0.0
            };

            for axis in active_dim..vec_dim {
                being
                    .position
                    .set(axis, if axis == active_dim { slice_coord } else { 0.0 });
                being.velocity.set(axis, 0.0);
            }

            // 3. Spherical boundary confinement (Reflective or Toroidal) in active k-D subspace
            let mut active_norm_sq = 0.0;
            for axis in 0..active_dim {
                let c = being.position.get(axis);
                active_norm_sq += c * c;
            }

            if active_norm_sq > bounds_radius_sq && active_norm_sq > 1e-24 {
                let active_norm = active_norm_sq.sqrt();
                let inv_norm = 1.0 / active_norm;

                match boundary_mode {
                    BoundaryMode::Toroidal => {
                        for axis in 0..active_dim {
                            let n_axis = being.position.get(axis) * inv_norm;
                            being.position.set(axis, -n_axis * (bounds_radius * 0.98));
                        }
                    }
                    BoundaryMode::Reflective => {
                        let mut v_dot_n = 0.0;
                        for axis in 0..active_dim {
                            let n_axis = being.position.get(axis) * inv_norm;
                            v_dot_n += being.velocity.get(axis) * n_axis;
                        }
                        for axis in 0..active_dim {
                            let n_axis = being.position.get(axis) * inv_norm;
                            being.position.set(axis, n_axis * bounds_radius);
                            if v_dot_n > 0.0 {
                                let reflected =
                                    being.velocity.get(axis) - 2.0 * v_dot_n * n_axis;
                                being.velocity.set(axis, reflected);
                            }
                        }
                    }
                }
            }

            // 4. Bivector plane rotations in active subspace (0 <= axis_i < axis_j < active_dim)
            for &(axis_i, axis_j, omega) in &being.angular_velocities {
                if axis_i < axis_j && axis_j < active_dim && omega.is_finite() && omega != 0.0 {
                    being
                        .orientation
                        .rotate_plane_in_place(axis_i, axis_j, omega * dt);
                }
            }

            being.reorth_counter = being.reorth_counter.wrapping_add(1) & 15;
            if being.reorth_counter == 0 {
                being.orientation.reorthogonalize();
            }
        }
    }

    // ========================================================================
    // System 2: REQ-SYS-002 — Higher-Dimensional Information Injection
    // ========================================================================

    /// Injects an `InformationPulseEntity` into `target_dim` from `source_dim`.
    pub fn inject_pulse(
        &mut self,
        source_entity_id: Option<u32>,
        source_dim: usize,
        target_dim: usize,
        origin: VectorND,
        axiom: Option<String>,
    ) -> u32 {
        let max_dim = self.config.max_universe_dimension.max(2);
        let t_dim = target_dim.clamp(2, max_dim);
        let s_dim = source_dim.max(t_dim + 1);

        let mut origin_vec = VectorND::zero(max_dim);
        let limit = t_dim.min(origin.dim()).min(max_dim);
        for d in 0..limit {
            origin_vec.set(d, origin.get(d));
        }

        let selected_axiom = match axiom {
            Some(a) if !a.is_empty() => a,
            _ => {
                let unlock_dim = (t_dim + 1).max(3);
                let candidates = self.config.get_axioms_for_dimension(unlock_dim);
                let idx = self.rng.int(0, (candidates.len() - 1) as i32) as usize;
                candidates[idx].clone()
            }
        };

        let pulse_id = self.allocate_entity_id();
        let pulse = InformationPulseEntity {
            id: pulse_id,
            source_entity_id,
            origin: origin_vec,
            source_dimension: s_dim,
            target_dimension: t_dim,
            axiom_payload: selected_axiom.clone(),
            wave_speed: self.config.pulse_wave_speed,
            current_radius: 0.0,
            max_radius: self.config.pulse_max_radius,
            intensity: 1.0,
            hit_entities: Vec::new(),
        };
        self.pulses.push(pulse);

        let src_json = match source_entity_id {
            Some(sid) => sid.to_string(),
            None => "-1".to_string(),
        };
        self.record_event(
            "INFO_INJECTED",
            format!(
                "{{\"pulseEntityId\":{},\"sourceEntityId\":{},\"sourceDimension\":{},\"targetDimension\":{},\"axiom\":\"{}\"}}",
                pulse_id, src_json, s_dim, t_dim, selected_axiom
            ),
        );

        pulse_id
    }

    /// Manually triggers a random higher-dimensional information pulse from `source_dim` into `target_dim`.
    pub fn trigger_random_injection(
        &mut self,
        source_dim: usize,
        target_dim: usize,
    ) -> Option<u32> {
        let max_dim = self.config.max_universe_dimension.max(2);
        let t_dim = target_dim.clamp(2, max_dim.saturating_sub(1).max(2));
        let s_dim = source_dim.max(t_dim + 1);

        let r = self
            .rng
            .range(0.0, (self.config.world_bounds_radius * 0.25).min(8.0));
        let dir = VectorND::random_unit(max_dim, &mut self.rng, t_dim);
        let mut origin = VectorND::zero(max_dim);
        for d in 0..t_dim {
            origin.set(d, dir.get(d) * r);
        }

        Some(self.inject_pulse(None, s_dim, t_dim, origin, None))
    }

    /// Executes one tick of higher-dimensional pulse emission and causal wavefront expansion (`REQ-SYS-002`).
    pub fn step_higher_dim_injection(&mut self, dt: f64) {
        if dt <= 0.0 {
            return;
        }
        let now = self.clock.universal_time;
        let max_concurrent = self.config.max_concurrent_pulses;

        // Collect emissions from higher-D beings (M >= 3) only when their injection interval has elapsed
        let mut pending_emissions: Vec<(u32, usize, usize, VectorND, String)> = Vec::new();
        let mut current_pulse_count = self.pulses.len();
        let n_beings = self.beings.len();

        for i in 0..n_beings {
            let emitter_high_dim =
                self.beings[i].active_dimension.max(self.beings[i].max_dimension);
            if emitter_high_dim <= 2 {
                continue;
            }
            let elapsed = now - self.beings[i].last_injection_time;
            if elapsed < self.config.higher_dim_injection_interval {
                continue;
            }

            if current_pulse_count >= max_concurrent {
                self.beings[i].last_injection_time = now;
                continue;
            }

            let target_dim = if self.beings[i].target_slice_dimension >= 2
                && self.beings[i].target_slice_dimension < emitter_high_dim
            {
                self.beings[i].target_slice_dimension
            } else {
                emitter_high_dim - 1
            };

            let unlock_dim = target_dim + 1;
            let required = self.config.get_axioms_for_dimension(unlock_dim);
            let matching: Vec<String> = self.beings[i]
                .known_axioms
                .iter()
                .filter(|ax| required.contains(ax))
                .cloned()
                .collect();
            let candidate_pool = if !matching.is_empty() {
                matching
            } else {
                required
            };

            let mut missing: Vec<String> = Vec::new();
            for ax in &candidate_pool {
                let any_missing = self
                    .beings
                    .iter()
                    .any(|other| other.active_dimension == target_dim && !other.known_axioms.contains(ax));
                if any_missing {
                    missing.push(ax.clone());
                }
            }

            let pool = if !missing.is_empty() {
                missing
            } else {
                candidate_pool
            };
            let idx = self.rng.int(0, (pool.len() - 1) as i32) as usize;
            let chosen_axiom = pool[idx].clone();

            pending_emissions.push((
                self.beings[i].id,
                emitter_high_dim,
                target_dim,
                self.beings[i].position.clone(),
                chosen_axiom,
            ));
            current_pulse_count += 1;

            let next_jitter = self.rng.range(
                -self.config.higher_dim_injection_interval * 0.1,
                self.config.higher_dim_injection_interval * 0.1,
            );
            self.beings[i].last_injection_time = now + next_jitter;
        }

        for (src_id, s_dim, t_dim, origin, axiom) in pending_emissions {
            self.inject_pulse(Some(src_id), s_dim, t_dim, origin, Some(axiom));
        }

        // Expand wavefronts and imprint axioms on lower-D beings
        let energy_boost = self.config.energy_gain_per_exchange * 0.5;
        let mut absorbed_events: Vec<String> = Vec::new();

        for pulse in &mut self.pulses {
            pulse.current_radius += pulse.wave_speed * dt;
            let progress = (pulse.current_radius / pulse.max_radius.max(1e-6)).clamp(0.0, 1.0);
            pulse.intensity = 1.0 - progress;

            let radius_sq = pulse.current_radius * pulse.current_radius;
            let target_dim = pulse.target_dimension;

            for b in &mut self.beings {
                if Some(b.id) == pulse.source_entity_id {
                    continue;
                }
                if b.active_dimension != target_dim {
                    continue;
                }
                if pulse.hit_entities.contains(&b.id) {
                    continue;
                }

                let mut dist_sq = 0.0;
                for axis in 0..target_dim {
                    let diff = b.position.get(axis) - pulse.origin.get(axis);
                    dist_sq += diff * diff;
                }

                if dist_sq <= radius_sq {
                    pulse.hit_entities.push(b.id);
                    let newly_discovered = !b.known_axioms.contains(&pulse.axiom_payload);
                    if newly_discovered {
                        b.known_axioms.push(pulse.axiom_payload.clone());
                        b.energy += energy_boost;
                        absorbed_events.push(format!(
                            "{{\"entityId\":{},\"pulseEntityId\":{},\"axiom\":\"{}\",\"dimension\":{},\"newlyDiscovered\":true}}",
                            b.id, pulse.id, pulse.axiom_payload, target_dim
                        ));
                    } else {
                        b.energy += 0.5;
                    }
                }
            }
        }

        self.pulses
            .retain(|p| p.current_radius < p.max_radius);

        for ev in absorbed_events {
            self.record_event("AXIOM_ABSORBED", ev);
        }
    }

    // ========================================================================
    // System 3: REQ-SYS-003 — Peer-to-Peer Epistemic Axiom Exchange
    // ========================================================================

    /// Synchronizes geometric axioms and records space-time communication links between peers
    /// within `communication_radius` in the same active dimension.
    pub fn step_peer_exchange(&mut self, dt: f64) {
        if dt <= 0.0 {
            return;
        }
        let now = self.clock.universal_time;
        let link_expiry = self.config.link_expiration_seconds;

        // 1. Update cooldowns, prune expired links, and ensure phased-down visitors hold (m + 1) axioms
        for b in &mut self.beings {
            if b.exchange_cooldown > 0.0 {
                b.exchange_cooldown = (b.exchange_cooldown - dt).max(0.0);
            }
            if !b.collaboration_links.is_empty() {
                b.collaboration_links
                    .retain(|&(_, _, ts)| now - ts <= link_expiry);
            }

            if b.max_dimension > b.active_dimension && b.known_axioms.is_empty() {
                b.known_axioms = self.config.get_axioms_for_dimension(b.active_dimension + 1);
            }
        }

        // 2. Pairwise proximity check within each active_dimension
        let base_comm_radius = self.config.communication_radius;
        let n = self.beings.len();
        let mut exchange_events: Vec<String> = Vec::new();

        for i in 0..n {
            for j in (i + 1)..n {
                if self.beings[i].active_dimension != self.beings[j].active_dimension {
                    continue;
                }
                if self.beings[i].exchange_cooldown > 0.0
                    && self.beings[j].exchange_cooldown > 0.0
                {
                    continue;
                }

                let active_dim = self.beings[i].active_dimension;
                let sensor_mult = self.beings[i]
                    .sensor_radius_multiplier
                    .max(self.beings[j].sensor_radius_multiplier);
                let eff_radius = base_comm_radius * sensor_mult;
                let eff_radius_sq = eff_radius * eff_radius;

                let mut dist_sq = 0.0;
                for axis in 0..active_dim {
                    let diff = self.beings[i].position.get(axis)
                        - self.beings[j].position.get(axis);
                    dist_sq += diff * diff;
                }

                if dist_sq > eff_radius_sq {
                    continue;
                }

                // Fast check for complementary axioms before allocating vectors
                let has_i_for_j = self.beings[i]
                    .known_axioms
                    .iter()
                    .any(|ax| !self.beings[j].known_axioms.contains(ax));
                let has_j_for_i = self.beings[j]
                    .known_axioms
                    .iter()
                    .any(|ax| !self.beings[i].known_axioms.contains(ax));

                if !has_i_for_j && !has_j_for_i {
                    self.beings[i].exchange_cooldown = self.config.exchange_cooldown;
                    self.beings[j].exchange_cooldown = self.config.exchange_cooldown;
                    continue;
                }

                let missing_for_j: Vec<String> = if has_i_for_j {
                    self.beings[i]
                        .known_axioms
                        .iter()
                        .filter(|ax| !self.beings[j].known_axioms.contains(ax))
                        .cloned()
                        .collect()
                } else {
                    Vec::new()
                };
                let missing_for_i: Vec<String> = if has_j_for_i {
                    self.beings[j]
                        .known_axioms
                        .iter()
                        .filter(|ax| !self.beings[i].known_axioms.contains(ax))
                        .cloned()
                        .collect()
                } else {
                    Vec::new()
                };

                for ax in &missing_for_j {
                    self.beings[j].known_axioms.push(ax.clone());
                }
                for ax in &missing_for_i {
                    self.beings[i].known_axioms.push(ax.clone());
                }

                let primary_axiom = missing_for_j
                    .first()
                    .or_else(|| missing_for_i.first())
                    .cloned()
                    .unwrap_or_else(|| "SYMMETRY_RESONANCE".to_string());

                let id_i = self.beings[i].id;
                let id_j = self.beings[j].id;

                self.beings[i].upsert_collaboration_link(id_j, primary_axiom.clone(), now);
                self.beings[j].upsert_collaboration_link(id_i, primary_axiom.clone(), now);

                self.beings[i].energy += self.config.energy_gain_per_exchange;
                self.beings[j].energy += self.config.energy_gain_per_exchange;
                self.beings[i].exchange_cooldown = self.config.exchange_cooldown;
                self.beings[j].exchange_cooldown = self.config.exchange_cooldown;

                exchange_events.push(format!(
                    "{{\"entityA\":{},\"entityB\":{},\"dimension\":{},\"axiom\":\"{}\"}}",
                    id_i, id_j, active_dim, primary_axiom
                ));
            }
        }

        for ev in exchange_events {
            self.record_event("AXIOM_ACQUIRED", ev);
        }
    }

    // ========================================================================
    // System 4: REQ-SYS-004 — Reproduction, Inheritance & Geometric Mutation
    // ========================================================================

    /// Accumulates evolutionary energy, divides energy to spawn offspring inheriting `known_axioms`,
    /// and applies stochastic geometric/epistemic mutations (including solo mutation ascension).
    pub fn step_inheritance_mutation(&mut self, dt: f64) {
        if dt <= 0.0 {
            return;
        }
        let now = self.clock.universal_time;
        let max_universe_dim = self.config.max_universe_dimension.max(2);
        let max_pop_per_dim = self.config.max_population_per_dimension;
        let max_total_beings = max_pop_per_dim * (max_universe_dim - 1).max(1);

        let mut pop_by_dim: BTreeMap<usize, usize> = BTreeMap::new();
        for b in &self.beings {
            *pop_by_dim.entry(b.active_dimension).or_insert(0) += 1;
            if b.max_dimension != b.active_dimension {
                *pop_by_dim.entry(b.max_dimension).or_insert(0) += 1;
            }
        }

        let mut total_beings = self.beings.len();
        let initial_len = self.beings.len();

        for idx in 0..initial_len {
            self.beings[idx].energy += self.config.energy_gain_per_second * dt;

            let active_dim = self.beings[idx].active_dimension;
            let max_dim = self.beings[idx].max_dimension;
            let current_pop = pop_by_dim
                .get(&active_dim)
                .copied()
                .unwrap_or(0)
                .max(pop_by_dim.get(&max_dim).copied().unwrap_or(0));

            if self.beings[idx].energy < self.config.reproduction_threshold
                || current_pop >= max_pop_per_dim
                || total_beings >= max_total_beings
            {
                continue;
            }

            // Divide parent energy in half for reproduction
            let offspring_energy = (self.beings[idx].energy * 0.5).max(25.0);
            self.beings[idx].energy *= 0.5;

            *pop_by_dim.entry(active_dim).or_insert(0) += 1;
            if max_dim != active_dim {
                *pop_by_dim.entry(max_dim).or_insert(0) += 1;
            }
            total_beings += 1;

            let parent_id = self.beings[idx].id;
            let parent_pos = self.beings[idx].position.clone();
            let inherited_axioms = self.beings[idx].known_axioms.clone();
            let inherited_gen = self.beings[idx].generation + 1;
            let inherited_symmetry = self.beings[idx].symmetry_order;
            let inherited_family = self.beings[idx].polytope.family;
            let inherited_sides = self.beings[idx].sides;
            let parent_speed_mult = self.beings[idx].speed_multiplier;
            let parent_sensor_mult = self.beings[idx].sensor_radius_multiplier;
            let parent_mut_count = self.beings[idx].mutation_count;
            let parent_hue = self.beings[idx].color_hue;

            let offset_dir = VectorND::random_unit(max_universe_dim, &mut self.rng, max_dim);
            let mut child_pos = parent_pos.add_scaled(&offset_dir, 1.5);
            if max_dim > active_dim && child_pos.get(max_dim - 1).abs() < 1.0 {
                child_pos.set(max_dim - 1, self.rng.range(3.0, 6.5));
            }
            for d in max_dim..max_universe_dim {
                child_pos.set(d, 0.0);
            }

            let child_vel_dir =
                VectorND::random_unit(max_universe_dim, &mut self.rng, max_dim);
            let base_speed = self.rng.range(2.5, 5.5);
            let child_vel = child_vel_dir.scale(base_speed);

            let child_id = self.spawn_being(
                max_dim,
                Some(inherited_family),
                inherited_sides,
                Some(child_pos),
                Some(child_vel),
                inherited_axioms.clone(),
                inherited_gen,
                Some(parent_id),
                offspring_energy,
            );

            let child_idx = self.beings.len() - 1;
            self.beings[child_idx].symmetry_order = inherited_symmetry;
            self.beings[child_idx].speed_multiplier = parent_speed_mult;
            self.beings[child_idx].sensor_radius_multiplier = parent_sensor_mult;
            self.beings[child_idx].mutation_count = parent_mut_count;

            let mutated_geometry = self.rng.chance(self.config.mutation_probability);
            let mutated_axiom = self
                .rng
                .chance(self.config.spontaneous_axiom_mutation_probability);

            let mut geom_mutated = false;
            let mut spontaneous_axiom: Option<String> = None;

            if mutated_geometry {
                geom_mutated = true;
                self.beings[child_idx].mutation_count += 1;
                let new_sym = (self.beings[child_idx].symmetry_order + 1).min(12);
                self.beings[child_idx].symmetry_order = new_sym;

                let speed_delta = self.rng.range(-0.08, 0.14);
                let sensor_delta = self.rng.range(-0.06, 0.14);
                self.beings[child_idx].speed_multiplier =
                    (self.beings[child_idx].speed_multiplier + speed_delta).clamp(0.5, 2.5);
                self.beings[child_idx].sensor_radius_multiplier =
                    (self.beings[child_idx].sensor_radius_multiplier + sensor_delta)
                        .clamp(0.6, 2.5);

                let c_max_dim = self.beings[child_idx].max_dimension;
                let c_radius = self.beings[child_idx].radius;
                if c_max_dim == 2 {
                    let sides = new_sym.max(3);
                    self.beings[child_idx].sides = sides;
                    self.beings[child_idx].polytope = PolytopeND::generate_polygon_2d(
                        sides,
                        c_radius,
                        max_universe_dim,
                    );
                } else {
                    let next_family = match self.beings[child_idx].polytope.family {
                        PolytopeFamily::Simplex => PolytopeFamily::Hypercube,
                        PolytopeFamily::Hypercube => PolytopeFamily::Orthoplex,
                        _ => PolytopeFamily::Simplex,
                    };
                    let new_poly = PolytopeND::generate_by_family(
                        next_family,
                        c_max_dim,
                        new_sym,
                        c_radius,
                        max_universe_dim,
                    );
                    self.beings[child_idx].sides = new_poly.facet_count;
                    self.beings[child_idx].polytope = new_poly;
                }
                self.beings[child_idx].color_hue = (parent_hue + 28) % 360;
            }

            if mutated_axiom && self.beings[child_idx].max_dimension < max_universe_dim {
                let next_dim = self.beings[child_idx].max_dimension + 1;
                let next_dim_axioms = self.config.get_axioms_for_dimension(next_dim);
                // Synthesize a missing axiom for (d + 1) when parent/child already holds >= 1 axiom for (d + 1) or via spontaneous insight
                let unknown: Vec<String> = next_dim_axioms
                    .iter()
                    .filter(|ax| !self.beings[child_idx].known_axioms.contains(ax))
                    .cloned()
                    .collect();
                let pool = if !unknown.is_empty() {
                    unknown
                } else {
                    next_dim_axioms
                };
                if !pool.is_empty() {
                    let pick_idx = self.rng.int(0, (pool.len() - 1) as i32) as usize;
                    let discovered = pool[pick_idx].clone();
                    if !self.beings[child_idx].known_axioms.contains(&discovered) {
                        self.beings[child_idx].known_axioms.push(discovered.clone());
                    }
                    self.beings[child_idx].mutation_count += 1;
                    spontaneous_axiom = Some(discovered);
                }
            }

            // Check for Mutation Ascension if child holds all axioms for (max_dim + 1) and symmetry_order >= max_dim + 2
            let mut ascension_event: Option<String> = None;
            if (geom_mutated || spontaneous_axiom.is_some())
                && self.beings[child_idx].max_dimension < max_universe_dim
            {
                let from_dim = self.beings[child_idx].max_dimension;
                let next_dim = from_dim + 1;
                let req_axioms = self.config.get_axioms_for_dimension(next_dim);
                let has_all = req_axioms
                    .iter()
                    .all(|ax| self.beings[child_idx].known_axioms.contains(ax));

                if has_all && self.beings[child_idx].symmetry_order >= from_dim + 2 {
                    let sym = self.beings[child_idx].symmetry_order;
                    let new_family = if sym == 4 {
                        PolytopeFamily::Hypercube
                    } else if sym >= 5 {
                        PolytopeFamily::Orthoplex
                    } else {
                        PolytopeFamily::Simplex
                    };
                    let radius = self.beings[child_idx].radius;
                    let new_poly = PolytopeND::generate_by_family(
                        new_family,
                        next_dim,
                        sym,
                        radius,
                        max_universe_dim,
                    );
                    let schlafli = new_poly.schlafli_symbol.clone();

                    self.beings[child_idx].max_dimension = next_dim;
                    self.beings[child_idx].active_dimension = next_dim;
                    self.beings[child_idx].phase_state = PhaseState::Native;
                    self.beings[child_idx].sides = new_poly.facet_count;
                    self.beings[child_idx].polytope = new_poly;
                    self.beings[child_idx].fold_progress = 1.0;

                    let unlocked_axis = next_dim - 1;
                    let lift_pos = self.rng.range(3.0, 7.0);
                    let lift_vel = self.rng.range(1.5, 3.5);
                    self.beings[child_idx].position.set(unlocked_axis, lift_pos);
                    self.beings[child_idx].velocity.set(unlocked_axis, lift_vel);
                    let omega = self.rng.range(0.6, 1.4);
                    self.beings[child_idx]
                        .angular_velocities
                        .push((0, unlocked_axis, omega));

                    if next_dim > self.max_unlocked_dimension {
                        self.max_unlocked_dimension = next_dim;
                    }

                    ascension_event = Some(format!(
                        "{{\"entityId\":{},\"collaboratorIds\":[{}],\"fromDimension\":{},\"toDimension\":{},\"newPolytopeFamily\":\"{}\",\"schlafliSymbol\":\"{}\",\"method\":\"MUTATION_ASCENSION\",\"universalTime\":{:.4}}}",
                        child_id,
                        child_id,
                        from_dim,
                        next_dim,
                        new_family.as_str(),
                        schlafli,
                        now
                    ));
                }
            }

            let child_active_dim = self.beings[child_idx].active_dimension;
            let child_sym = self.beings[child_idx].symmetry_order;
            let child_fam = self.beings[child_idx].polytope.family.as_str();

            self.record_event(
                "BEING_REPRODUCED",
                format!(
                    "{{\"parentId\":{},\"childId\":{},\"generation\":{},\"dimension\":{},\"inheritedAxiomCount\":{}}}",
                    parent_id,
                    child_id,
                    inherited_gen,
                    child_active_dim,
                    inherited_axioms.len()
                ),
            );

            if geom_mutated || spontaneous_axiom.is_some() {
                let sp_ax_json = match &spontaneous_axiom {
                    Some(s) => format!("\"{}\"", s),
                    None => "null".to_string(),
                };
                self.record_event(
                    "BEING_MUTATED",
                    format!(
                        "{{\"entityId\":{},\"parentId\":{},\"generation\":{},\"symmetryOrder\":{},\"polytopeFamily\":\"{}\",\"geometricMutation\":{},\"spontaneousAxiom\":{}}}",
                        child_id,
                        parent_id,
                        inherited_gen,
                        child_sym,
                        child_fam,
                        geom_mutated,
                        sp_ax_json
                    ),
                );
            }

            if let Some(asc_json) = ascension_event {
                self.ascension_log.push(asc_json.clone());
                if self.ascension_log.len() > 64 {
                    let excess = self.ascension_log.len() - 64;
                    self.ascension_log.drain(0..excess);
                }
                self.record_event("DIMENSIONAL_ASCENSION", asc_json);
            }
        }
    }

    // ========================================================================
    // System 5: REQ-SYS-005 — Collaborative Discovery & Polytope Folding
    // ========================================================================

    /// Detects connected clusters of `k`-D beings within `folding_radius` (or solo mutated beings)
    /// that collectively hold all required axioms for dimension `k + 1`, folding the leader into
    /// a `(k + 1)`-D polytope and lifting off along orthogonal axis `k`.
    pub fn step_collaborative_folding(&mut self, dt: f64) {
        if dt <= 0.0 {
            return;
        }
        let now = self.clock.universal_time;
        let max_universe_dim = self.config.max_universe_dimension.max(2);
        let min_collaborators = self.config.min_collaborators_for_folding;
        let base_folding_radius = self.config.folding_radius;
        let max_pop_per_dim = self.config.max_population_per_dimension;

        let mut folded_this_tick: BTreeSet<u32> = BTreeSet::new();
        let mut ascension_events: Vec<String> = Vec::new();

        for k in 2..max_universe_dim {
            let target_dim = k + 1;
            let mut target_pop = self
                .beings
                .iter()
                .filter(|b| b.max_dimension == target_dim)
                .count();
            if target_pop >= max_pop_per_dim {
                continue;
            }

            let required_axioms = self.config.get_axioms_for_dimension(target_dim);

            // Indices of native k-D beings ready to fold
            let native_indices: Vec<usize> = self
                .beings
                .iter()
                .enumerate()
                .filter(|(_, b)| {
                    !folded_this_tick.contains(&b.id)
                        && b.active_dimension == k
                        && b.max_dimension == k
                        && b.exchange_cooldown <= 0.0
                })
                .map(|(idx, _)| idx)
                .collect();

            if native_indices.is_empty() {
                continue;
            }

            // Indices of all beings currently active in dimension k (including phased-down visitors)
            let collab_indices: Vec<usize> = self
                .beings
                .iter()
                .enumerate()
                .filter(|(_, b)| !folded_this_tick.contains(&b.id) && b.active_dimension == k)
                .map(|(idx, _)| idx)
                .collect();

            for &anchor_idx in &native_indices {
                if target_pop >= max_pop_per_dim {
                    break;
                }
                let anchor_id = self.beings[anchor_idx].id;
                if folded_this_tick.contains(&anchor_id) {
                    continue;
                }
                if self.clock.tick_count > 2
                    && ((self.clock.tick_count + (anchor_id as u64)) & 3 != 0)
                {
                    continue;
                }

                let anchor_sensor = self.beings[anchor_idx].sensor_radius_multiplier;
                let mut cluster_indices = vec![anchor_idx];

                for &cand_idx in &collab_indices {
                    let cand_id = self.beings[cand_idx].id;
                    if cand_idx == anchor_idx || folded_this_tick.contains(&cand_id) {
                        continue;
                    }
                    let sensor_mult =
                        anchor_sensor.max(self.beings[cand_idx].sensor_radius_multiplier);
                    let fold_rad = base_folding_radius * sensor_mult;
                    let fold_rad_sq = fold_rad * fold_rad;

                    let mut dist_sq = 0.0;
                    for axis in 0..k {
                        let diff = self.beings[anchor_idx].position.get(axis)
                            - self.beings[cand_idx].position.get(axis);
                        dist_sq += diff * diff;
                    }
                    if dist_sq <= fold_rad_sq {
                        cluster_indices.push(cand_idx);
                    }
                }

                let anchor_sym = self.beings[anchor_idx].symmetry_order;
                let can_solo_fold = anchor_sym >= k + 2;

                if cluster_indices.len() < min_collaborators && !can_solo_fold {
                    continue;
                }

                // Zero-allocation check of required_axioms across cluster_indices
                let matched_count = required_axioms
                    .iter()
                    .filter(|req| {
                        cluster_indices.iter().any(|&c_idx| {
                            self.beings[c_idx].known_axioms.iter().any(|ax| ax == *req)
                        })
                    })
                    .count();
                let progress =
                    (matched_count as f64) / (required_axioms.len().max(1) as f64);
                for &c_idx in &cluster_indices {
                    if progress > self.beings[c_idx].fold_progress {
                        self.beings[c_idx].fold_progress = progress;
                    }
                }

                if matched_count < required_axioms.len() {
                    continue;
                }

                // Select lead native k-D being with highest score
                let mut lead_idx = anchor_idx;
                let mut best_score = f64::NEG_INFINITY;
                for &c_idx in &cluster_indices {
                    if self.beings[c_idx].max_dimension != k {
                        continue;
                    }
                    let score = (self.beings[c_idx].known_axioms.len() as f64) * 100.0
                        + self.beings[c_idx].energy;
                    if score > best_score {
                        best_score = score;
                        lead_idx = c_idx;
                    }
                }
                let lead_id = self.beings[lead_idx].id;

                // Share all required_axioms across cluster members and link them
                let mut collaborator_ids = Vec::with_capacity(cluster_indices.len());
                for &c_idx in &cluster_indices {
                    let cid = self.beings[c_idx].id;
                    collaborator_ids.push(cid);
                    folded_this_tick.insert(cid);

                    for req in &required_axioms {
                        if !self.beings[c_idx].known_axioms.contains(req) {
                            self.beings[c_idx].known_axioms.push(req.clone());
                        }
                    }
                    if cid != lead_id {
                        self.beings[c_idx].upsert_collaboration_link(
                            lead_id,
                            format!("FOLD_DIM_{}", target_dim),
                            now,
                        );
                        self.beings[c_idx].exchange_cooldown = self.config.exchange_cooldown;
                        self.beings[c_idx].energy += self.config.energy_gain_per_exchange * 0.5;
                    }
                }

                // Grant leader one insight axiom of (target_dim + 1) if within max_universe_dim
                if target_dim + 1 <= max_universe_dim {
                    let next_tier = self.config.get_axioms_for_dimension(target_dim + 1);
                    if !next_tier.is_empty() {
                        let insight =
                            next_tier[(lead_id as usize) % next_tier.len()].clone();
                        if !self.beings[lead_idx].known_axioms.contains(&insight) {
                            self.beings[lead_idx].known_axioms.push(insight);
                        }
                    }
                }

                target_pop += 1;

                // Determine folded polytope family
                let new_family = self.select_folded_family(&cluster_indices, lead_idx);
                let new_radius = (self.beings[lead_idx].radius * 1.15).min(2.6);
                let new_poly = PolytopeND::generate_by_family(
                    new_family,
                    target_dim,
                    self.beings[lead_idx].symmetry_order,
                    new_radius,
                    max_universe_dim,
                );
                let schlafli = new_poly.schlafli_symbol.clone();

                self.beings[lead_idx].radius = new_radius;
                self.beings[lead_idx].sides = new_poly.facet_count;
                self.beings[lead_idx].polytope = new_poly;
                self.beings[lead_idx].color_hue = ((target_dim as u32) * 72 + 20) % 360;
                self.beings[lead_idx].last_cross_section = None;
                self.beings[lead_idx].max_dimension = target_dim;
                self.beings[lead_idx].active_dimension = target_dim;
                self.beings[lead_idx].phase_state = PhaseState::Native;
                self.beings[lead_idx].target_slice_dimension = k;
                self.beings[lead_idx].phase_timer = self.config.phasing_interval;
                self.beings[lead_idx].fold_progress = 1.0;

                if target_dim > self.max_unlocked_dimension {
                    self.max_unlocked_dimension = target_dim;
                }

                // Unlock orthogonal axis k
                let unlocked_axis = k;
                let lift_pos = self.rng.range(3.5, 7.5);
                let lift_vel = self.rng.range(1.8, 3.8);
                self.beings[lead_idx].position.set(unlocked_axis, lift_pos);
                self.beings[lead_idx].velocity.set(unlocked_axis, lift_vel);

                let omega1 = self.rng.range(0.55, 1.25);
                let omega2 = self.rng.range(-1.15, -0.45);
                if self.beings[lead_idx].angular_velocities.len() < 6 {
                    self.beings[lead_idx]
                        .angular_velocities
                        .push((0, unlocked_axis, omega1));
                    self.beings[lead_idx]
                        .angular_velocities
                        .push((unlocked_axis - 1, unlocked_axis, omega2));
                }

                let collab_str: Vec<String> =
                    collaborator_ids.iter().map(|id| id.to_string()).collect();
                let ev_json = format!(
                    "{{\"entityId\":{},\"collaboratorIds\":[{}],\"fromDimension\":{},\"toDimension\":{},\"newPolytopeFamily\":\"{}\",\"schlafliSymbol\":\"{}\",\"method\":\"COLLABORATIVE_FOLDING\",\"universalTime\":{:.4}}}",
                    lead_id,
                    collab_str.join(","),
                    k,
                    target_dim,
                    new_family.as_str(),
                    schlafli,
                    now
                );
                ascension_events.push(ev_json);
            }
        }

        for ev in ascension_events {
            self.ascension_log.push(ev.clone());
            if self.ascension_log.len() > 64 {
                let excess = self.ascension_log.len() - 64;
                self.ascension_log.drain(0..excess);
            }
            self.record_event("DIMENSIONAL_ASCENSION", ev);
        }
    }

    fn select_folded_family(
        &self,
        cluster_indices: &[usize],
        lead_idx: usize,
    ) -> PolytopeFamily {
        let lead_fam = self.beings[lead_idx].polytope.family;
        if lead_fam == PolytopeFamily::Hypercube {
            return PolytopeFamily::Hypercube;
        }
        if lead_fam == PolytopeFamily::Orthoplex {
            return PolytopeFamily::Orthoplex;
        }

        let mut total_sym = 0usize;
        for &c_idx in cluster_indices {
            total_sym += self.beings[c_idx].symmetry_order;
        }
        let avg_sym = (total_sym as f64) / (cluster_indices.len().max(1) as f64);
        let lead_order = self.beings[lead_idx].symmetry_order;

        if lead_order == 4 || (avg_sym.round() as usize) == 4 {
            PolytopeFamily::Hypercube
        } else if lead_order >= 5 || avg_sym >= 4.6 {
            PolytopeFamily::Orthoplex
        } else {
            PolytopeFamily::Simplex
        }
    }

    // ========================================================================
    // System 6: REQ-SYS-006 — Dimensional Phasing & Hyperplane Cross-Sections
    // ========================================================================

    /// Computes the live hyperplane cross-section of `being` intersecting `x_{target_dim} = slice_value`.
    pub fn compute_being_cross_section(
        being: &BeingEntity,
        target_dim: usize,
        slice_value: f64,
    ) -> Option<CrossSectionData> {
        let max_poly_dim = being.polytope.dimension.max(1);
        let slice_axis = target_dim.min(max_poly_dim.saturating_sub(1));
        let bound = being.radius * 1.45;

        // O(1) bounding-sphere pre-check before transforming any vertices
        if (being.position.get(slice_axis) - slice_value).abs() > bound {
            return None;
        }
        for d in (slice_axis + 1)..being.max_dimension {
            if being.active_dimension > d && being.position.get(d).abs() > bound {
                return None;
            }
        }

        let world_vertices = PolytopeND::transform_vertices(
            &being.polytope,
            &being.position,
            &being.orientation,
            1.0,
        );
        if world_vertices.is_empty() {
            return None;
        }

        let max_poly_dim = being
            .polytope
            .dimension
            .min(world_vertices[0].dim())
            .max(1);
        let slice_axis = target_dim.min(max_poly_dim.saturating_sub(1));

        let mut active_vertices = world_vertices;
        let mut active_edges = being.polytope.edges.clone();

        if max_poly_dim > slice_axis + 1 {
            for d in ((slice_axis + 1)..max_poly_dim).rev() {
                let mid_slice = CrossSectionND::slice_polytope(
                    &active_vertices,
                    &active_edges,
                    d,
                    0.0,
                    1e-7,
                );
                if mid_slice.intersects && mid_slice.points.len() >= 3 {
                    // Deduplicate intermediate vertices within 1e-3 and cap at 24 vertices
                    let mut dedup_verts: Vec<VectorND> = Vec::with_capacity(24);
                    for pt in mid_slice.points {
                        if !dedup_verts
                            .iter()
                            .any(|existing| existing.distance_to(&pt) <= 1e-3)
                        {
                            dedup_verts.push(pt);
                            if dedup_verts.len() >= 24 {
                                break;
                            }
                        }
                    }

                    let n = dedup_verts.len();
                    if n >= 3 {
                        let inv_n = 1.0 / (n as f64);
                        let mut cx = 0.0;
                        let mut cy = 0.0;
                        for v in &dedup_verts {
                            cx += v.get(0);
                            cy += v.get(1);
                        }
                        cx *= inv_n;
                        cy *= inv_n;

                        dedup_verts.sort_by(|a, b| {
                            let angle_a = (a.get(1) - cy).atan2(a.get(0) - cx);
                            let angle_b = (b.get(1) - cy).atan2(b.get(0) - cx);
                            angle_a
                                .partial_cmp(&angle_b)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        });

                        // O(V) linear edge graph: cycle edges + antipodal chords (<= 2*n edges)
                        let mut new_edges = Vec::with_capacity(2 * n);
                        let half = n / 2;
                        for i in 0..n {
                            new_edges.push((i, (i + 1) % n));
                            let chord = (i + half) % n;
                            if i < chord {
                                new_edges.push((i, chord));
                            }
                        }
                        active_vertices = dedup_verts;
                        active_edges = new_edges;
                    }
                }
            }
        }

        let mut result = CrossSectionND::slice_polytope(
            &active_vertices,
            &active_edges,
            slice_axis,
            slice_value,
            1e-7,
        );

        if result.intersects {
            let mut unique_subspace: Vec<VectorND> = Vec::new();
            for mut pt in result.points {
                for d in (slice_axis + 1)..pt.dim() {
                    pt.set(d, 0.0);
                }
                if !unique_subspace
                    .iter()
                    .any(|existing| existing.distance_to(&pt) <= 1e-7)
                {
                    unique_subspace.push(pt);
                    if unique_subspace.len() >= 32 {
                        break;
                    }
                }
            }
            result.points = unique_subspace;
        }

        Some(result)
    }

    /// Transitions a higher-dimensional being (`entity_id`) to `target_dim` (`2 <= target_dim <= max_dimension`).
    pub fn jump_being_to_dimension(&mut self, entity_id: u32, target_dim: usize) -> bool {
        let idx = match self.beings.iter().position(|b| b.id == entity_id) {
            Some(i) => i,
            None => return false,
        };

        let max_dim = self.beings[idx].max_dimension;
        if target_dim < 2 || target_dim > max_dim {
            return false;
        }

        let from_dim = self.beings[idx].active_dimension;
        let vec_dim = self.beings[idx].position.dim();

        if target_dim < max_dim {
            // Jump DOWN into lower-dimensional hyperplane m = target_dim
            self.beings[idx].active_dimension = target_dim;
            self.beings[idx].phase_state = PhaseState::PhasedDown;
            self.beings[idx].target_slice_dimension = target_dim;
            self.beings[idx].slice_coordinate = 0.0;
            self.beings[idx].phase_timer = self.config.phase_residence_time;

            for axis in target_dim..vec_dim {
                self.beings[idx].position.set(axis, 0.0);
                self.beings[idx].velocity.set(axis, 0.0);
            }

            let axioms_to_carry = self.config.get_axioms_for_dimension(target_dim + 1);
            for ax in axioms_to_carry {
                if !self.beings[idx].known_axioms.contains(&ax) {
                    self.beings[idx].known_axioms.push(ax);
                }
            }

            let cross_section = Self::compute_being_cross_section(
                &self.beings[idx],
                target_dim,
                self.beings[idx].slice_coordinate,
            );
            let cs_vert_count = cross_section
                .as_ref()
                .map(|cs| cs.points.len())
                .unwrap_or(0);
            self.beings[idx].last_cross_section = cross_section;

            self.record_event(
                "DIMENSIONAL_PHASE_JUMP",
                format!(
                    "{{\"entityId\":{},\"direction\":\"DOWN\",\"fromDimension\":{},\"toDimension\":{},\"maxDimension\":{},\"crossSectionVertexCount\":{}}}",
                    entity_id, from_dim, target_dim, max_dim, cs_vert_count
                ),
            );
            return true;
        }

        // Jump UP back to native higher dimension M = max_dim
        self.beings[idx].active_dimension = max_dim;
        self.beings[idx].phase_state = PhaseState::Native;
        self.beings[idx].target_slice_dimension = max_dim.saturating_sub(1).max(2);
        self.beings[idx].phase_timer = self.config.phasing_interval;
        self.beings[idx].last_cross_section = None;

        let start_axis = from_dim.min(max_dim.saturating_sub(1)).max(2);
        for axis in start_axis..max_dim {
            if self.beings[idx].position.get(axis).abs() < 1e-6 {
                let lift_pos = self.rng.range(3.5, 7.5);
                let lift_vel = self.rng.range(1.5, 3.5);
                self.beings[idx].position.set(axis, lift_pos);
                self.beings[idx].velocity.set(axis, lift_vel);
            }
        }

        self.record_event(
            "DIMENSIONAL_PHASE_JUMP",
            format!(
                "{{\"entityId\":{},\"direction\":\"UP\",\"fromDimension\":{},\"toDimension\":{},\"maxDimension\":{}}}",
                entity_id, from_dim, max_dim, max_dim
            ),
        );
        true
    }

    /// Executes one deterministic tick of higher-D observation, live cross-section slicing,
    /// and phase residence timer management (`REQ-SYS-006`).
    pub fn step_dimensional_phasing(&mut self, dt: f64) {
        if dt <= 0.0 {
            return;
        }
        let cone_radius = self.config.communication_radius * 1.8;
        let cone_radius_sq = cone_radius * cone_radius;
        let n = self.beings.len();

        let mut pending_jumps: Vec<(u32, usize)> = Vec::new();

        for i in 0..n {
            if self.beings[i].max_dimension <= 2 {
                continue;
            }

            // Case A: Phased down in a lower dimension m < max_dimension
            if self.beings[i].phase_state == PhaseState::PhasedDown
                || self.beings[i].active_dimension < self.beings[i].max_dimension
            {
                if self.beings[i].last_cross_section.is_none()
                    || ((self.clock.tick_count + (i as u64)) & 7 == 0)
                {
                    let active_dim = self.beings[i].active_dimension;
                    let slice_coord = self.beings[i].slice_coordinate;
                    self.beings[i].last_cross_section =
                        Self::compute_being_cross_section(&self.beings[i], active_dim, slice_coord);
                }

                if self.beings[i].phase_timer > 0.0 {
                    self.beings[i].phase_timer -= dt;
                    if self.beings[i].phase_timer <= 0.0 {
                        pending_jumps.push((self.beings[i].id, self.beings[i].max_dimension));
                    }
                }
                continue;
            }

            // Case B: Operating in highest unlocked dimension M = max_dimension
            let m_dim = self.beings[i].max_dimension;
            let timer_expiring = self.beings[i].phase_timer > 0.0 && self.beings[i].phase_timer <= dt;
            let should_scan = timer_expiring || ((self.clock.tick_count + (i as u64)) & 15 == 0);

            if should_scan {
                let mut observed = Vec::new();
                for j in 0..n {
                    if i == j {
                        continue;
                    }
                    let sub_dim = self.beings[j].active_dimension;
                    if sub_dim >= m_dim {
                        continue;
                    }
                    let mut proj_dist_sq = 0.0;
                    for axis in 0..sub_dim {
                        let diff = self.beings[i].position.get(axis)
                            - self.beings[j].position.get(axis);
                        proj_dist_sq += diff * diff;
                    }
                    if proj_dist_sq <= cone_radius_sq {
                        observed.push(self.beings[j].id);
                    }
                }

                self.beings[i].phase_state = if !observed.is_empty() {
                    PhaseState::Observing
                } else {
                    PhaseState::Native
                };
                self.beings[i].observed_entities = observed;
            }

            if self.beings[i].phase_timer <= 0.0 {
                self.beings[i].phase_timer = self.config.phasing_interval;
            } else {
                self.beings[i].phase_timer -= dt;
                if self.beings[i].phase_timer <= 0.0 {
                    self.beings[i].phase_timer = self.config.phasing_interval;
                    if !self.beings[i].observed_entities.is_empty() && self.rng.chance(0.25) {
                        let t_slice = if self.beings[i].target_slice_dimension >= 2
                            && self.beings[i].target_slice_dimension < m_dim
                        {
                            self.beings[i].target_slice_dimension
                        } else {
                            m_dim - 1
                        };
                        pending_jumps.push((self.beings[i].id, t_slice));
                    }
                }
            }
        }

        for (eid, target_dim) in pending_jumps {
            self.jump_being_to_dimension(eid, target_dim);
        }
    }

    // ========================================================================
    // Worldline Sampling & Pure-Rust JSON Snapshot Serialization
    // ========================================================================

    /// Samples worldline coordinates `(universal_time, position, active_dimension)` every `worldline_sample_interval`.
    pub fn sample_worldlines(&mut self) {
        let now = self.clock.universal_time;
        let interval = self.config.worldline_sample_interval.max(1e-4);
        let max_samples = self.config.worldline_max_samples.max(2);

        for b in &mut self.beings {
            if now - b.last_worldline_sample_time + 1e-9 >= interval {
                if b.worldline.len() >= max_samples {
                    b.worldline.remove(0);
                }
                b.worldline
                    .push((now, b.position.data.clone(), b.active_dimension));
                b.last_worldline_sample_time = now;
            }
        }
    }

    /// Serializes the complete N-dimensional universe state into a valid UTF-8 JSON string
    /// matching the JS visualizer's snapshot contract.
    pub fn to_snapshot_json(&self) -> String {
        let mut pop_by_dim: BTreeMap<usize, usize> = BTreeMap::new();
        for d in 2..=self.config.max_universe_dimension {
            pop_by_dim.insert(d, 0);
        }
        let mut discovered_axioms: BTreeSet<&str> = BTreeSet::new();
        let mut max_unlocked = self.max_unlocked_dimension;

        for b in &self.beings {
            *pop_by_dim.entry(b.active_dimension).or_insert(0) += 1;
            if b.max_dimension > max_unlocked {
                max_unlocked = b.max_dimension;
            }
            for ax in &b.known_axioms {
                discovered_axioms.insert(ax.as_str());
            }
        }

        let mut out = String::with_capacity(8192);
        out.push('{');
        out.push_str(&format!(
            "\"universalTime\":{:.6},\"tickCount\":{},\"timeScale\":{:.6},\"paused\":{},\"maxUnlockedDimension\":{},\"maxUniverseDimension\":{},\"worldBoundsRadius\":{:.4},\"reproductionThreshold\":{:.4},\"viewerSliceZ\":{:.4},\"totalAxiomsDiscovered\":{},",
            self.clock.universal_time,
            self.clock.tick_count,
            self.clock.time_scale,
            self.clock.paused,
            max_unlocked,
            self.config.max_universe_dimension,
            self.config.world_bounds_radius,
            self.config.reproduction_threshold,
            self.viewer_slice_z,
            discovered_axioms.len()
        ));

        // discoveredAxioms
        out.push_str("\"discoveredAxioms\":[");
        for (i, ax) in discovered_axioms.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            out.push('"');
            out.push_str(ax);
            out.push('"');
        }
        out.push_str("],");

        // populationByDimension
        out.push_str("\"populationByDimension\":{");
        for (i, (dim, count)) in pop_by_dim.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            out.push_str(&format!("\"{}\":{}", dim, count));
        }
        out.push_str("},");

        // beings
        out.push_str("\"beings\":[");
        for (i, b) in self.beings.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            let world_verts = PolytopeND::transform_vertices(
                &b.polytope,
                &b.position,
                &b.orientation,
                1.0,
            );

            out.push('{');
            out.push_str(&format!(
                "\"id\":{},\"entityId\":{},\"activeDimension\":{},\"maxDimension\":{},\"phaseState\":\"{}\",\"family\":\"{}\",\"sides\":{},\"schlafliSymbol\":\"{}\",\"colorHue\":{},",
                b.id,
                b.id,
                b.active_dimension,
                b.max_dimension,
                b.phase_state.as_str(),
                b.polytope.family.as_str(),
                b.sides,
                b.polytope.schlafli_symbol,
                b.color_hue
            ));

            // position
            out.push_str("\"position\":");
            push_f64_slice_json(&mut out, &b.position.data);
            out.push(',');

            // velocity
            out.push_str("\"velocity\":");
            push_f64_slice_json(&mut out, &b.velocity.data);
            out.push(',');

            // vertices (worldspace)
            out.push_str("\"vertices\":[");
            for (vi, v) in world_verts.iter().enumerate() {
                if vi > 0 {
                    out.push(',');
                }
                push_f64_slice_json(&mut out, &v.data);
            }
            out.push_str("],");

            // edges
            out.push_str("\"edges\":[");
            for (ei, &(u, v)) in b.polytope.edges.iter().enumerate() {
                if ei > 0 {
                    out.push(',');
                }
                out.push_str(&format!("[{},{}]", u, v));
            }
            out.push_str("],\"facets\":[");
            for (fi, facet) in b.polytope.facets.iter().enumerate() {
                if fi > 0 {
                    out.push(',');
                }
                out.push('[');
                for (vi, &v_idx) in facet.iter().enumerate() {
                    if vi > 0 {
                        out.push(',');
                    }
                    out.push_str(&v_idx.to_string());
                }
                out.push(']');
            }
            out.push_str("],");

            let parent_json = match b.parent_id {
                Some(pid) => pid.to_string(),
                None => "null".to_string(),
            };
            out.push_str(&format!(
                "\"energy\":{:.4},\"generation\":{},\"parentId\":{},\"mutationCount\":{},\"symmetryOrder\":{},\"foldProgress\":{:.4},",
                b.energy,
                b.generation,
                parent_json,
                b.mutation_count,
                b.symmetry_order,
                b.fold_progress
            ));

            // knownAxioms
            out.push_str("\"knownAxioms\":[");
            for (ai, ax) in b.known_axioms.iter().enumerate() {
                if ai > 0 {
                    out.push(',');
                }
                out.push('"');
                out.push_str(ax);
                out.push('"');
            }
            out.push_str("],");

            // activeLinks
            out.push_str("\"activeLinks\":[");
            for (li, (partner_id, axiom, ts)) in b.collaboration_links.iter().enumerate() {
                if li > 0 {
                    out.push(',');
                }
                out.push_str(&format!(
                    "{{\"partnerId\":{},\"axiom\":\"{}\",\"timestamp\":{:.4}}}",
                    partner_id, axiom, ts
                ));
            }
            out.push_str("],");

            // crossSection / sliceAtZ
            out.push_str("\"crossSection\":");
            if let Some(cs) = &b.last_cross_section {
                push_cross_section_json(&mut out, cs);
            } else {
                out.push_str("null");
            }
            out.push_str(",\"sliceAtZ\":");
            if b.max_dimension >= 3 {
                if let Some(slice_cs) = Self::compute_being_cross_section(b, 2, self.viewer_slice_z)
                {
                    push_cross_section_json(&mut out, &slice_cs);
                } else {
                    out.push_str("{\"intersects\":false,\"points\":[],\"orderedPolygon2D\":[],\"segments\":[],\"centroid\":[],\"boundingRadius\":0}");
                }
            } else {
                out.push_str("null");
            }
            out.push(',');

            // worldline
            out.push_str("\"worldline\":[");
            if self.include_worldlines_in_snapshot {
                let wl_slice = if b.worldline.len() > 36 {
                    &b.worldline[b.worldline.len() - 36..]
                } else {
                    &b.worldline[..]
                };
                for (wi, (t, coords, dim)) in wl_slice.iter().enumerate() {
                    if wi > 0 {
                        out.push(',');
                    }
                    out.push_str(&format!(
                        "{{\"time\":{:.4},\"dimension\":{},\"coordinates\":",
                        t, dim
                    ));
                    push_f64_slice_json(&mut out, coords);
                    out.push('}');
                }
            }
            out.push_str("]}");
        }
        out.push_str("],");

        // pulses
        out.push_str("\"pulses\":[");
        for (pi, p) in self.pulses.iter().enumerate() {
            if pi > 0 {
                out.push(',');
            }
            let src_id = p.source_entity_id.map(|id| id as i64).unwrap_or(-1);
            out.push_str(&format!(
                "{{\"id\":{},\"pulseEntityId\":{},\"sourceEntityId\":{},\"sourceDimension\":{},\"targetDimension\":{},\"axiom\":\"{}\",\"currentRadius\":{:.4},\"maxRadius\":{:.4},\"intensity\":{:.4},\"origin\":",
                p.id,
                p.id,
                src_id,
                p.source_dimension,
                p.target_dimension,
                p.axiom_payload,
                p.current_radius,
                p.max_radius,
                p.intensity
            ));
            push_f64_slice_json(&mut out, &p.origin.data);
            out.push('}');
        }
        out.push_str("],");

        // ascensionLog
        out.push_str("\"ascensionLog\":[");
        for (ai, entry) in self.ascension_log.iter().enumerate() {
            if ai > 0 {
                out.push(',');
            }
            out.push_str(entry);
        }
        out.push_str("],");

        // recentEvents
        out.push_str("\"recentEvents\":[");
        let start_ev = self.recent_events.len().saturating_sub(40);
        for (ei, ev) in self.recent_events[start_ev..].iter().enumerate() {
            if ei > 0 {
                out.push(',');
            }
            out.push_str(&format!(
                "{{\"type\":\"{}\",\"universalTime\":{:.4},\"tickCount\":{},\"payload\":{}}}",
                ev.event_type, ev.universal_time, ev.tick_count, ev.summary_json
            ));
        }
        out.push_str("]}");

        out
    }
}

fn push_f64_slice_json(out: &mut String, slice: &[f64]) {
    out.push('[');
    for (i, &val) in slice.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        if val.is_finite() {
            out.push_str(&format!("{:.5}", val));
        } else {
            out.push_str("0.0");
        }
    }
    out.push(']');
}

fn push_cross_section_json(out: &mut String, cs: &CrossSectionData) {
    out.push('{');
    out.push_str(&format!(
        "\"intersects\":{},\"boundingRadius\":{:.5},\"centroid\":",
        cs.intersects, cs.bounding_radius
    ));
    if let Some(ref c) = cs.centroid {
        push_f64_slice_json(out, &c.data);
    } else {
        out.push_str("[0.0,0.0]");
    }
    out.push_str(",\"points\":[");
    for (i, pt) in cs.points.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        push_f64_slice_json(out, &pt.data);
    }
    out.push_str("],\"orderedPolygon2D\":[");
    for (i, pt2) in cs.ordered_polygon_2d.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&format!("[{:.5},{:.5}]", pt2[0], pt2[1]));
    }
    out.push_str("],\"segments\":[");
    for (i, seg) in cs.segments.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&format!("[{},{}]", seg[0], seg[1]));
    }
    out.push_str("]}");
}
