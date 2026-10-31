/**
 * @fileoverview Dual-Viewport N-Dimensional Space-Time & 2D Flatland Cross-Section Visualizer (`SpaceTimeVisualizer`).
 * Renders:
 *   1. Main 3D / N-D Omniscient Space-Time Viewport (`canvas3D`): stacked dimensional hyperplanes,
 *      cascaded N-D polytope wireframes & facets, expanding InformationPulse light-cones,
 *      peer communication beams, dimensional phase shadows, and 3D Space-Time Worldline corkscrews.
 *   2. 2D Flatland Hyperplane Cross-Section Viewport (`canvas2D`): true 2D plane view at z = sliceZ
 *      showing native 2D polygon beings alongside live mathematical cross-sections (`CrossSectionND`)
 *      of higher-dimensional (3D/4D/5D/6D) beings intersecting or phased into the plane.
 * Implements REQ-VIS-002, REQ-FOUND-002, and REQ-FOUND-003.
 * License: MIT
 */

import { ProjectorND } from './ProjectorND.js';

const DIMENSION_PALETTE = Object.freeze({
  2: { primary: '#38bdf8', glow: 'rgba(56, 189, 248, 0.35)', fill: 'rgba(56, 189, 248, 0.14)', label: '2D Flatland' },
  3: { primary: '#a855f7', glow: 'rgba(168, 85, 247, 0.42)', fill: 'rgba(168, 85, 247, 0.16)', label: '3D Polyhedra' },
  4: { primary: '#f59e0b', glow: 'rgba(245, 158, 11, 0.45)', fill: 'rgba(245, 158, 11, 0.16)', label: '4D Polychora' },
  5: { primary: '#ec4899', glow: 'rgba(236, 72, 153, 0.45)', fill: 'rgba(236, 72, 153, 0.16)', label: '5D 5-Polytopes' },
  6: { primary: '#10b981', glow: 'rgba(16, 185, 129, 0.48)', fill: 'rgba(16, 185, 129, 0.18)', label: '6D 6-Polytopes' }
});

function getDimensionStyle(dim) {
  if (DIMENSION_PALETTE[dim]) {
    return DIMENSION_PALETTE[dim];
  }
  const hue = (dim * 67) % 360;
  return {
    primary: `hsl(${hue}, 88%, 62%)`,
    glow: `hsla(${hue}, 88%, 62%, 0.4)`,
    fill: `hsla(${hue}, 88%, 62%, 0.15)`,
    label: `${dim}D Hyper-Stratum`
  };
}

export class SpaceTimeVisualizer {
  /**
   * @param {object} params
   * @param {import('../engine/RustWasmBridge.js').RustWasmUniverse} params.universe
   * @param {HTMLCanvasElement | null} [params.canvas3D=null]
   * @param {HTMLCanvasElement | null} [params.canvas2D=null]
   * @param {ProjectorND} [params.projector]
   * @param {((entityId: number | null) => void) | null} [params.onEntitySelect=null]
   */
  constructor({
    universe,
    canvas3D = null,
    canvas2D = null,
    projector = new ProjectorND(),
    onEntitySelect = null
  } = {}) {
    this.universe = universe;
    this.canvas3D = canvas3D;
    this.canvas2D = canvas2D;
    this.projector = projector;
    this.onEntitySelect = onEntitySelect;

    this.selectedEntityId = null;
    this.showWorldlines = false;
    this.showAxisGizmo = true;
    this.sliceZ = 0.0;
    this.dimensionFilter = 0; // 0 = show all dimensions
    this.autoOrbit = false;

    /** @type {Array<{ entityId: number, u: number, v: number, radius: number }>} */
    this._hitTargets3D = [];
    /** @type {Array<{ entityId: number, u: number, v: number, radius: number }>} */
    this._hitTargets2D = [];

    this._isDragging = false;
    this._dragStartX = 0;
    this._dragStartY = 0;
    this._dragMoved = false;

    if (this.canvas3D) {
      this._bindCanvas3DEvents(this.canvas3D);
    }
    if (this.canvas2D) {
      this._bindCanvas2DEvents(this.canvas2D);
    }
  }

  /**
   * Sets the hyperplane slice Z coordinate for the 2D cross-section viewport
   * and notifies the Rust WASM core so `sliceAtZ` is computed natively in Rust.
   *
   * @param {number} zValue
   */
  setSliceZ(zValue) {
    this.sliceZ = Number.isFinite(zValue) ? zValue : 0.0;
    if (typeof this.universe?.setSliceZ === 'function') {
      this.universe.setSliceZ(this.sliceZ);
    }
  }

  /**
   * Selects a specific entity ID (or null to clear selection).
   *
   * @param {number | null} entityId
   */
  selectEntity(entityId) {
    this.selectedEntityId = entityId;
    if (typeof this.onEntitySelect === 'function') {
      this.onEntitySelect(this.selectedEntityId);
    }
  }

