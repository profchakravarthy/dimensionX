/**
 * @fileoverview End-to-End Integration Tests for the Compiled Rust WebAssembly Core
 * (`src/wasm/dimensionx_core.wasm`) and `RustWasmUniverse` bridge (`src/engine/RustWasmBridge.js`).
 * ZERO core math or simulation code is duplicated in JS.
 * License: MIT
 */

import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { RustWasmUniverse } from '../src/engine/RustWasmBridge.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const WASM_PATH = path.resolve(__dirname, '../src/wasm/dimensionx_core.wasm');

test('RustWasmUniverse: loads compiled dimensionx_core.wasm, seeds 2D/3D/4D beings, and computes N-D vertices and 2D slices natively in Rust', async () => {
  const wasmBytes = fs.readFileSync(WASM_PATH);
  const universe = await RustWasmUniverse.create(wasmBytes, {
    maxUniverseDimension: 6,
    initialTimeScale: 1.0,
    randomSeed: 20261031
  });

  assert.equal(universe.isRustWasm, true);

  const ids = universe.seedInitialPopulation({
    count2D: 12,
    count3D: 2,
    count4D: 1
  });
  assert.equal(ids.length, 15);

  const snap = universe.getUniverseSnapshot();
  assert.equal(snap.beings.length, 15);
  assert.equal(snap.populationByDimension[2], 12);
  assert.equal(snap.populationByDimension[3], 2);
  assert.equal(snap.populationByDimension[4], 1);
  assert.equal(snap.maxUnlockedDimension, 4);

  // Verify Rust WASM natively computes world-space vertices and sliceAtZ cross-sections
  for (const b of snap.beings) {
    assert.ok(Array.isArray(b.vertices) && b.vertices.length >= 3);
    assert.ok(Array.isArray(b.edges) && b.edges.length >= 3);
    if (b.maxDimension >= 3) {
      assert.ok(b.sliceAtZ && typeof b.sliceAtZ.intersects === 'boolean');
    }
  }
});

test('RustWasmUniverse: Proper Time slowDownTime (-) and speedUpTime (+) across slow-motion and hyper-warp', async () => {
  const wasmBytes = fs.readFileSync(WASM_PATH);
  const universe = await RustWasmUniverse.create(wasmBytes, {
    maxUniverseDimension: 6,
    initialTimeScale: 1.0,
    randomSeed: 42
  });

  // Base 1x rate: 1.0 real second -> 60 ticks = 1.0 simulated second
  const step1x = universe.step(1.0);
  assert.equal(step1x.ticksExecuted, 60);
  assert.ok(Math.abs(universe.clock.universalTime - 1.0) < 1e-5);

  // Slow down proper time via '-' control into sub-1x slow-motion
  assert.equal(universe.slowDownTime(), 0.5);
  assert.equal(universe.slowDownTime(), 0.25);

  // At 0.25x slow-motion, 1.0 real second -> 15 ticks = 0.25 simulated seconds (total 1.25s)
  const stepSlow = universe.step(1.0);
  assert.equal(stepSlow.ticksExecuted, 15);
  assert.ok(Math.abs(universe.clock.universalTime - 1.25) < 1e-5);

  // Step down all the way to 0x (Paused)
  assert.equal(universe.slowDownTime(), 0.1);
  assert.equal(universe.slowDownTime(), 0.05);
  assert.equal(universe.slowDownTime(), 0.0);
  assert.equal(universe.clock.paused, true);

  // Speed up proper time via '+' control back through slow-mo to 1x and 2x
  assert.equal(universe.speedUpTime(), 0.05);
  assert.equal(universe.speedUpTime(), 0.1);
  assert.equal(universe.speedUpTime(), 0.25);
  assert.equal(universe.speedUpTime(), 0.5);
  assert.equal(universe.speedUpTime(), 1.0);
  assert.equal(universe.speedUpTime(), 2.0);
  assert.equal(universe.clock.paused, false);
});

test('RustWasmUniverse: Higher-D pulse injection, dimensional phase jump, setSliceZ, and multi-tick evolution', async () => {
  const wasmBytes = fs.readFileSync(WASM_PATH);
  const universe = await RustWasmUniverse.create(wasmBytes, {
    maxUniverseDimension: 6,
    initialTimeScale: 1.0,
    randomSeed: 999
  });

  universe.seedInitialPopulation({ count2D: 14, count3D: 2, count4D: 1 });

  // Trigger a 3D -> 2D information pulse in Rust WASM
  const pulseId = universe.triggerRandomInjection(3, 2);
  assert.ok(pulseId !== null && pulseId > 0);
  assert.ok(universe.getUniverseSnapshot().pulses.length >= 1);

  // Find a 3D being and command it to phase-jump down to 2D Flatland
  const snapBefore = universe.getUniverseSnapshot();
  const being3D = snapBefore.beings.find((b) => b.maxDimension === 3);
  assert.ok(being3D, '3D being exists');

  const jumpedDown = universe.jumpBeingToDimension(being3D.entityId, 2);
  assert.equal(jumpedDown, true);

  const afterJump = universe
    .getUniverseSnapshot()
    .beings.find((b) => b.entityId === being3D.entityId);
  assert.equal(afterJump.activeDimension, 2);
  assert.equal(afterJump.phaseState, 'PHASED_DOWN');
  assert.ok(afterJump.crossSection && afterJump.crossSection.intersects);

  // Verify setSliceZ updates viewerSliceZ in Rust WASM
  universe.setSliceZ(1.25);
  assert.ok(Math.abs(universe.getUniverseSnapshot().viewerSliceZ - 1.25) < 1e-4);

  // Advance 180 ticks (3 simulated seconds) in Rust WASM and verify kinematics, pulses, and worldlines
  const stepRes = universe.stepTicks(180);
  assert.equal(stepRes.ticksExecuted, 180);

  // Verify on-demand worldline serialization toggle (REQ-PERF-001)
  universe.setIncludeWorldlines(false);
  const snapNoWl = universe.getUniverseSnapshot();
  assert.ok(snapNoWl.universalTime >= 2.99);
  assert.ok(snapNoWl.totalAxiomsDiscovered > 0);
  assert.equal(snapNoWl.beings[0].worldline.length, 0, 'Worldlines omitted when includeWorldlines is false');

  universe.setIncludeWorldlines(true);
  const snapWithWl = universe.getUniverseSnapshot();
  assert.ok(snapWithWl.beings[0].worldline.length > 0, 'Worldlines serialized when includeWorldlines is true');

  // Verify sustained 300-tick (5-second) execution completes without freeze
  universe.setIncludeWorldlines(false);
  const t0 = performance.now();
  universe.stepTicks(300);
  const elapsedMs = performance.now() - t0;
  assert.ok(elapsedMs < 5000, `300 ticks executed in ${elapsedMs.toFixed(1)}ms`);
});
