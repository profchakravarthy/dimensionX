/**
 * @fileoverview Zero-Dependency Rust WebAssembly Bridge (`RustWasmUniverse`).
 * Loads the compiled `dimensionx_core.wasm` binary (`wasm32-unknown-unknown`) and exposes
 * the Rust simulation universe directly to `SpaceTimeVisualizer` and `ControlHUD`.
 * ZERO core math or simulation logic is duplicated in JavaScript: all N-D vector/rotor math,
 * polytope generation, hyperplane cross-section slicing, proper-time clock integration,
 * and simulation systems execute exclusively inside the compiled Rust WebAssembly core.
 * License: MIT
 */

const FAMILY_TO_CODE = Object.freeze({
  polygon: 0,
  simplex: 1,
  hypercube: 2,
  orthoplex: 3
});

export class RustWasmUniverse {
  /**
   * Instantiates a `RustWasmUniverse` from an already-instantiated WebAssembly module's exports.
   *
   * @param {WebAssembly.Exports} wasmExports
   * @param {object} [configOverrides={}]
   */
  constructor(wasmExports, configOverrides = {}) {
    if (!wasmExports || typeof wasmExports.universe_init !== 'function') {
      throw new Error('RustWasmUniverse requires valid dimensionx_core.wasm exports.');
    }
    this.wasm = wasmExports;
    this.isRustWasm = true;

    const maxUniverseDimension = Math.max(2, Number(configOverrides.maxUniverseDimension) || 6);
    const initialDimension = Math.max(2, Number(configOverrides.initialDimension) || 2);
    const initialTimeScale =
      Number.isFinite(configOverrides.initialTimeScale) && configOverrides.initialTimeScale >= 0
        ? Number(configOverrides.initialTimeScale)
        : 1.0;
    const randomSeed = (Number(configOverrides.randomSeed) || 20261031) >>> 0;

    this.config = {
      maxUniverseDimension,
      initialDimension,
      initialTimeScale,
      randomSeed,
      worldBoundsRadius: 40.0,
      reproductionThreshold: 100.0,
      fixedDeltaTime: 1.0 / 60.0
    };

    this.clock = {
      universalTime: 0,
      tickCount: 0,
      timeScale: initialTimeScale,
      paused: initialTimeScale === 0,
      fixedDeltaTime: 1.0 / 60.0
    };

    this.maxUnlockedDimension = initialDimension;
    this.ascensionLog = [];
    this.recentEvents = [];
    this._latestSnapshot = null;
    this._lastSliceZ = 0.0;
    this._includeWorldlines = true;
    this._decoder = new TextDecoder('utf-8');

    this.wasm.universe_init(maxUniverseDimension >>> 0, initialTimeScale, randomSeed);
    this._syncFromWasm();
  }

  /**
   * Asynchronously loads and instantiates `dimensionx_core.wasm` from a URL (browser)
   * or `BufferSource` (Node.js / test runner).
   *
   * @param {string | BufferSource} wasmSource
   * @param {object} [configOverrides={}]
   * @returns {Promise<RustWasmUniverse>}
   */
  static async create(wasmSource = './src/wasm/dimensionx_core.wasm', configOverrides = {}) {
    let instance;
    if (typeof wasmSource === 'string') {
      const response = await fetch(wasmSource);
      if (!response.ok) {
        throw new Error(`Failed to fetch Rust WASM binary at ${wasmSource}: HTTP ${response.status}`);
      }
      if (typeof WebAssembly.instantiateStreaming === 'function') {
        try {
          const result = await WebAssembly.instantiateStreaming(response, {});
          instance = result.instance;
        } catch {
          const fallbackResp = await fetch(wasmSource);
          const bytes = await fallbackResp.arrayBuffer();
          const result = await WebAssembly.instantiate(bytes, {});
          instance = result.instance;
        }
      } else {
        const bytes = await response.arrayBuffer();
        const result = await WebAssembly.instantiate(bytes, {});
        instance = result.instance;
      }
    } else {
      const result = await WebAssembly.instantiate(wasmSource, {});
      instance = result.instance || result;
    }

    return new RustWasmUniverse(instance.exports, configOverrides);
  }

