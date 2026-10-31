/**
 * @fileoverview Space-Time Control HUD & Evolutionary Telemetry Controller (`ControlHUD`).
 * Binds time-warp controls (0x Pause, +1 Step, 1x Real-Time [1 sec/sec], 5x, 20x, 100x, 500x),
 * Space-Time Worldline Mode toggles, hyperplane slice controls, manual Higher-D Information Pulse
 * injection, Dimensional Phase Jump controls, and live population/event telemetry.
 * Implements REQ-VIS-002, REQ-FOUND-002, and REQ-FOUND-003.
 * License: MIT
 */

export class ControlHUD {
  /**
   * @param {object} params
   * @param {import('../engine/RustWasmBridge.js').RustWasmUniverse} params.universe
   * @param {import('../render/SpaceTimeVisualizer.js').SpaceTimeVisualizer} params.visualizer
   * @param {Document | HTMLElement} [params.root=document]
   * @param {(() => void) | null} [params.onResetUniverse=null]
   */
  constructor({
    universe,
    visualizer,
    root = typeof document !== 'undefined' ? document : null,
    onResetUniverse = null
  } = {}) {
    this.universe = universe;
    this.visualizer = visualizer;
    this.root = root;
    this.onResetUniverse = onResetUniverse;
    this._lastEventCount = 0;
    this._lastInspectorSignature = '';
    this._lastEventFeedSignature = '';

    if (this.root) {
      this._bindControls();
      this.update();
    }
  }

  /**
   * Queries an element inside `this.root`.
   * @private
   */
  _qs(selector) {
    return this.root?.querySelector?.(selector) ?? null;
  }

  /**
   * Queries all matching elements inside `this.root`.
   * @private
   */
  _qsa(selector) {
    return Array.from(this.root?.querySelectorAll?.(selector) ?? []);
  }