  /**
   * Binds interactive orbit, zoom, and entity selection handlers to the 3D viewport canvas.
   * @private
   */
  _bindCanvas3DEvents(canvas) {
    canvas.addEventListener('pointerdown', (e) => {
      this._isDragging = true;
      this._dragMoved = false;
      this._dragStartX = e.clientX;
      this._dragStartY = e.clientY;
      canvas.setPointerCapture?.(e.pointerId);
    });

    canvas.addEventListener('pointermove', (e) => {
      if (!this._isDragging) return;
      const dx = e.clientX - this._dragStartX;
      const dy = e.clientY - this._dragStartY;
      if (Math.abs(dx) > 2 || Math.abs(dy) > 2) {
        this._dragMoved = true;
      }
      this._dragStartX = e.clientX;
      this._dragStartY = e.clientY;
      this.projector.orbit(-dx * 0.0075, dy * 0.0065);
    });

    const endDrag = (e) => {
      if (!this._isDragging) return;
      this._isDragging = false;
      if (!this._dragMoved) {
        const rect = canvas.getBoundingClientRect();
        const u = e.clientX - rect.left;
        const v = e.clientY - rect.top;
        this._handleHitSelection(u, v, this._hitTargets3D);
      }
    };

    canvas.addEventListener('pointerup', endDrag);
    canvas.addEventListener('pointercancel', () => {
      this._isDragging = false;
    });

    canvas.addEventListener(
      'wheel',
      (e) => {
        e.preventDefault();
        const factor = e.deltaY > 0 ? 1.08 : 0.92;
        this.projector.zoom(factor);
      },
      { passive: false }
    );
  }

  /**
   * Binds click-to-select on the 2D Flatland slice canvas.
   * @private
   */
  _bindCanvas2DEvents(canvas) {
    canvas.addEventListener('click', (e) => {
      const rect = canvas.getBoundingClientRect();
      const u = e.clientX - rect.left;
      const v = e.clientY - rect.top;
      this._handleHitSelection(u, v, this._hitTargets2D);
    });
  }

  /**
   * Picks the closest entity within hit tolerance.
   * @private
   */
  _handleHitSelection(u, v, targets) {
    let bestId = null;
    let bestDist = 26;
    for (let i = 0; i < targets.length; i++) {
      const t = targets[i];
      const dist = Math.hypot(u - t.u, v - t.v);
      const limit = Math.max(18, t.radius + 8);
      if (dist <= limit && dist < bestDist) {
        bestDist = dist;
        bestId = t.entityId;
      }
    }
    this.selectEntity(bestId);
  }

  /**
   * Resizes a canvas element to match its CSS client size and devicePixelRatio.
   * @private
   */
  _prepareCanvas(canvas) {
    const dpr = (typeof window !== 'undefined' && window.devicePixelRatio) || 1;
    const width = Math.max(200, canvas.clientWidth || canvas.width || 800);
    const height = Math.max(160, canvas.clientHeight || canvas.height || 600);
    const targetW = Math.round(width * dpr);
    const targetH = Math.round(height * dpr);
    if (canvas.width !== targetW || canvas.height !== targetH) {
      canvas.width = targetW;
      canvas.height = targetH;
    }
    const ctx = canvas.getContext('2d');
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    return { ctx, width, height };
  }

  /**
   * Renders both the 3D/N-D Omniscient Space-Time Viewport and the 2D Flatland Slice Viewport.
   */
  render() {
    if (this.autoOrbit && !this._isDragging) {
      this.projector.orbit(0.0025, 0);
    }
    if (this.canvas3D) {
      this.render3DViewport();
    }
    if (this.canvas2D) {
      this.render2DViewport();
    }
  }

