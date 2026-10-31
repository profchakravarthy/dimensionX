/**
 * @fileoverview Cascaded N-Dimensional to 3D and 2D Screen Projection Pipeline (`ProjectorND`).
 * Projects arbitrary N-dimensional vectors (N >= 2) down to 3D world space and 2D viewport coordinates,
 * supporting cascaded perspective projection for N >= 4, hyperplane slice offsets, and Space-Time Worldline Mode.
 * Implements REQ-VIS-001, REQ-FOUND-002, and REQ-FOUND-003.
 * License: MIT
 */

export class ProjectorND {
  /**
   * @param {object} [options={}]
   * @param {number} [options.cameraYaw=0.65] - Horizontal orbit angle around the vertical Z axis (radians).
   * @param {number} [options.cameraPitch=0.45] - Elevation pitch angle above the 2D Flatland plane (radians).
   * @param {number} [options.cameraDistance=68] - Distance of the 3D pinhole camera from the origin.
   * @param {number} [options.fovScale=520] - Focal length scale factor for 3D -> 2D perspective projection.
   * @param {number} [options.higherDimProjectionDistance=12] - Perspective eye distance for N >= 4 -> 3D projection.
   * @param {boolean} [options.spaceTimeMode=false] - Whether vertical Z axis encodes Proper Time T worldlines.
   * @param {number} [options.timeAxisScale=1.2] - Scale factor mapping proper time delta onto the 3D Z axis.
   */
  constructor({
    cameraYaw = 0.65,
    cameraPitch = 0.45,
    cameraDistance = 68,
    fovScale = 520,
    higherDimProjectionDistance = 12,
    spaceTimeMode = false,
    timeAxisScale = 1.2
  } = {}) {
    this.cameraYaw = Number.isFinite(cameraYaw) ? cameraYaw : 0.65;
    this.cameraPitch = Number.isFinite(cameraPitch) ? cameraPitch : 0.45;
    this.cameraDistance = Number.isFinite(cameraDistance) && cameraDistance > 1 ? cameraDistance : 68;
    this.fovScale = Number.isFinite(fovScale) && fovScale > 0 ? fovScale : 520;
    this.higherDimProjectionDistance =
      Number.isFinite(higherDimProjectionDistance) && higherDimProjectionDistance > 1
        ? higherDimProjectionDistance
        : 12;
    this.spaceTimeMode = Boolean(spaceTimeMode);
    this.timeAxisScale = Number.isFinite(timeAxisScale) ? timeAxisScale : 1.2;

    /** @type {Map<number, number>} */
    this.sliceOffsets = new Map();
  }

  /**
   * Extracts a numeric coordinate array from an Array, TypedArray, or coordinate container.
   *
   * @param {ArrayLike<number> | { data: ArrayLike<number> }} vecND
   * @returns {ArrayLike<number>}
   */
  static extractCoordinates(vecND) {
    if (Array.isArray(vecND) || ArrayBuffer.isView(vecND)) {
      return vecND;
    }
    if (vecND && (Array.isArray(vecND.data) || ArrayBuffer.isView(vecND.data))) {
      return vecND.data;
    }
    throw new TypeError('ProjectorND requires an Array or TypedArray of coordinates');
  }

  /**
   * Rotates the 3D orbit camera by `(deltaYaw, deltaPitch)` in radians.
   *
   * @param {number} deltaYaw
   * @param {number} deltaPitch
   * @returns {this}
   */
  orbit(deltaYaw = 0, deltaPitch = 0) {
    if (Number.isFinite(deltaYaw)) {
      this.cameraYaw += deltaYaw;
    }
    if (Number.isFinite(deltaPitch)) {
      const maxPitch = Math.PI * 0.48;
      this.cameraPitch = Math.max(-maxPitch, Math.min(maxPitch, this.cameraPitch + deltaPitch));
    }
    return this;
  }

