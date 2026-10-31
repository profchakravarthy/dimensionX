//! WebAssembly C-ABI Exports for `dimensionx-core` (`wasm32-unknown-unknown`).
//!
//! Exposes a zero-dependency C-ABI interface so the JavaScript browser host or native test runner
//! can initialize, step, control proper-time dilation (`-` / `+`), inject pulses, phase-jump beings,
//! and read UTF-8 JSON snapshots directly from linear memory.
//!
//! 100% pure Rust standard library (`std`), zero external dependencies, MIT License.

use std::ptr;
use std::sync::{Mutex, OnceLock};

use crate::config::SimulationConfig;
use crate::math::PolytopeFamily;
use crate::universe::DimensionXUniverse;

struct WasmState {
    universe: DimensionXUniverse,
    snapshot_buf: Vec<u8>,
}

fn global_state() -> &'static Mutex<WasmState> {
    static STATE: OnceLock<Mutex<WasmState>> = OnceLock::new();
    STATE.get_or_init(|| {
        Mutex::new(WasmState {
            universe: DimensionXUniverse::default(),
            snapshot_buf: Vec::new(),
        })
    })
}

/// Initializes (or re-initializes) the global `DimensionXUniverse` instance.
#[no_mangle]
pub extern "C" fn universe_init(max_dim: u32, initial_time_scale: f64, seed: u32) {
    let mut cfg = SimulationConfig::default();
    cfg.max_universe_dimension = (max_dim as usize).max(2);
    cfg.initial_time_scale = if initial_time_scale.is_finite() && initial_time_scale >= 0.0 {
        initial_time_scale
    } else {
        1.0
    };
    cfg.random_seed = if seed == 0 { 20_261_031 } else { seed };

    if let Ok(mut guard) = global_state().lock() {
        guard.universe = DimensionXUniverse::new(cfg);
        guard.snapshot_buf.clear();
    }
}

/// Seeds the initial population across 2D, 3D, and 4D.
#[no_mangle]
pub extern "C" fn universe_seed_default(count_2d: u32, count_3d: u32, count_4d: u32) {
    if let Ok(mut guard) = global_state().lock() {
        guard.universe.seed_initial_population(
            count_2d as usize,
            count_3d as usize,
            count_4d as usize,
        );
    }
}

/// Advances the simulation clock by `real_dt_seconds` and returns the number of fixed ticks executed.
#[no_mangle]
pub extern "C" fn universe_step(real_dt_seconds: f64) -> u32 {
    if let Ok(mut guard) = global_state().lock() {
        let res = guard.universe.step(real_dt_seconds);
        res.ticks_executed as u32
    } else {
        0
    }
}

/// Synchronously executes `ticks` fixed simulation ticks regardless of pause state.
#[no_mangle]
pub extern "C" fn universe_step_ticks(ticks: u32) -> u32 {
    if let Ok(mut guard) = global_state().lock() {
        let res = guard.universe.step_ticks(ticks as usize);
        res.ticks_executed as u32
    } else {
        0
    }
}

/// Sets the universal proper-time speed multiplier (`0.0` = paused, `< 1.0` = slow-mo, `1.0` = 1s/s, `> 1.0` = hyper-warp).
#[no_mangle]
pub extern "C" fn universe_set_time_scale(scale: f64) -> f64 {
    if let Ok(mut guard) = global_state().lock() {
        guard.universe.set_time_scale(scale)
    } else {
        0.0
    }
}

/// Returns the current universal proper-time speed multiplier.
#[no_mangle]
pub extern "C" fn universe_get_time_scale() -> f64 {
    if let Ok(guard) = global_state().lock() {
        guard.universe.clock.time_scale
    } else {
        0.0
    }
}

/// **`-` control:** Steps down to the next lower value in `TIME_SCALE_LADDER` (supporting `< 1x` slow-motion).
#[no_mangle]
pub extern "C" fn universe_slow_down_time() -> f64 {
    if let Ok(mut guard) = global_state().lock() {
        guard.universe.slow_down_time()
    } else {
        0.0
    }
}