  /**
   * Renders the Primary 3D / N-D Omniscient Space-Time Viewport directly from the Rust WASM snapshot.
   */
  render3DViewport() {
    const { ctx, width, height } = this._prepareCanvas(this.canvas3D);
    const snap = this.universe.getUniverseSnapshot();
    const universalTime = snap.universalTime;
    const worldBounds = snap.worldBoundsRadius ?? this.universe.config?.worldBoundsRadius ?? 40;
    const bounds = Math.min(worldBounds * 0.45, 18);

    // 1. Deep Space-Time Observatory Background
    const bgGrad = ctx.createRadialGradient(
      width * 0.5,
      height * 0.45,
      20,
      width * 0.5,
      height * 0.5,
      Math.max(width, height) * 0.75
    );
    bgGrad.addColorStop(0, '#0b1329');
    bgGrad.addColorStop(0.55, '#060a17');
    bgGrad.addColorStop(1, '#02040a');
    ctx.fillStyle = bgGrad;
    ctx.fillRect(0, 0, width, height);

    // 2. Render Stacked Dimensional Hyperplanes (z=0 Flatland, z=sliceZ, z>0 3D & 4D+ strata)
    this._drawHyperplaneGrid3D(ctx, width, height, bounds);

    // 3. Render N-D Coordinate Axis Gizmo
    if (this.showAxisGizmo) {
      this._drawAxisGizmo3D(ctx, width, height, bounds * 0.55);
    }

    // 4. Render Expanding Higher-Dimensional Information Pulses & Light-Cones
    this._drawInformationPulses3D(ctx, width, height, snap.pulses || [], universalTime);

    // 5. Render Worldlines + Peer Communication Links from Rust WASM beings
    const beings = snap.beings || [];
    if (this.showWorldlines) {
      this._drawWorldlines3D(ctx, width, height, beings, universalTime);
    }

    this._drawPeerLinks3D(ctx, width, height, beings, universalTime);

    // 6. Depth-sort beings back-to-front and render their N-D Polytope bodies
    const renderQueue = [];
    for (let i = 0; i < beings.length; i++) {
      const being = beings[i];
      if (
        this.dimensionFilter > 0 &&
        being.activeDimension !== this.dimensionFilter &&
        being.maxDimension !== this.dimensionFilter
      ) {
        continue;
      }

      const centerScreen = this.projector.projectNDToScreen(
        being.position,
        width,
        height,
        {
          universalTime,
          sampleTime: universalTime
        }
      );

      renderQueue.push({
        being,
        centerScreen
      });
    }

    renderQueue.sort((a, b) => b.centerScreen.depth - a.centerScreen.depth);

    this._hitTargets3D = [];

    for (let i = 0; i < renderQueue.length; i++) {
      const item = renderQueue[i];
      this._drawBeingPolytope3D(ctx, width, height, item, universalTime);
    }

    // 7. Viewport Overlay Telemetry Badge
    this._draw3DOverlayHeader(ctx, width, height, universalTime);
  }

  /**
   * Draws the translucent 2D Flatland hyperplane grid (z=0), active slice plane (z=sliceZ),
   * and higher-dimensional strata halos in the 3D viewport.
   * @private
   */
  _drawHyperplaneGrid3D(ctx, width, height, bounds) {
    const gridSteps = 6;
    const stepSize = bounds / gridSteps;

    // Draw z = 0 Flatland Grid Lines
    ctx.save();
    ctx.lineWidth = 1;
    for (let i = -gridSteps; i <= gridSteps; i++) {
      const coord = i * stepSize;
      const isAxis = i === 0;
      ctx.strokeStyle = isAxis
        ? 'rgba(56, 189, 248, 0.38)'
        : 'rgba(56, 189, 248, 0.13)';

      const a1 = this.projector.project3DTo2D({ x: -bounds, y: coord, z: 0 }, width, height);
      const a2 = this.projector.project3DTo2D({ x: bounds, y: coord, z: 0 }, width, height);
      if (a1.visible && a2.visible) {
        ctx.beginPath();
        ctx.moveTo(a1.u, a1.v);
        ctx.lineTo(a2.u, a2.v);
        ctx.stroke();
      }

      const b1 = this.projector.project3DTo2D({ x: coord, y: -bounds, z: 0 }, width, height);
      const b2 = this.projector.project3DTo2D({ x: coord, y: bounds, z: 0 }, width, height);
      if (b1.visible && b2.visible) {
        ctx.beginPath();
        ctx.moveTo(b1.u, b1.v);
        ctx.lineTo(b2.u, b2.v);
        ctx.stroke();
      }
    }

    // Draw 3D Stratum Reference Ring at z = 5.5 and 4D Stratum Ring at z = 10.0
    const strata = [
      { z: 0, color: 'rgba(56, 189, 248, 0.45)', label: '2D FLATLAND HYPERPLANE (z = 0)' },
      { z: 5.5, color: 'rgba(168, 85, 247, 0.28)', label: '3D POLYHEDRAL STRATUM (x₂ > 0)' },
      { z: 9.8, color: 'rgba(245, 158, 11, 0.24)', label: '4D+ POLYCHORON STRATUM (x₃, x₄ > 0)' }
    ];

    if (Math.abs(this.sliceZ) > 0.05) {
      strata.push({
        z: this.sliceZ,
        color: 'rgba(244, 63, 94, 0.55)',
        label: `ACTIVE SLICING HYPERPLANE (z = ${this.sliceZ.toFixed(2)})`
      });
    }

    for (const stratum of strata) {
      ctx.strokeStyle = stratum.color;
      ctx.lineWidth = stratum.z === 0 ? 1.5 : 1.1;
      ctx.beginPath();
      const segs = 48;
      let firstPt = null;
      for (let s = 0; s <= segs; s++) {
        const theta = (2 * Math.PI * s) / segs;
        const pt = this.projector.project3DTo2D(
          {
            x: bounds * Math.cos(theta),
            y: bounds * Math.sin(theta),
            z: stratum.z
          },
          width,
          height
        );
        if (s === 0) {
          ctx.moveTo(pt.u, pt.v);
          firstPt = pt;
        } else {
          ctx.lineTo(pt.u, pt.v);
        }
      }
      ctx.stroke();

      if (firstPt && firstPt.visible) {
        ctx.fillStyle = stratum.color;
        ctx.font = '600 10px "JetBrains Mono", "SF Mono", monospace';
        ctx.fillText(stratum.label, firstPt.u + 8, firstPt.v - 4);
      }
    }

    ctx.restore();
  }