  /**
   * Zooms the 3D camera distance by a multiplicative `deltaFactor`.
   *
   * @param {number} deltaFactor - Factor > 0 (e.g., 0.9 zooms in, 1.1 zooms out).
   * @returns {this}
   */
  zoom(deltaFactor = 1.0) {
    if (Number.isFinite(deltaFactor) && deltaFactor > 0) {
      this.cameraDistance = Math.max(8, Math.min(400, this.cameraDistance * deltaFactor));
    }
    return this;
  }

  /**
   * Enables or disables Space-Time Worldline Mode (mapping proper time onto the 3D vertical Z axis).
   *
   * @param {boolean} enabled
   * @returns {this}
   */
  setSpaceTimeMode(enabled) {
    this.spaceTimeMode = Boolean(enabled);
    return this;
  }

  /**
   * Sets a hyperplane slice offset along coordinate axis `axisIndex` (for axisIndex >= 2).
   *
   * @param {number} axisIndex
   * @param {number} value
   * @returns {this}
   */
  setSliceOffset(axisIndex, value) {
    if (!Number.isInteger(axisIndex) || axisIndex < 0) {
      throw new RangeError(`axisIndex must be a non-negative integer, received ${axisIndex}`);
    }
    this.sliceOffsets.set(axisIndex, Number(value) || 0);
    return this;
  }

  /**
   * Returns the configured hyperplane slice offset for `axisIndex`.
   *
   * @param {number} axisIndex
   * @returns {number}
   */
  getSliceOffset(axisIndex) {
    return this.sliceOffsets.get(axisIndex) ?? 0;
  }

  /**
   * Projects an arbitrary N-dimensional coordinate vector (N >= 2) down to 3D world coordinates
   * `{ x, y, z, wDepth, effectiveScale, dimension }`.
   *
   * - Preserves exact 2D and 3D coordinates when N <= 3 (subject to any configured slice offsets).
   * - For N >= 4, applies cascaded perspective projection from dimension N-1 down to 3D so
   *   4D Tesseracts, 5D Penteracts, and 6D Hexeracts display nested polychoron depth structure.
   * - When `spaceTimeMode` is active, maps proper time offset onto the vertical 3D Z axis.
   *
   * @param {VectorND | ArrayLike<number>} vecND
   * @param {object} [options={}]
   * @param {Record<number, number> | Map<number, number>} [options.sliceOffsets]
   * @param {boolean} [options.spaceTimeMode]
   * @param {number} [options.universalTime]
   * @param {number} [options.sampleTime]
   * @param {number} [options.timeAxisScale]
   * @returns {{ x: number, y: number, z: number, wDepth: number, effectiveScale: number, dimension: number }}
   */
  projectNDTo3D(vecND, options = {}) {
    const raw = ProjectorND.extractCoordinates(vecND);
    const dim = raw.length;
    if (dim < 2) {
      throw new RangeError(`ProjectorND requires vector dimension >= 2, received ${dim}`);
    }

    const workLen = Math.max(3, dim);
    const work = new Float64Array(workLen);

    for (let d = 0; d < dim; d++) {
      let offset = this.sliceOffsets.get(d) ?? 0;
      if (options.sliceOffsets) {
        if (options.sliceOffsets instanceof Map) {
          offset = options.sliceOffsets.get(d) ?? offset;
        } else if (typeof options.sliceOffsets === 'object' && options.sliceOffsets[d] !== undefined) {
          offset = Number(options.sliceOffsets[d]) || 0;
        }
      }
      work[d] = raw[d] - (d >= 2 ? offset : 0);
    }

    let wDepth = 0;
    let effectiveScale = 1.0;

    // Cascaded N-D -> (N-1)-D -> ... -> 3D perspective & oblique projection for axes d >= 3
    if (dim >= 4) {
      for (let d = dim - 1; d >= 3; d--) {
        const coordD = work[d];
        const projDist = Math.max(2.5, this.higherDimProjectionDistance + (d - 3) * 3.0);
        const denom = Math.max(0.6, projDist - coordD * 0.35);
        const factor = projDist / denom;

        // Oblique hyper-axis coupling so higher axes (w, v, u...) project with distinct 3D directions
        const hyperAngle = 0.52 * (d - 2);
        const obliqueShift = coordD * 0.24;

        work[0] = work[0] * factor + obliqueShift * Math.cos(hyperAngle);
        work[1] = work[1] * factor + obliqueShift * Math.sin(hyperAngle);
        work[2] = work[2] * factor + coordD * 0.32;

        for (let k = 3; k < d; k++) {
          work[k] *= factor;
        }

        wDepth += coordD / (d - 2);
        effectiveScale *= factor;
      }
    }

    const useSpaceTime = options.spaceTimeMode ?? this.spaceTimeMode;
    const tScale = options.timeAxisScale ?? this.timeAxisScale;

    let z = work[2];
    if (useSpaceTime) {
      let timeDelta = 0;
      if (Number.isFinite(options.sampleTime) && Number.isFinite(options.universalTime)) {
        timeDelta = options.sampleTime - options.universalTime;
      } else if (Number.isFinite(options.sampleTime)) {
        timeDelta = options.sampleTime;
      } else if (Number.isFinite(options.universalTime)) {
        timeDelta = options.universalTime;
      }
      z = work[2] + timeDelta * tScale;
    }

    return {
      x: work[0],
      y: work[1],
      z,
      wDepth,
      effectiveScale,
      dimension: dim
    };
  }