/// **`+` control:** Steps up to the next higher value in `TIME_SCALE_LADDER` (supporting `> 1x` hyper-warp).
#[no_mangle]
pub extern "C" fn universe_speed_up_time() -> f64 {
    if let Ok(mut guard) = global_state().lock() {
        guard.universe.speed_up_time()
    } else {
        0.0
    }
}

/// Sets the hyperplane slice Z coordinate (`x_2 = slice_z`) for 2D cross-section computation in Rust.
#[no_mangle]
pub extern "C" fn universe_set_slice_z(slice_z: f64) {
    if let Ok(mut guard) = global_state().lock() {
        guard.universe.set_viewer_slice_z(slice_z);
    }
}

/// Enables (`include != 0`) or disables (`include == 0`) serializing worldline arrays in `universe_refresh_snapshot()`.
#[no_mangle]
pub extern "C" fn universe_set_include_worldlines(include: u32) {
    if let Ok(mut guard) = global_state().lock() {
        guard.universe.set_include_worldlines(include != 0);
    }
}

/// Manually triggers a higher-dimensional information pulse from `source_dim` into `target_dim`.
/// Returns the created pulse entity ID (or `0` if none).
#[no_mangle]
pub extern "C" fn universe_trigger_injection(source_dim: u32, target_dim: u32) -> u32 {
    if let Ok(mut guard) = global_state().lock() {
        guard
            .universe
            .trigger_random_injection(source_dim as usize, target_dim as usize)
            .unwrap_or(0)
    } else {
        0
    }
}

/// Commands `entity_id` to phase-jump to `target_dim` (`1` = success, `0` = failure).
#[no_mangle]
pub extern "C" fn universe_jump_being_to_dimension(entity_id: u32, target_dim: u32) -> u32 {
    if let Ok(mut guard) = global_state().lock() {
        u32::from(
            guard
                .universe
                .jump_being_to_dimension(entity_id, target_dim as usize),
        )
    } else {
        0
    }
}

/// Spawns a new being in dimension `dim` with `family_code` (`0`=Polygon, `1`=Simplex, `2`=Hypercube, `3`=Orthoplex).
/// Returns the newly spawned entity ID.
#[no_mangle]
pub extern "C" fn universe_spawn_being(
    dim: u32,
    family_code: u32,
    sides: u32,
    energy: f64,
) -> u32 {
    let family = match family_code {
        0 => PolytopeFamily::Polygon,
        1 => PolytopeFamily::Simplex,
        2 => PolytopeFamily::Hypercube,
        3 => PolytopeFamily::Orthoplex,
        _ => {
            if dim <= 2 {
                PolytopeFamily::Polygon
            } else {
                PolytopeFamily::Simplex
            }
        }
    };

    if let Ok(mut guard) = global_state().lock() {
        guard.universe.spawn_being(
            dim as usize,
            Some(family),
            sides as usize,
            None,
            None,
            Vec::new(),
            1,
            None,
            energy,
        )
    } else {
        0
    }
}

/// Serializes the current universe snapshot into the static UTF-8 buffer and returns its byte length.
#[no_mangle]
pub extern "C" fn universe_refresh_snapshot() -> usize {
    if let Ok(mut guard) = global_state().lock() {
        let json = guard.universe.to_snapshot_json();
        guard.snapshot_buf = json.into_bytes();
        guard.snapshot_buf.len()
    } else {
        0
    }
}

/// Returns a raw pointer to the UTF-8 JSON snapshot buffer in WebAssembly linear memory.
#[no_mangle]
pub extern "C" fn universe_get_snapshot_ptr() -> *const u8 {
    if let Ok(guard) = global_state().lock() {
        if guard.snapshot_buf.is_empty() {
            ptr::null()
        } else {
            guard.snapshot_buf.as_ptr()
        }
    } else {
        ptr::null()
    }
}

/// Returns the byte length of the last generated UTF-8 JSON snapshot buffer.
#[no_mangle]
pub extern "C" fn universe_get_snapshot_len() -> usize {
    if let Ok(guard) = global_state().lock() {
        guard.snapshot_buf.len()
    } else {
        0
    }
}