  /**
   * Wires up all DOM event listeners for time warp, view modes, spawning, injection, and phasing.
   * @private
   */
  _bindControls() {
    // 1. Preset Time-Warp Speed Buttons (0x, 1x, 5x, 20x, 100x, 500x)
    const speedBtns = this._qsa('[data-time-scale]');
    for (const btn of speedBtns) {
      btn.addEventListener('click', () => {
        const scale = Number(btn.getAttribute('data-time-scale'));
        if (Number.isFinite(scale)) {
          this.universe.setTimeScale(scale);
          const slider = this._qs('#time-scale-slider');
          if (slider) {
            slider.value = String(Math.min(500, scale));
          }
          this.update();
        }
      });
    }

    // Single-Tick Step Button
    const stepBtn = this._qs('#btn-step-tick');
    if (stepBtn) {
      stepBtn.addEventListener('click', () => {
        this.universe.setTimeScale(0);
        this.universe.stepTicks(1);
        this.visualizer.render();
        this.update();
      });
    }

    // Slow Down Proper Time (-) & Speed Up Proper Time (+) Buttons
    const slowerBtn = this._qs('#btn-time-slower');
    if (slowerBtn) {
      slowerBtn.addEventListener('click', () => {
        const next = this.universe.slowDownTime();
        const slider = this._qs('#time-scale-slider');
        if (slider) slider.value = String(Math.min(500, next));
        this.update();
      });
    }

    const fasterBtn = this._qs('#btn-time-faster');
    if (fasterBtn) {
      fasterBtn.addEventListener('click', () => {
        const next = this.universe.speedUpTime();
        const slider = this._qs('#time-scale-slider');
        if (slider) slider.value = String(Math.min(500, next));
        this.update();
      });
    }

    // Keyboard shortcuts: '-' / '_' slows down proper time, '+' / '=' speeds up proper time
    if (typeof window !== 'undefined' && typeof window.addEventListener === 'function') {
      window.addEventListener('keydown', (e) => {
        const tag = e.target?.tagName?.toLowerCase?.() ?? '';
        if (tag === 'input' || tag === 'select' || tag === 'textarea') return;
        if (e.key === '-' || e.key === '_') {
          const next = this.universe.slowDownTime();
          const slider = this._qs('#time-scale-slider');
          if (slider) slider.value = String(Math.min(500, next));
          this.update();
        } else if (e.key === '+' || e.key === '=') {
          const next = this.universe.speedUpTime();
          const slider = this._qs('#time-scale-slider');
          if (slider) slider.value = String(Math.min(500, next));
          this.update();
        }
      });
    }

    // Continuous Time-Scale Slider
    const timeSlider = this._qs('#time-scale-slider');
    if (timeSlider) {
      timeSlider.addEventListener('input', (e) => {
        const val = Number(e.target.value);
        if (Number.isFinite(val)) {
          this.universe.setTimeScale(val);
          this.update();
        }
      });
    }

    // 2. Space-Time Worldline Mode & Trail Toggles
    const stModeCheckbox = this._qs('#toggle-spacetime-mode');
    if (stModeCheckbox) {
      stModeCheckbox.addEventListener('change', (e) => {
        const enabled = Boolean(e.target.checked);
        this.visualizer.projector.setSpaceTimeMode(enabled);
        if (enabled) {
          this.visualizer.showWorldlines = true;
          if (worldlineCheckbox) worldlineCheckbox.checked = true;
        }
        this.universe.setIncludeWorldlines?.(
          Boolean(this.visualizer.showWorldlines || enabled)
        );
        this.visualizer.render();
      });
    }

    const worldlineCheckbox = this._qs('#toggle-worldlines');
    if (worldlineCheckbox) {
      worldlineCheckbox.addEventListener('change', (e) => {
        const enabled = Boolean(e.target.checked);
        this.visualizer.showWorldlines = enabled;
        this.universe.setIncludeWorldlines?.(enabled);
        this.visualizer.render();
      });
    }

    const stToggleBtn = this._qs('#btn-toggle-spacetime');
    if (stToggleBtn) {
      stToggleBtn.addEventListener('click', () => {
        const next = !this.visualizer.showWorldlines;
        this.visualizer.showWorldlines = next;
        this.visualizer.projector.setSpaceTimeMode(next);
        this.universe.setIncludeWorldlines?.(next);
        stToggleBtn.classList.toggle('active', next);
        this.visualizer.render();
      });
    }

    const autoOrbitCheckbox = this._qs('#toggle-auto-orbit');
    if (autoOrbitCheckbox) {
      autoOrbitCheckbox.addEventListener('change', (e) => {
        this.visualizer.autoOrbit = Boolean(e.target.checked);
      });
    }

    // 3. Hyperplane Slice Z Slider
    const sliceSlider = this._qs('#slice-z-slider');
    const sliceValueLabel = this._qs('#slice-z-value');
    if (sliceSlider) {
      sliceSlider.addEventListener('input', (e) => {
        const zVal = Number(e.target.value) || 0;
        this.visualizer.setSliceZ(zVal);
        if (sliceValueLabel) {
          sliceValueLabel.textContent = zVal.toFixed(2);
        }
        this.visualizer.render();
      });
    }

    // 4. Dimension Filter Buttons
    const dimFilterBtns = this._qsa('[data-dim-filter]');
    for (const btn of dimFilterBtns) {
      btn.addEventListener('click', () => {
        const dim = Number(btn.getAttribute('data-dim-filter')) || 0;
        this.visualizer.dimensionFilter = dim;
        for (const b of dimFilterBtns) {
          b.classList.toggle('active', b === btn);
        }
        this.visualizer.render();
      });
    }

    // 5. Observer / God Interventions: Inject Pulse & Spawn Beings
    const injectBtn = this._qs('#btn-inject-pulse');
    if (injectBtn) {
      injectBtn.addEventListener('click', () => {
        const sourceSelect = this._qs('#inject-source-dim');
        const sourceDim = Number(sourceSelect?.value) || 3;
        const targetDim = Math.max(2, sourceDim - 1);
        this.universe.triggerRandomInjection(sourceDim, targetDim);
        this.visualizer.render();
        this.update();
      });
    }

    const spawn2DBtn = this._qs('#btn-spawn-2d');
    if (spawn2DBtn) {
      spawn2DBtn.addEventListener('click', () => {
        const sides = 3 + Math.floor(Math.random() * 4);
        const id = this.universe.spawnBeing({
          dimension: 2,
          maxDimension: 2,
          family: 'polygon',
          sides,
          energy: 55
        });
        this.visualizer.selectEntity(id);
        this.visualizer.render();
        this.update();
      });
    }

    const spawn3DBtn = this._qs('#btn-spawn-3d');
    if (spawn3DBtn) {
      spawn3DBtn.addEventListener('click', () => {
        const families = ['simplex', 'hypercube', 'orthoplex'];
        const family = families[Math.floor(Math.random() * families.length)];
        const id = this.universe.spawnBeing({
          dimension: 3,
          maxDimension: 3,
          family,
          sides: 4,
          energy: 65
        });
        this.visualizer.selectEntity(id);
        this.visualizer.render();
        this.update();
      });
    }

    const spawn4DBtn = this._qs('#btn-spawn-4d');
    if (spawn4DBtn) {
      spawn4DBtn.addEventListener('click', () => {
        const families = ['hypercube', 'simplex', 'orthoplex'];
        const family = families[Math.floor(Math.random() * families.length)];
        const id = this.universe.spawnBeing({
          dimension: 4,
          maxDimension: 4,
          family,
          sides: 5,
          energy: 75
        });
        this.visualizer.selectEntity(id);
        this.visualizer.render();
        this.update();
      });
    }

    const resetBtn = this._qs('#btn-reset-universe');
    if (resetBtn && typeof this.onResetUniverse === 'function') {
      resetBtn.addEventListener('click', () => {
        this.onResetUniverse();
      });
    }
  }