  /**
   * Renders the N-dimensional coordinate axis gizmo at the world origin.
   * @private
   */
  _drawAxisGizmo3D(ctx, width, height, axisLength) {
    const maxDim = Math.min(this.universe.config.maxUniverseDimension, 6);
    const axisLabels = ['x₀ (X)', 'x₁ (Y)', 'x₂ (Z/T)', 'x₃ (W)', 'x₄ (V)', 'x₅ (U)'];
    const axisColors = ['#38bdf8', '#22d3ee', '#a855f7', '#f59e0b', '#ec4899', '#10b981'];

    const originVec = new Float64Array(maxDim);
    const originScreen = this.projector.projectNDToScreen(originVec, width, height);
    if (!originScreen.visible) return;

    ctx.save();
    for (let d = 0; d < maxDim; d++) {
      const tipVec = new Float64Array(maxDim);
      tipVec[d] = axisLength * (d >= 3 ? 0.85 : 1.0);
      const tipScreen = this.projector.projectNDToScreen(tipVec, width, height);
      if (!tipScreen.visible) continue;

      ctx.strokeStyle = axisColors[d % axisColors.length];
      ctx.lineWidth = d < 3 ? 1.8 : 1.4;
      if (d >= 3) {
        ctx.setLineDash([4, 3]);
      } else {
        ctx.setLineDash([]);
      }

      ctx.beginPath();
      ctx.moveTo(originScreen.u, originScreen.v);
      ctx.lineTo(tipScreen.u, tipScreen.v);
      ctx.stroke();

      ctx.fillStyle = axisColors[d % axisColors.length];
      ctx.font = '600 10px "JetBrains Mono", monospace';
      ctx.fillText(axisLabels[d] ?? `x_${d}`, tipScreen.u + 4, tipScreen.v + 3);
    }
    ctx.restore();
  }

  /**
   * Renders expanding causal InformationPulse wavefronts and vertical injection light-cones.
   * @private
   */
  _drawInformationPulses3D(ctx, width, height, pulses, universalTime) {
    ctx.save();
    for (let i = 0; i < pulses.length; i++) {
      const pulse = pulses[i];
      if (!pulse) continue;

      const originCoords = pulse.origin || [0, 0, 0];
      const ox = originCoords[0] || 0;
      const oy = originCoords[1] || 0;
      const targetZ = pulse.targetDimension === 2 ? 0 : (originCoords[2] || 4.5);
      const sourceZ = targetZ + 6.0;

      const progress = Math.min(1, pulse.currentRadius / Math.max(1, pulse.maxRadius));
      const alpha = Math.max(0.08, (1 - progress) * 0.78);

      const style = getDimensionStyle(pulse.sourceDimension);

      // 1. Vertical Light-Cone from Higher Dimension down to Target Hyperplane
      const emitterScreen = this.projector.project3DTo2D({ x: ox, y: oy, z: sourceZ }, width, height);
      const centerScreen = this.projector.project3DTo2D({ x: ox, y: oy, z: targetZ }, width, height);

      if (emitterScreen.visible && centerScreen.visible) {
        const beamGrad = ctx.createLinearGradient(
          emitterScreen.u,
          emitterScreen.v,
          centerScreen.u,
          centerScreen.v
        );
        beamGrad.addColorStop(0, `rgba(245, 158, 11, ${alpha})`);
        beamGrad.addColorStop(1, `rgba(56, 189, 248, ${alpha * 0.3})`);

        ctx.strokeStyle = beamGrad;
        ctx.lineWidth = 2.0;
        ctx.setLineDash([5, 4]);
        ctx.beginPath();
        ctx.moveTo(emitterScreen.u, emitterScreen.v);
        ctx.lineTo(centerScreen.u, centerScreen.v);
        ctx.stroke();
        ctx.setLineDash([]);
      }

      // 2. Expanding Wavefront Ring on the Target Hyperplane
      const segments = 48;
      ctx.strokeStyle = style.primary;
      ctx.globalAlpha = alpha;
      ctx.lineWidth = 2.2;
      ctx.beginPath();
      let labelPt = null;

      for (let s = 0; s <= segments; s++) {
        const theta = (2 * Math.PI * s) / segments;
        const wx = ox + pulse.currentRadius * Math.cos(theta);
        const wy = oy + pulse.currentRadius * Math.sin(theta);
        const pt = this.projector.project3DTo2D({ x: wx, y: wy, z: targetZ }, width, height);
        if (s === 0) {
          ctx.moveTo(pt.u, pt.v);
          labelPt = pt;
        } else {
          ctx.lineTo(pt.u, pt.v);
        }
      }
      ctx.stroke();

      if (labelPt && labelPt.visible) {
        ctx.font = '600 10px "JetBrains Mono", monospace';
        ctx.fillStyle = style.primary;
        ctx.fillText(
          `⚡ ${pulse.sourceDimension}D→${pulse.targetDimension}D [${pulse.axiom}]`,
          labelPt.u + 6,
          labelPt.v - 4
        );
      }
    }
    ctx.restore();
  }