  /**
   * Seeds the initial N-dimensional population inside the Rust WASM core.
   *
   * @param {object} [counts]
   * @param {number} [counts.count2D=14]
   * @param {number} [counts.count3D=2]
   * @param {number} [counts.count4D=1]
   * @returns {number[]} Array of spawned entity IDs
   */
  seedInitialPopulation({ count2D = 14, count3D = 2, count4D = 1 } = {}) {
    this.wasm.universe_seed_default(
      Math.max(0, count2D | 0),
      Math.max(0, count3D | 0),
      Math.max(0, count4D | 0)
    );
    const snap = this._syncFromWasm();
    return snap.beings.map((b) => b.entityId);
  }

  /**
   * Spawns a new being in the Rust WASM core.
   *
   * @param {object} [options={}]
   * @returns {number} Newly spawned entity ID
   */
  spawnBeing({
    dimension = 2,
    maxDimension = dimension,
    family = dimension === 2 ? 'polygon' : 'simplex',
    sides = 3,
    energy = 45.0
  } = {}) {
    const dim = Math.max(2, maxDimension || dimension);
    const familyCode = FAMILY_TO_CODE[String(family).toLowerCase()] ?? (dim === 2 ? 0 : 1);
    const id = this.wasm.universe_spawn_being(
      dim >>> 0,
      familyCode >>> 0,
      Math.max(3, sides | 0),
      Number(energy) || 45.0
    );
    this._syncFromWasm();
    return id;
  }

  /**
   * Advances universal space-time in the Rust WASM core by `realDeltaSeconds`.
   *
   * @param {number} realDeltaSeconds
   * @returns {{ ticksExecuted: number, universalTime: number, timeScale: number, paused: boolean }}
   */
  step(realDeltaSeconds) {
    const ticksExecuted = this.wasm.universe_step(Number(realDeltaSeconds) || 0);
    if (ticksExecuted > 0 || !this._latestSnapshot) {
      this._syncFromWasm();
    }
    return {
      ticksExecuted,
      universalTime: this.clock.universalTime,
      timeScale: this.clock.timeScale,
      paused: this.clock.paused
    };
  }

  /**
   * Synchronously executes `tickCount` fixed simulation ticks in the Rust WASM core.
   *
   * @param {number} [tickCount=1]
   * @returns {{ ticksExecuted: number, universalTime: number, tickCount: number }}
   */
  stepTicks(tickCount = 1) {
    const count = Math.max(1, Math.floor(tickCount));
    const ticksExecuted = this.wasm.universe_step_ticks(count >>> 0);
    this._syncFromWasm();
    return {
      ticksExecuted,
      universalTime: this.clock.universalTime,
      tickCount: this.clock.tickCount
    };
  }

  /**
   * Sets the universal proper-time speed multiplier (`0` = paused, `< 1` = slow-mo, `1` = 1s/s, `> 1` = hyper-warp).
   *
   * @param {number} scale
   * @returns {number}
   */
  setTimeScale(scale) {
    const applied = this.wasm.universe_set_time_scale(Number(scale) || 0);
    this._syncFromWasm();
    return applied;
  }

  /**
   * Slows down universal proper time by stepping down `TIME_SCALE_LADDER` (`-` button).
   *
   * @returns {number} New timeScale
   */
  slowDownTime() {
    const next = this.wasm.universe_slow_down_time();
    this._syncFromWasm();
    return next;
  }

  /**
   * Speeds up universal proper time by stepping up `TIME_SCALE_LADDER` (`+` button).
   *
   * @returns {number} New timeScale
   */
  speedUpTime() {
    const next = this.wasm.universe_speed_up_time();
    this._syncFromWasm();
    return next;
  }