  /**
   * Synchronizes all live DOM telemetry readouts, dimension population bars,
   * selected entity inspector card, and space-time event feed.
   */
  update() {
    if (!this.root) return;
    const snapshot = this.universe.getUniverseSnapshot();

    // 1. Space-Time Clock Readouts
    const properTimeEl = this._qs('#stat-proper-time');
    if (properTimeEl) {
      properTimeEl.textContent = `${snapshot.universalTime.toFixed(2)} s`;
    }

    const tickCountEl = this._qs('#stat-tick-count');
    if (tickCountEl) {
      tickCountEl.textContent = snapshot.tickCount.toLocaleString();
    }

    const timeScaleEl = this._qs('#stat-time-scale');
    if (timeScaleEl) {
      const cleanRate = Number(snapshot.timeScale.toFixed(2));
      timeScaleEl.textContent = snapshot.paused
        ? '0× (PAUSED)'
        : `${cleanRate}× (${cleanRate}s / s)`;
    }

    const maxDimEl = this._qs('#stat-max-dim');
    if (maxDimEl) {
      maxDimEl.textContent = `${snapshot.maxUnlockedDimension}D`;
    }

    const totalBeingsEl = this._qs('#stat-total-beings');
    if (totalBeingsEl) {
      totalBeingsEl.textContent = String(snapshot.beings.length);
    }

    const axiomsEl = this._qs('#stat-axioms');
    if (axiomsEl) {
      axiomsEl.textContent = String(snapshot.totalAxiomsDiscovered);
    }

    const ascensionsEl = this._qs('#stat-ascensions');
    if (ascensionsEl) {
      ascensionsEl.textContent = String(snapshot.ascensionLog.length);
    }

    // Highlight active speed button
    const speedBtns = this._qsa('[data-time-scale]');
    for (const btn of speedBtns) {
      const btnScale = Number(btn.getAttribute('data-time-scale'));
      const isMatch = snapshot.paused
        ? btnScale === 0
        : Math.abs(btnScale - snapshot.timeScale) < 0.01;
      btn.classList.toggle('active', isMatch);
    }

    // 2. Population Breakdown per Dimension (2D .. 6D)
    const totalPop = Math.max(1, snapshot.beings.length);
    for (let d = 2; d <= this.universe.config.maxUniverseDimension; d++) {
      const count = snapshot.populationByDimension[d] ?? 0;
      const countEl = this._qs(`#pop-count-${d}d`);
      const barEl = this._qs(`#pop-bar-${d}d`);
      if (countEl) countEl.textContent = String(count);
      if (barEl) {
        const pct = Math.min(100, Math.round((count / totalPop) * 100));
        barEl.style.width = `${pct}%`;
      }
    }

    // 3. Selected Being Inspector Panel
    this._updateInspectorPanel(snapshot);

    // 4. Live Space-Time Evolutionary Event Feed
    this._updateEventFeed(snapshot.recentEvents);
  }