  /**
   * Projects a 3D world point `{ x, y, z, wDepth, effectiveScale }` onto 2D screen coordinates
   * `{ u, v, depth, scale, visible, wDepth }` using the 3D orbit camera.
   *
   * @param {{ x: number, y: number, z: number, wDepth?: number, effectiveScale?: number }} point3D
   * @param {number} viewportWidth
   * @param {number} viewportHeight
   * @returns {{ u: number, v: number, depth: number, scale: number, visible: boolean, wDepth: number }}
   */
  project3DTo2D(point3D, viewportWidth, viewportHeight) {
    const x = Number(point3D?.x) || 0;
    const y = Number(point3D?.y) || 0;
    const z = Number(point3D?.z) || 0;
    const wDepth = Number(point3D?.wDepth) || 0;
    const effectiveScale = Number.isFinite(point3D?.effectiveScale) ? point3D.effectiveScale : 1.0;

    const cosYaw = Math.cos(this.cameraYaw);
    const sinYaw = Math.sin(this.cameraYaw);
    const cosPitch = Math.cos(this.cameraPitch);
    const sinPitch = Math.sin(this.cameraPitch);

    // Orbit rotation around vertical Z axis
    const rx = x * cosYaw - y * sinYaw;
    const ry = x * sinYaw + y * cosYaw;

    // Elevation tilt by cameraPitch
    const camX = rx;
    const camY = z * cosPitch - ry * sinPitch;
    const camZ = ry * cosPitch + z * sinPitch;

    const depth = this.cameraDistance - camZ;
    const visible = depth > 0.5;
    const safeDepth = Math.max(0.5, depth);
    const perspectiveScale = this.fovScale / safeDepth;
    const scale = perspectiveScale * effectiveScale;

    const u = viewportWidth * 0.5 + camX * perspectiveScale;
    const v = viewportHeight * 0.5 - camY * perspectiveScale;

    return {
      u,
      v,
      depth,
      scale,
      visible,
      wDepth
    };
  }

  /**
   * Chains `projectNDTo3D` and `project3DTo2D` to project any N-dimensional vector directly to 2D canvas coordinates.
   *
   * @param {VectorND | ArrayLike<number>} vecND
   * @param {number} viewportWidth
   * @param {number} viewportHeight
   * @param {object} [options={}]
   * @returns {{
   *   u: number,
   *   v: number,
   *   depth: number,
   *   scale: number,
   *   visible: boolean,
   *   wDepth: number,
   *   point3D: { x: number, y: number, z: number, wDepth: number, effectiveScale: number, dimension: number }
   * }}
   */
  projectNDToScreen(vecND, viewportWidth, viewportHeight, options = {}) {
    const point3D = this.projectNDTo3D(vecND, options);
    const screen = this.project3DTo2D(point3D, viewportWidth, viewportHeight);
    return {
      ...screen,
      point3D
    };
  }
}
