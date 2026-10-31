//! Parameterized Simulation Configuration & Procedural N-D Axiom Generator (`REQ-FOUND-002`).
//!
//! Zero hardcoded dimensions, zero magic constants: all simulation rates, spatial bounds,
//! epistemic radii, and dimensional parameters are configured via `SimulationConfig`.
//!
//! 100% pure Rust standard library (`std`), MIT License.

/// Canonical geometric axiom archetypes unlocked during dimensional ascension.
pub const AXIOM_ARCHETYPES: &[&str] = &[
    "ORTHOGONAL_NORMAL",
    "DIHEDRAL_FOLD_ANGLE",
    "EULER_CLOSURE",
    "CROSS_SECTION_INVARIANCE",
    "SCHLAFLI_DUALITY",
    "BIVECTOR_PLANE_ROTATION",
    "HYPERVOLUME_INTEGRATION",
    "PARALLEL_HYPERPLANE_OFFSET",
];

/// Spatial boundary confinement behavior in each active dimension.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoundaryMode {
    Reflective,
    Toroidal,
}

impl BoundaryMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            BoundaryMode::Reflective => "reflective",
            BoundaryMode::Toroidal => "toroidal",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "toroidal" => BoundaryMode::Toroidal,
            _ => BoundaryMode::Reflective,
        }
    }
}

/// Complete parameterized configuration for an N-dimensional space-time simulation (`REQ-FOUND-002`).
#[derive(Clone, Debug)]
pub struct SimulationConfig {
    pub max_universe_dimension: usize,
    pub initial_dimension: usize,
    pub world_bounds_radius: f64,
    pub boundary_mode: BoundaryMode,
    pub fixed_delta_time: f64,
    pub initial_time_scale: f64,
    pub max_sub_steps_per_frame: usize,
    pub axioms_per_dimension: usize,
    pub communication_radius: f64,
    pub exchange_cooldown: f64,
    pub link_expiration_seconds: f64,
    pub folding_radius: f64,
    pub min_collaborators_for_folding: usize,
    pub higher_dim_injection_interval: f64,
    pub pulse_wave_speed: f64,
    pub pulse_max_radius: f64,
    pub max_concurrent_pulses: usize,
    pub reproduction_threshold: f64,
    pub energy_gain_per_second: f64,
    pub energy_gain_per_exchange: f64,
    pub mutation_probability: f64,
    pub spontaneous_axiom_mutation_probability: f64,
    pub max_population_per_dimension: usize,
    pub phasing_interval: f64,
    pub phase_residence_time: f64,
    pub worldline_sample_interval: f64,
    pub worldline_max_samples: usize,
    pub random_seed: u32,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            max_universe_dimension: 6,
            initial_dimension: 2,
            world_bounds_radius: 40.0,
            boundary_mode: BoundaryMode::Reflective,
            fixed_delta_time: 1.0 / 60.0,
            initial_time_scale: 1.0,
            max_sub_steps_per_frame: 600,
            axioms_per_dimension: 4,
            communication_radius: 6.5,
            exchange_cooldown: 1.5,
            link_expiration_seconds: 1.2,
            folding_radius: 8.0,
            min_collaborators_for_folding: 2,
            higher_dim_injection_interval: 3.0,
            pulse_wave_speed: 14.0,
            pulse_max_radius: 28.0,
            max_concurrent_pulses: 24,
            reproduction_threshold: 100.0,
            energy_gain_per_second: 8.0,
            energy_gain_per_exchange: 1.5,
            mutation_probability: 0.35,
            spontaneous_axiom_mutation_probability: 0.20,
            max_population_per_dimension: 45,
            phasing_interval: 6.0,
            phase_residence_time: 4.5,
            worldline_sample_interval: 0.25,
            worldline_max_samples: 120,
            random_seed: 20_261_031,
        }
    }
}

impl SimulationConfig {
    /// Creates a new `SimulationConfig` with default parameters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Procedurally generates the required geometric axioms to ascend from `(target_dim - 1)` to `target_dim`
    /// for any integer dimension `target_dim >= 3`, with zero upper dimension limit.
    pub fn get_axioms_for_dimension(&self, target_dim: usize) -> Vec<String> {
        let td = target_dim.max(3);
        let count = self.axioms_per_dimension.max(1);
        let mut axioms = Vec::with_capacity(count);
        for i in 0..count {
            if let Some(&archetype) = AXIOM_ARCHETYPES.get(i) {
                axioms.push(format!("DIM{}_{}", td, archetype));
            } else {
                axioms.push(format!("DIM{}_GEOMETRIC_INVARIANT_{}", td, i + 1));
            }
        }
        axioms
    }
}