  /**
   * Renders space-time worldline trajectories (`being.worldline`) for all beings.
   * @private
   */
  _drawWorldlines3D(ctx, width, height, beings, universalTime) {
    ctx.save();

    for (let i = 0; i < beings.length; i++) {
      const being = beings[i];
      const samples = being.worldline;
      if (!samples || samples.length < 2) continue;

      if (this.dimensionFilter > 0 && being.activeDimension !== this.dimensionFilter) {
        continue;
      }

      const isSelected = being.entityId === this.selectedEntityId;
      const style = getDimensionStyle(being.maxDimension ?? 2);

      ctx.strokeStyle = style.primary;
      ctx.lineWidth = isSelected ? 2.4 : 1.25;
      ctx.globalAlpha = isSelected ? 0.85 : this.projector.spaceTimeMode ? 0.58 : 0.32;

      ctx.beginPath();
      let started = false;
      for (let s = 0; s < samples.length; s++) {
        const sample = samples[s];
        const pt = this.projector.projectNDToScreen(
          sample.coordinates,
          width,
          height,
          {
            sampleTime: sample.time,
            universalTime,
            spaceTimeMode: this.projector.spaceTimeMode
          }
        );
        if (!pt.visible) continue;
        if (!started) {
          ctx.moveTo(pt.u, pt.v);
          started = true;
        } else {
          ctx.lineTo(pt.u, pt.v);
        }
      }
      ctx.stroke();
    }

    ctx.restore();
  }

  /**
   * Renders peer-to-peer geometric axiom exchange beams (`being.activeLinks`).
   * Deduplicates bidirectional links between unordered entity pairs (`REQ-PERF-001`).
   * @private
   */
  _drawPeerLinks3D(ctx, width, height, beings, universalTime) {
    ctx.save();

    const byId = new Map();
    for (let i = 0; i < beings.length; i++) {
      byId.set(beings[i].entityId, beings[i]);
    }

    const drawnPairs = new Set();

    for (let i = 0; i < beings.length; i++) {
      const being = beings[i];
      const links = being.activeLinks || being.collaborationLinks || [];
      if (links.length === 0) continue;

      let screenA = null;

      for (let l = 0; l < links.length; l++) {
        const link = links[l];
        const age = universalTime - (link.timestamp ?? 0);
        if (age > 2.5) continue;

        const idA = being.entityId;
        const idB = link.partnerId;
        const pairKey = idA < idB ? `${idA}:${idB}` : `${idB}:${idA}`;
        if (drawnPairs.has(pairKey)) continue;

        const partner = byId.get(idB);
        if (!partner) continue;

        if (!screenA) {
          screenA = this.projector.projectNDToScreen(being.position, width, height, {
            universalTime,
            sampleTime: universalTime
          });
        }
        if (!screenA.visible) break;

        const screenB = this.projector.projectNDToScreen(
          partner.position,
          width,
          height,
          {
            universalTime,
            sampleTime: universalTime
          }
        );
        if (!screenB.visible) continue;

        drawnPairs.add(pairKey);

        const alpha = Math.max(0.15, 1.0 - age / 2.5);
        ctx.strokeStyle = `rgba(56, 189, 248, ${alpha * 0.75})`;
        ctx.lineWidth = 1.6;
        ctx.beginPath();
        ctx.moveTo(screenA.u, screenA.v);
        ctx.lineTo(screenB.u, screenB.v);
        ctx.stroke();

        // Animated data packet traveling along link
        const t = ((universalTime * 2.5) % 1 + 1) % 1;
        const pu = screenA.u + (screenB.u - screenA.u) * t;
        const pv = screenA.v + (screenB.v - screenA.v) * t;
        ctx.fillStyle = '#fde047';
        ctx.beginPath();
        ctx.arc(pu, pv, 2.6, 0, Math.PI * 2);
        ctx.fill();
      }
    }

    ctx.restore();
  }

