/**
 * @fileoverview Browser Entrypoint for dimensionX (`src/main.js`).
 * Loads the compiled Rust WebAssembly Core (`src/wasm/dimensionx_core.wasm`) via `RustWasmUniverse`,
 * bootstraps `ProjectorND`, `SpaceTimeVisualizer`, and `ControlHUD`, and drives the real-time
 * `requestAnimationFrame` simulation & dual-viewport rendering loop.
 * ZERO core math or simulation systems are duplicated in JS.
 * License: MIT
 */

import { RustWasmUniverse } from './engine/RustWasmBridge.js';
import { ProjectorND } from './render/ProjectorND.js';
import { SpaceTimeVisualizer } from './render/SpaceTimeVisualizer.js';
import { ControlHUD } from './ui/ControlHUD.js';

export async function bootstrapDimensionX(wasmSource = './src/wasm/dimensionx_core.wasm') {
  const canvas3D = document.getElementById('canvas-3d');
  const canvas2D = document.getElementById('canvas-2d');
  const engineBadge = document.getElementById('core-engine-badge');

  let universe = await RustWasmUniverse.create(wasmSource, {
    maxUniverseDimension: 6,
    initialDimension: 2,
    initialTimeScale: 1.0
  });
  const wasmExports = universe.wasm;

  if (engineBadge) {
    engineBadge.textContent = 'CORE: RUST WASM (N-D ACTIVE)';
  }

  const seededIds = universe.seedInitialPopulation({
    count2D: 16,
    count3D: 3,
    count4D: 1
  });

  // Trigger an initial Higher-D Information Pulse so the user sees wavefronts immediately
  universe.triggerRandomInjection(3, 2);

  const projector = new ProjectorND({
    cameraYaw: 0.68,
    cameraPitch: 0.46,
    cameraDistance: 64,
    fovScale: 520,
    higherDimProjectionDistance: 12,
    spaceTimeMode: false,
    timeAxisScale: 1.25
  });

  let hud = null;

  const visualizer = new SpaceTimeVisualizer({
    universe,
    canvas3D,
    canvas2D,
    projector,
    onEntitySelect: () => {
      if (hud) hud.update();
    }
  });

  // Synchronize on-demand worldline serialization with visualizer.showWorldlines (REQ-PERF-001)
  universe.setIncludeWorldlines(visualizer.showWorldlines);
  const wlCheckbox = document.getElementById('toggle-worldlines');
  if (wlCheckbox) {
    wlCheckbox.checked = visualizer.showWorldlines;
  }

  if (seededIds.length > 0) {
    visualizer.selectEntity(seededIds[seededIds.length - 1]);
  }

  // Optional deterministic URL query parameters for deep-linking & observatory state inspection
  if (typeof window !== 'undefined' && window.location && window.location.search) {
    const params = new URLSearchParams(window.location.search);
    if (params.get('worldlines') === '1') {
      visualizer.showWorldlines = true;
      universe.setIncludeWorldlines(true);
      if (wlCheckbox) wlCheckbox.checked = true;
    }
    if (params.get('spacetime') === '1') {
      projector.spaceTimeMode = true;
      visualizer.showWorldlines = true;
      universe.setIncludeWorldlines(true);
      const stCheckbox = document.getElementById('toggle-spacetime-mode');
      if (stCheckbox) stCheckbox.checked = true;
      if (wlCheckbox) wlCheckbox.checked = true;
    }
    const ticksParam = Number(params.get('ticks'));
    if (Number.isFinite(ticksParam) && ticksParam > 0) {
      universe.stepTicks(Math.min(2400, Math.floor(ticksParam)));
    }
    const sliceZParam = Number(params.get('sliceZ'));
    if (params.has('sliceZ') && Number.isFinite(sliceZParam)) {
      visualizer.sliceZ = sliceZParam;
      universe.setSliceZ(sliceZParam);
      const sliceSlider = document.getElementById('slice-z-slider');
      const sliceVal = document.getElementById('slice-z-value');
      if (sliceSlider) sliceSlider.value = String(sliceZParam);
      if (sliceVal) sliceVal.textContent = sliceZParam.toFixed(2);
    }
    if (params.get('jumpDown') === '1') {
      const snap = universe.getUniverseSnapshot();
      const higherBeing = snap.beings.find((b) => b.maxDimension >= 4) || snap.beings.find((b) => b.maxDimension >= 3);
      if (higherBeing) {
        universe.jumpBeingToDimension(higherBeing.entityId, 2);
        visualizer.selectEntity(higherBeing.entityId);
      }
    }
    if (params.get('pulse') === '1') {
      universe.triggerRandomInjection(3, 2);
      universe.triggerRandomInjection(4, 3);
      universe.stepTicks(18);
    }
  }

  const resetUniverse = () => {
    const currentScale = universe.clock.timeScale;
    const seed = Date.now() & 0x7fffffff;
    universe = new RustWasmUniverse(wasmExports, {
      maxUniverseDimension: 6,
      initialDimension: 2,
      initialTimeScale: currentScale,
      randomSeed: seed
    });
    universe.setIncludeWorldlines(visualizer.showWorldlines);
    const newIds = universe.seedInitialPopulation({
      count2D: 16,
      count3D: 3,
      count4D: 1
    });
    universe.triggerRandomInjection(3, 2);
    visualizer.universe = universe;
    hud.universe = universe;
    if (newIds.length > 0) {
      visualizer.selectEntity(newIds[0]);
    }
    visualizer.render();
    hud.update();
    if (typeof window !== 'undefined' && window.dimensionX) {
      window.dimensionX.universe = universe;
    }
  };

  hud = new ControlHUD({
    universe,
    visualizer,
    root: document,
    onResetUniverse: resetUniverse
  });

  let lastFrameTime = performance.now();
  let hudUpdateAccumulator = 0;

  function animationLoop(now) {
    const rawDeltaSeconds = Math.max(0, Math.min(0.1, (now - lastFrameTime) / 1000));
    lastFrameTime = now;

    universe.step(rawDeltaSeconds);
    visualizer.render();

    hudUpdateAccumulator += rawDeltaSeconds;
    if (hudUpdateAccumulator >= 0.12) {
      hudUpdateAccumulator = 0;
      hud.update();
    }

    requestAnimationFrame(animationLoop);
  }

  visualizer.render();
  hud.update();
  requestAnimationFrame(animationLoop);

  const appHandle = { universe, projector, visualizer, hud, resetUniverse };
  if (typeof window !== 'undefined') {
    window.dimensionX = appHandle;
  }
  return appHandle;
}

if (typeof window !== 'undefined' && typeof document !== 'undefined') {
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => bootstrapDimensionX());
  } else {
    bootstrapDimensionX();
  }
}