  /**
   * Renders the Inspector Card for the currently selected N-dimensional being,
   * including interactive Dimensional Phase Jump buttons.
   * Uses structural signature diffing (`REQ-PERF-001`) so phase-jump buttons are not
   * destroyed mid-click when only continuous telemetry (energy/position) changes.
   * @private
   */
  _updateInspectorPanel(snapshot) {
    const container = this._qs('#inspector-content');
    if (!container) return;

    const selectedId = this.visualizer.selectedEntityId;
    const being = snapshot.beings.find((b) => b.entityId === selectedId);

    if (!being) {
      if (this._lastInspectorSignature !== 'empty') {
        this._lastInspectorSignature = 'empty';
        container.innerHTML = `
          <div class="inspector-empty">
            Click any polygon/polytope being in the 3D Space-Time or 2D Slice viewport to inspect its genome, epistemic axioms, and trigger dimensional phase jumps.
          </div>
        `;
      }
      return;
    }

    const posStr = being.position
      .slice(0, being.maxDimension)
      .map((v, i) => `x<sub>${i}</sub>=${v.toFixed(1)}`)
      .join(', ');
    const energyStr = `${being.energy.toFixed(1)} / ${this.universe.config.reproductionThreshold}`;

    const structuralSig = `${being.entityId}:${being.activeDimension}:${being.maxDimension}:${being.phaseState}:${being.family}:${being.schlafliSymbol}:${being.generation}:${being.parentId ?? 0}:${being.symmetryOrder}:${(being.knownAxioms || []).join(',')}`;

    if (this._lastInspectorSignature === structuralSig) {
      const energyEl = this._qs('[data-inspector-energy]');
      if (energyEl) {
        energyEl.textContent = energyStr;
      }
      const coordsEl = this._qs('[data-inspector-coords]');
      if (coordsEl) {
        coordsEl.innerHTML = `<span class="label">N-D Coordinates:</span> (${posStr})`;
      }
      return;
    }

    this._lastInspectorSignature = structuralSig;

    const axiomBadges =
      being.knownAxioms.length > 0
        ? being.knownAxioms
            .map((ax) => `<span class="axiom-badge">${ax}</span>`)
            .join('')
        : '<span class="muted-note">No higher-D axioms discovered yet</span>';

    let jumpButtonsHtml = '';
    if (being.maxDimension > 2) {
      const buttons = [];
      for (let targetDim = 2; targetDim < being.maxDimension; targetDim++) {
        if (being.activeDimension !== targetDim) {
          buttons.push(
            `<button class="hud-btn hud-btn-accent" data-jump-entity="${being.entityId}" data-jump-dim="${targetDim}">
              ⬇ Jump Down to ${targetDim}D Slice
            </button>`
          );
        }
      }
      if (being.activeDimension < being.maxDimension) {
        buttons.push(
          `<button class="hud-btn hud-btn-primary" data-jump-entity="${being.entityId}" data-jump-dim="${being.maxDimension}">
            ⬆ Ascend Back to Native ${being.maxDimension}D
          </button>`
        );
      }
      jumpButtonsHtml = `<div class="inspector-actions">${buttons.join('')}</div>`;
    }

    container.innerHTML = `
      <div class="inspector-header">
        <span class="inspector-id">Entity #${being.entityId}</span>
        <span class="inspector-dim-badge dim-${being.maxDimension}d">
          Active: ${being.activeDimension}D / Max: ${being.maxDimension}D (${being.phaseState})
        </span>
      </div>
      <div class="inspector-grid">
        <div><span class="label">Polytope:</span> <strong>${being.family} ${being.schlafliSymbol}</strong></div>
        <div><span class="label">Generation:</span> <strong>Gen ${being.generation}</strong> ${being.parentId ? `(Parent #${being.parentId})` : '(Primordial)'}</div>
        <div><span class="label">Energy:</span> <strong data-inspector-energy>${energyStr}</strong></div>
        <div><span class="label">Symmetry:</span> <strong>Order ${being.symmetryOrder}</strong></div>
      </div>
      <div class="inspector-coords" data-inspector-coords><span class="label">N-D Coordinates:</span> (${posStr})</div>
      <div class="inspector-axioms-title">Known Geometric Axioms (${being.knownAxioms.length}):</div>
      <div class="inspector-axioms">${axiomBadges}</div>
      ${jumpButtonsHtml}
    `;

    const jumpBtns = Array.from(container.querySelectorAll('[data-jump-entity]'));
    for (const btn of jumpBtns) {
      btn.addEventListener('click', () => {
        const entityId = Number(btn.getAttribute('data-jump-entity'));
        const targetDim = Number(btn.getAttribute('data-jump-dim'));
        this.universe.jumpBeingToDimension(entityId, targetDim);
        this.visualizer.render();
        this.update();
      });
    }
  }

  /**
   * Renders the chronological feed of significant evolutionary and dimensional events.
   * Uses signature diffing (`REQ-PERF-001`) to avoid redundant DOM rebuilds.
   * @private
   */
  _updateEventFeed(recentEvents) {
    const feedEl = this._qs('#event-feed-list');
    if (!feedEl) return;

    const importantTypes = new Set([
      'DIMENSIONAL_ASCENSION',
      'INFO_INJECTED',
      'DIMENSIONAL_PHASE_JUMP',
      'BEING_MUTATED',
      'BEING_REPRODUCED',
      'AXIOM_ACQUIRED',
      'AXIOM_ABSORBED',
      'PEER_EXCHANGE'
    ]);

    const filtered = (recentEvents || [])
      .filter((ev) => importantTypes.has(ev.type ?? ev.eventType))
      .slice(-18)
      .reverse();

    if (filtered.length === 0) {
      if (this._lastEventFeedSignature !== 'empty') {
        this._lastEventFeedSignature = 'empty';
        feedEl.innerHTML = `<li class="event-item muted">Awaiting higher-dimensional pulses & collaborations...</li>`;
      }
      return;
    }

    const newest = filtered[0];
    const feedSig = `${filtered.length}:${newest.type ?? newest.eventType}:${(newest.universalTime ?? 0).toFixed(2)}`;
    if (this._lastEventFeedSignature === feedSig) {
      return;
    }
    this._lastEventFeedSignature = feedSig;

    feedEl.innerHTML = filtered
      .map((ev) => {
        const evType = ev.type ?? ev.eventType;
        const tStr = `T=${(ev.universalTime ?? 0).toFixed(1)}s`;
        const p = ev.payload || {};
        if (evType === 'DIMENSIONAL_ASCENSION') {
          const eid = p.entityId ?? p.ascendedEntityId ?? p.leaderId;
          const fam = p.newPolytopeFamily ?? p.polytopeFamily ?? 'polytope';
          return `<li class="event-item event-ascension">
            <span class="ev-time">${tStr}</span>
            <strong>🚀 ASCENSION ${p.fromDimension}D → ${p.toDimension}D!</strong>
            Entity #${eid} folded into ${fam} ${p.schlafliSymbol ?? ''}
          </li>`;
        }
        if (evType === 'INFO_INJECTED') {
          return `<li class="event-item event-pulse">
            <span class="ev-time">${tStr}</span>
            <strong>⚡ ${p.sourceDimension}D→${p.targetDimension}D Pulse:</strong>
            Injected <code>${p.axiom ?? p.axiomPayload}</code>
          </li>`;
        }
        if (evType === 'DIMENSIONAL_PHASE_JUMP') {
          return `<li class="event-item event-phase">
            <span class="ev-time">${tStr}</span>
            <strong>🌀 Phase Jump (${p.direction ?? 'phase'}):</strong>
            Entity #${p.entityId} ${p.fromDimension}D → ${p.toDimension}D
          </li>`;
        }
        if (evType === 'BEING_MUTATED') {
          const eid = p.entityId ?? p.childId ?? p.offspringId;
          const mutAxiom = p.spontaneousAxiom ?? p.mutatedAxiom;
          const fam = p.polytopeFamily ?? p.newFamily ?? 'polytope';
          const sym = p.symmetryOrder ?? p.newSymmetryOrder ?? '';
          return `<li class="event-item event-mutation">
            <span class="ev-time">${tStr}</span>
            <strong>🧬 Mutation:</strong>
            Offspring #${eid} (from #${p.parentId}) ${mutAxiom ? `synthesized <code>${mutAxiom}</code>` : `evolved to ${fam} order ${sym}`}
          </li>`;
        }
        if (evType === 'AXIOM_ACQUIRED' || evType === 'AXIOM_ABSORBED' || evType === 'PEER_EXCHANGE') {
          const eid = p.entityId ?? p.entityA ?? p.receiverId;
          const ax = p.axiom ?? (Array.isArray(p.axiomsAtoB) ? p.axiomsAtoB[0] : 'axiom');
          return `<li class="event-item event-axiom">
            <span class="ev-time">${tStr}</span>
            <strong>💡 Insight:</strong>
            Entity #${eid} absorbed <code>${ax}</code>
          </li>`;
        }
        const childId = p.childId ?? p.offspringId ?? p.entityId;
        return `<li class="event-item">
          <span class="ev-time">${tStr}</span>
          Entity #${childId} spawned (Gen ${p.generation ?? 1})
        </li>`;
      })
      .join('');
  }
}