  /**
   * Renders an individual N-dimensional being's facets, edges, vertices (already rotated & translated
   * in the Rust WASM core), projection shadow onto Flatland, and epistemic badges.
   * @private
   */
  _drawBeingPolytope3D(ctx, width, height, item, universalTime) {
    const { being, centerScreen } = item;
    if (!centerScreen.visible || !being.vertices || being.vertices.length === 0) return;

    const entityId = being.entityId;
    const isSelected = entityId === this.selectedEntityId;
    const maxDim = being.maxDimension;
    const activeDim = being.activeDimension;
    const style = getDimensionStyle(maxDim);

    // 1. If being is in 3D+ (z != 0), draw a vertical projection tether down to the z=0 Flatland plane
    const posArr = being.position;
    const flatShadow = this.projector.project3DTo2D(
      { x: posArr[0] || 0, y: posArr[1] || 0, z: 0 },
      width,
      height
    );

    if (flatShadow.visible && Math.abs(centerScreen.point3D.z) > 0.35) {
      ctx.save();
      ctx.strokeStyle = style.glow;
      ctx.lineWidth = isSelected ? 1.6 : 1.0;
      ctx.setLineDash([3, 4]);
      ctx.beginPath();
      ctx.moveTo(centerScreen.u, centerScreen.v);
      ctx.lineTo(flatShadow.u, flatShadow.v);
      ctx.stroke();

      ctx.fillStyle = style.glow;
      ctx.beginPath();
      ctx.arc(flatShadow.u, flatShadow.v, 3.2, 0, Math.PI * 2);
      ctx.fill();
      ctx.restore();
    }

    // 2. Project Rust-computed world-space N-D vertices to 2D screen coordinates
    const projectedVerts = being.vertices.map((v) =>
      this.projector.projectNDToScreen(v, width, height, {
        universalTime,
        sampleTime: universalTime
      })
    );

    // 3. Draw translucent polygon/polytope facets
    ctx.save();
    ctx.fillStyle = style.fill;
    if (maxDim === 2 && projectedVerts.length >= 3) {
      ctx.beginPath();
      for (let k = 0; k < projectedVerts.length; k++) {
        const pv = projectedVerts[k];
        if (!pv) continue;
        if (k === 0) ctx.moveTo(pv.u, pv.v);
        else ctx.lineTo(pv.u, pv.v);
      }
      ctx.closePath();
      ctx.fill();
    } else if (maxDim <= 4 && Array.isArray(being.facets) && being.facets.length > 0 && being.facets.length <= 16) {
      const maxFacetsToFill = Math.min(being.facets.length, 12);
      for (let f = 0; f < maxFacetsToFill; f++) {
        const facetIndices = being.facets[f];
        if (!facetIndices || facetIndices.length < 3) continue;
        ctx.beginPath();
        for (let k = 0; k < Math.min(facetIndices.length, 6); k++) {
          const pv = projectedVerts[facetIndices[k]];
          if (!pv) continue;
          if (k === 0) ctx.moveTo(pv.u, pv.v);
          else ctx.lineTo(pv.u, pv.v);
        }
        ctx.closePath();
        ctx.fill();
      }
    }

    // 4. Draw 1-skeleton wireframe edges with w-depth spectral shading for 4D/5D/6D polytopes
    const edges = being.edges || [];
    for (let e = 0; e < edges.length; e++) {
      const [uIdx, vIdx] = edges[e];
      const pu = projectedVerts[uIdx];
      const pv = projectedVerts[vIdx];
      if (!pu || !pv || !pu.visible || !pv.visible) continue;

      if (maxDim >= 4) {
        const avgW = (pu.wDepth + pv.wDepth) * 0.5;
        const hueShift = ((being.colorHue || 40) + avgW * 48 + 360) % 360;
        ctx.strokeStyle = `hsla(${hueShift}, 92%, 65%, ${isSelected ? 0.95 : 0.78})`;
      } else {
        ctx.strokeStyle = style.primary;
      }

      ctx.lineWidth = isSelected ? 2.2 : maxDim >= 4 ? 1.45 : 1.7;
      ctx.beginPath();
      ctx.moveTo(pu.u, pu.v);
      ctx.lineTo(pv.u, pv.v);
      ctx.stroke();
    }

    // 5. Draw vertices
    ctx.fillStyle = '#f8fafc';
    for (let v = 0; v < projectedVerts.length; v++) {
      const pv = projectedVerts[v];
      if (!pv.visible) continue;
      ctx.beginPath();
      ctx.arc(pv.u, pv.v, isSelected ? 2.6 : 1.9, 0, Math.PI * 2);
      ctx.fill();
    }

    // 6. Draw Epistemic Axiom Orbit Pips around the Being
    const axiomCount = (being.knownAxioms || []).length;
    const pipOrbitRadius = Math.max(12, centerScreen.scale * 1.45);
    for (let a = 0; a < axiomCount; a++) {
      const angle = universalTime * 1.6 + (2 * Math.PI * a) / Math.max(1, axiomCount);
      const px = centerScreen.u + pipOrbitRadius * Math.cos(angle);
      const py = centerScreen.v + pipOrbitRadius * Math.sin(angle);
      ctx.fillStyle = '#fde047';
      ctx.beginPath();
      ctx.arc(px, py, 2.3, 0, Math.PI * 2);
      ctx.fill();
    }

    // 7. Selection Reticle & Label
    if (isSelected) {
      ctx.strokeStyle = '#38bdf8';
      ctx.lineWidth = 1.8;
      ctx.setLineDash([4, 3]);
      ctx.beginPath();
      ctx.arc(centerScreen.u, centerScreen.v, pipOrbitRadius + 7, 0, Math.PI * 2);
      ctx.stroke();
      ctx.setLineDash([]);
    }

    const phaseTag =
      activeDim < maxDim ? ` [PHASED→${activeDim}D]` : `${maxDim}D`;
    ctx.font = isSelected
      ? '700 11px "JetBrains Mono", monospace'
      : '500 10px "JetBrains Mono", monospace';
    ctx.fillStyle = isSelected ? '#ffffff' : 'rgba(226, 232, 240, 0.88)';
    ctx.fillText(
      `#${entityId} ${phaseTag} ${being.schlafliSymbol}`,
      centerScreen.u + 10,
      centerScreen.v - 8
    );

    ctx.restore();

    this._hitTargets3D.push({
      entityId,
      u: centerScreen.u,
      v: centerScreen.v,
      radius: pipOrbitRadius
    });
  }