  /**
   * Enables or disables worldline history serialization in the Rust WASM snapshot (`REQ-PERF-001`).
   * When disabled, Rust skips serializing historical worldline samples every frame until needed.
   *
   * @param {boolean} enabled
   * @returns {boolean}
   */
  setIncludeWorldlines(enabled) {
    const flag = Boolean(enabled);
    this._includeWorldlines = flag;
    if (typeof this.wasm.universe_set_include_worldlines === 'function') {
      this.wasm.universe_set_include_worldlines(flag ? 1 : 0);
    }
    this._syncFromWasm();
    return flag;
  }

  /**
   * Sets the 2D hyperplane slice coordinate `x_2 = sliceZ` inside the Rust WASM core
   * so Rust computes exact `CrossSectionND::slice_polytope` contours (`sliceAtZ`).
   *
   * @param {number} sliceZ
   */
  setSliceZ(sliceZ) {
    const val = Number(sliceZ) || 0.0;
    if (Math.abs(val - this._lastSliceZ) < 1e-4) {
      return;
    }
    this._lastSliceZ = val;
    if (typeof this.wasm.universe_set_slice_z === 'function') {
      this.wasm.universe_set_slice_z(val);
      this._syncFromWasm();
    }
  }

  /**
   * Manually triggers a higher-dimensional information pulse in the Rust WASM core.
   *
   * @param {number} [sourceDimension=3]
   * @param {number} [targetDimension=2]
   * @returns {number | null}
   */
  triggerRandomInjection(sourceDimension = 3, targetDimension = 2) {
    const pulseId = this.wasm.universe_trigger_injection(
      Math.max(3, sourceDimension | 0),
      Math.max(2, targetDimension | 0)
    );
    this._syncFromWasm();
    return pulseId > 0 ? pulseId : null;
  }

  /**
   * Commands a higher-dimensional being to phase-jump to `targetDimension` in the Rust WASM core.
   *
   * @param {number} entityId
   * @param {number} targetDimension
   * @returns {boolean}
   */
  jumpBeingToDimension(entityId, targetDimension) {
    const ok =
      this.wasm.universe_jump_being_to_dimension(entityId >>> 0, targetDimension >>> 0) === 1;
    this._syncFromWasm();
    return ok;
  }

  /**
   * Returns the latest universe telemetry snapshot generated by the Rust WASM core.
   */
  getUniverseSnapshot() {
    if (!this._latestSnapshot) {
      this._syncFromWasm();
    }
    return this._latestSnapshot;
  }

  /**
   * Reads the UTF-8 JSON snapshot from Rust WebAssembly linear memory.
   * @private
   */
  _syncFromWasm() {
    const byteLen = this.wasm.universe_refresh_snapshot();
    const ptr = this.wasm.universe_get_snapshot_ptr();
    if (!ptr || byteLen === 0) {
      return this._latestSnapshot;
    }

    const memoryBytes = new Uint8Array(this.wasm.memory.buffer, ptr, byteLen);
    const jsonText = this._decoder.decode(memoryBytes);
    const snap = JSON.parse(jsonText);

    if (!this._includeWorldlines && Array.isArray(snap.beings)) {
      for (let i = 0; i < snap.beings.length; i++) {
        if (snap.beings[i].worldline && snap.beings[i].worldline.length > 0) {
          snap.beings[i].worldline = [];
        }
      }
    }

    this._latestSnapshot = snap;
    this.clock.universalTime = snap.universalTime;
    this.clock.tickCount = snap.tickCount;
    this.clock.timeScale = snap.timeScale;
    this.clock.paused = snap.paused;
    this.maxUnlockedDimension = snap.maxUnlockedDimension;
    this.ascensionLog = snap.ascensionLog || [];
    this.recentEvents = snap.recentEvents || [];
    if (Number.isFinite(snap.worldBoundsRadius)) {
      this.config.worldBoundsRadius = snap.worldBoundsRadius;
    }
    if (Number.isFinite(snap.reproductionThreshold)) {
      this.config.reproductionThreshold = snap.reproductionThreshold;
    }
    if (Number.isFinite(snap.maxUniverseDimension)) {
      this.config.maxUniverseDimension = snap.maxUniverseDimension;
    }

    return snap;
  }
}