  /**
   * Draws top-left HUD telemetry watermark inside the 3D viewport.
   * @private
   */
  _draw3DOverlayHeader(ctx, width, height, universalTime) {
    ctx.save();
    ctx.font = '600 11px "JetBrains Mono", monospace';
    ctx.fillStyle = 'rgba(148, 163, 184, 0.9)';
    const modeLabel = this.projector.spaceTimeMode
      ? 'MODE: SPACE-TIME WORLDLINE CORKSCREW (Z = Proper Time ΔT)'
      : 'MODE: N-D SPATIAL HYPER-STRATA (Z = x₂, W = x₃, V = x₄, U = x₅)';
    ctx.fillText(modeLabel, 14, height - 14);
    ctx.restore();
  }

  /**
   * Renders the 2D Flatland Cross-Section Slice Viewport (`canvas2D`) at hyperplane x_2 = this.sliceZ
   * using exact cross-section polygons (`being.sliceAtZ`) computed natively by the Rust WASM core.
   */
  render2DViewport() {
    const { ctx, width, height } = this._prepareCanvas(this.canvas2D);
    const snap = this.universe.getUniverseSnapshot();
    const worldBounds = snap.worldBoundsRadius ?? this.universe.config?.worldBoundsRadius ?? 40;

    // 1. Background
    ctx.fillStyle = '#050914';
    ctx.fillRect(0, 0, width, height);

    const cx = width * 0.5;
    const cy = height * 0.5;
    const viewWorldRadius = Math.min(worldBounds * 0.45, 18);
    const pxPerUnit = (Math.min(width, height) * 0.44) / viewWorldRadius;

    const toScreen2D = (x, y) => ({
      u: cx + x * pxPerUnit,
      v: cy - y * pxPerUnit
    });

    // 2. Polar & Cartesian Grid on the 2D Slice Plane
    ctx.save();
    ctx.strokeStyle = 'rgba(56, 189, 248, 0.12)';
    ctx.lineWidth = 1;
    const gridStep = 3;
    for (let g = -viewWorldRadius; g <= viewWorldRadius; g += gridStep) {
      const p1 = toScreen2D(g, -viewWorldRadius);
      const p2 = toScreen2D(g, viewWorldRadius);
      ctx.beginPath();
      ctx.moveTo(p1.u, p1.v);
      ctx.lineTo(p2.u, p2.v);
      ctx.stroke();

      const q1 = toScreen2D(-viewWorldRadius, g);
      const q2 = toScreen2D(viewWorldRadius, g);
      ctx.beginPath();
      ctx.moveTo(q1.u, q1.v);
      ctx.lineTo(q2.u, q2.v);
      ctx.stroke();
    }

    // Concentric Range Circles
    for (const r of [6, 12, viewWorldRadius]) {
      ctx.strokeStyle =
        r === viewWorldRadius ? 'rgba(56, 189, 248, 0.38)' : 'rgba(56, 189, 248, 0.14)';
      ctx.beginPath();
      ctx.arc(cx, cy, r * pxPerUnit, 0, Math.PI * 2);
      ctx.stroke();
    }

    // 3. Render Information Pulses on the 2D Plane
    const pulses = snap.pulses || [];
    for (let i = 0; i < pulses.length; i++) {
      const pulse = pulses[i];
      if (!pulse || pulse.targetDimension !== 2) continue;
      const originCoords = pulse.origin || [0, 0];
      const center = toScreen2D(originCoords[0] || 0, originCoords[1] || 0);
      const progress = Math.min(1, pulse.currentRadius / Math.max(1, pulse.maxRadius));
      const alpha = Math.max(0.1, (1 - progress) * 0.75);

      ctx.strokeStyle = `rgba(245, 158, 11, ${alpha})`;
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.arc(center.u, center.v, pulse.currentRadius * pxPerUnit, 0, Math.PI * 2);
      ctx.stroke();
    }

    // 4. Render Native 2D Beings and Rust-Computed 2D Cross-Sections (`sliceAtZ`) of Higher-D Beings
    const beings = snap.beings || [];
    this._hitTargets2D = [];

    for (let i = 0; i < beings.length; i++) {
      const being = beings[i];
      if (!being || !being.vertices) continue;

      const entityId = being.entityId;
      const isSelected = entityId === this.selectedEntityId;
      const maxDim = being.maxDimension;
      const style = getDimensionStyle(maxDim);
      const worldVertices = being.vertices;

      // Case A: Native 2D Polygon Being on z = 0
      if (maxDim === 2 && Math.abs(this.sliceZ) <= 0.75) {
        const center2D = toScreen2D(being.position[0] || 0, being.position[1] || 0);

        ctx.fillStyle = style.fill;
        ctx.strokeStyle = isSelected ? '#ffffff' : style.primary;
        ctx.lineWidth = isSelected ? 2.4 : 1.8;

        ctx.beginPath();
        for (let v = 0; v < worldVertices.length; v++) {
          const sv = toScreen2D(worldVertices[v][0] || 0, worldVertices[v][1] || 0);
          if (v === 0) ctx.moveTo(sv.u, sv.v);
          else ctx.lineTo(sv.u, sv.v);
        }
        ctx.closePath();
        ctx.fill();
        ctx.stroke();

        // Velocity vector indicator
        const vel = being.velocity || [0, 0];
        const velTip = toScreen2D(
          (being.position[0] || 0) + (vel[0] || 0) * 0.25,
          (being.position[1] || 0) + (vel[1] || 0) * 0.25
        );
        ctx.strokeStyle = 'rgba(56, 189, 248, 0.55)';
        ctx.lineWidth = 1.2;
        ctx.beginPath();
        ctx.moveTo(center2D.u, center2D.v);
        ctx.lineTo(velTip.u, velTip.v);
        ctx.stroke();

        ctx.font = '600 10px "JetBrains Mono", monospace';
        ctx.fillStyle = '#e2e8f0';
        ctx.fillText(
          `#${entityId} {${being.sides}}`,
          center2D.u + 9,
          center2D.v - 7
        );

        this._hitTargets2D.push({
          entityId,
          u: center2D.u,
          v: center2D.v,
          radius: 1.35 * pxPerUnit
        });
        continue;
      }

      // Case B: Higher-Dimensional Being (M >= 3) — Render Rust-Computed Cross-Section (`sliceAtZ`)
      if (maxDim >= 3) {
        const sliceResult = being.sliceAtZ || being.crossSection;

        if (sliceResult && sliceResult.intersects && (sliceResult.orderedPolygon2D || []).length >= 2) {
          const cArr = sliceResult.centroid || being.position || [0, 0];
          const centroid2D = toScreen2D(cArr[0] || 0, cArr[1] || 0);

          // Glowing hyper-dimensional cross-section aura
          ctx.save();
          ctx.fillStyle = style.fill;
          ctx.strokeStyle = style.primary;
          ctx.lineWidth = isSelected ? 2.6 : 2.0;

          ctx.beginPath();
          for (let p = 0; p < sliceResult.orderedPolygon2D.length; p++) {
            const [px, py] = sliceResult.orderedPolygon2D[p];
            const sp = toScreen2D(px, py);
            if (p === 0) ctx.moveTo(sp.u, sp.v);
            else ctx.lineTo(sp.u, sp.v);
          }
          ctx.closePath();
          ctx.fill();
          ctx.stroke();

          // Outer dashed visitor ring showing Flatlanders are seeing a higher-D visitor slice
          ctx.setLineDash([4, 3]);
          ctx.strokeStyle = style.glow;
          ctx.beginPath();
          ctx.arc(
            centroid2D.u,
            centroid2D.v,
            Math.max(10, (sliceResult.boundingRadius || 1.2) * pxPerUnit + 5),
            0,
            Math.PI * 2
          );
          ctx.stroke();
          ctx.setLineDash([]);

          ctx.font = '700 10px "JetBrains Mono", monospace';
          ctx.fillStyle = style.primary;
          ctx.fillText(
            `SLICE #${entityId} (${maxDim}D→2D: ${(sliceResult.points || []).length}-gon)`,
            centroid2D.u + 10,
            centroid2D.v - 8
          );
          ctx.restore();

          this._hitTargets2D.push({
            entityId,
            u: centroid2D.u,
            v: centroid2D.v,
            radius: Math.max(12, (sliceResult.boundingRadius || 1.2) * pxPerUnit)
          });
        } else {
          // Draw faint ghost projection of overhead higher-D being onto Flatland
          const ghostPos = toScreen2D(being.position[0] || 0, being.position[1] || 0);
          ctx.fillStyle = style.glow;
          ctx.beginPath();
          ctx.arc(ghostPos.u, ghostPos.v, 3, 0, Math.PI * 2);
          ctx.fill();
        }
      }
    }

    // 5. Slice Viewport Header Overlay
    ctx.font = '600 10px "JetBrains Mono", monospace';
    ctx.fillStyle = 'rgba(148, 163, 184, 0.9)';
    ctx.fillText(
      `2D HYPERPLANE SLICE AT x₂ (z) = ${this.sliceZ.toFixed(2)} (RUST CORE)`,
      12,
      height - 10
    );

    ctx.restore();
  }
}
