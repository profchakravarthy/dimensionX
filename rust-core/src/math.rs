//! Core N-Dimensional Mathematics Engine for dimensionX.
//!
//! Implements:
//! - `SeededRandom`: Deterministic Mulberry32 PRNG + Marsaglia Polar Gaussian sampling.
//! - `VectorND` (`REQ-MATH-001`): Arbitrary N-dimensional Euclidean vector algebra.
//! - `RotorND` (`REQ-MATH-002`): Special Orthogonal group `SO(N)` rotation matrices & bivector plane rotations.
//! - `PolytopeND` (`REQ-MATH-003`): Procedural regular N-polytope generation (Polygons, Simplexes, Hypercubes, Orthoplexes).
//! - `CrossSectionND` (`REQ-MATH-004`): Exact N-dimensional hyperplane slicing and 2D contour extraction.
//!
//! 100% pure Rust standard library (`std`), zero external dependencies, MIT License.

use std::f64::consts::PI;

// ============================================================================
// SeededRandom (Deterministic Mulberry32 PRNG)
// ============================================================================

/// Deterministic Mulberry32 PRNG for reproducible N-dimensional space-time simulations.
#[derive(Clone, Debug)]
pub struct SeededRandom {
    pub state: u32,
    pub initial_seed: u32,
    spare_gaussian: Option<f64>,
}

impl SeededRandom {
    /// Creates a new `SeededRandom` initialized with `seed`.
    pub fn new(seed: u32) -> Self {
        let state = if seed == 0 { 0x6d2b79f5 } else { seed };
        Self {
            state,
            initial_seed: seed,
            spare_gaussian: None,
        }
    }

    /// Resets the PRNG state to `seed`.
    pub fn reset(&mut self, seed: u32) {
        self.initial_seed = seed;
        self.state = if seed == 0 { 0x6d2b79f5 } else { seed };
        self.spare_gaussian = None;
    }

    /// Returns a uniform `f64` in `[0.0, 1.0)` using Mulberry32.
    pub fn next_f64(&mut self) -> f64 {
        self.state = self.state.wrapping_add(0x6d2b79f5);
        let mut t = self.state;
        t = (t ^ (t >> 15)).wrapping_mul(t | 1);
        t ^= t.wrapping_add((t ^ (t >> 7)).wrapping_mul(t | 61));
        let out = t ^ (t >> 14);
        (out as f64) / 4_294_967_296.0
    }

    /// Returns a uniform `f64` in `[min, max)`.
    pub fn range(&mut self, min: f64, max: f64) -> f64 {
        if max <= min {
            return min;
        }
        min + self.next_f64() * (max - min)
    }

    /// Returns a uniform integer in `[min, max]` (both inclusive).
    pub fn int(&mut self, min: i32, max: i32) -> i32 {
        let lo = min.min(max);
        let hi = min.max(max);
        let span = (hi as i64 - lo as i64 + 1) as f64;
        lo + (self.next_f64() * span).floor() as i32
    }

    /// Returns `true` with the given probability in `[0.0, 1.0]`.
    pub fn chance(&mut self, probability: f64) -> bool {
        if probability <= 0.0 {
            return false;
        }
        if probability >= 1.0 {
            return true;
        }
        self.next_f64() < probability
    }

    /// Samples from a normal distribution `N(mean, std_dev^2)` using the Marsaglia polar method.
    pub fn gaussian(&mut self, mean: f64, std_dev: f64) -> f64 {
        if let Some(spare) = self.spare_gaussian.take() {
            return mean + std_dev * spare;
        }
        loop {
            let u = self.next_f64() * 2.0 - 1.0;
            let v = self.next_f64() * 2.0 - 1.0;
            let s = u * u + v * v;
            if s > 0.0 && s < 1.0 {
                let mul = ((-2.0 * s.ln()) / s).sqrt();
                self.spare_gaussian = Some(v * mul);
                return mean + std_dev * (u * mul);
            }
        }
    }
}

// ============================================================================
// REQ-MATH-001: Arbitrary N-Dimensional Vector Algebra (VectorND)
// ============================================================================

/// Arbitrary N-dimensional Euclidean vector in `R^N` for any integer `N >= 1`.
#[derive(Clone, Debug, PartialEq)]
pub struct VectorND {
    pub data: Vec<f64>,
}

impl VectorND {
    /// Creates an N-dimensional zero vector (`dim >= 1`).
    pub fn zero(dim: usize) -> Self {
        let d = dim.max(1);
        Self {
            data: vec![0.0; d],
        }
    }

    /// Creates a `VectorND` from a coordinate slice.
    pub fn from_slice(slice: &[f64]) -> Self {
        assert!(!slice.is_empty(), "VectorND slice must have length >= 1");
        Self {
            data: slice.to_vec(),
        }
    }

    /// Creates a standard basis vector in `R^dim` along `axis` scaled by `length`.
    pub fn basis(dim: usize, axis: usize, length: f64) -> Self {
        assert!(axis < dim, "Basis axis {} out of bounds for dim {}", axis, dim);
        let mut v = Self::zero(dim);
        v.data[axis] = length;
        v
    }

    /// Generates a uniformly distributed random unit vector on the `(active_dim - 1)`-sphere
    /// embedded in `dim` dimensions using spherical Gaussian sampling.
    pub fn random_unit(dim: usize, rng: &mut SeededRandom, active_dim: usize) -> Self {
        let d = dim.max(1);
        let act = active_dim.clamp(1, d);
        let mut v = Self::zero(d);
        let mut norm_sq = 0.0;
        for i in 0..act {
            let g = rng.gaussian(0.0, 1.0);
            v.data[i] = g;
            norm_sq += g * g;
        }
        if norm_sq < 1e-24 {
            v.data[0] = 1.0;
            return v;
        }
        let inv_norm = 1.0 / norm_sq.sqrt();
        for i in 0..act {
            v.data[i] *= inv_norm;
        }
        v
    }

    /// Dimensionality `N` of this vector.
    #[inline]
    pub fn dim(&self) -> usize {
        self.data.len()
    }

    /// Coordinate at axis `i` (returns `0.0` if `i >= dim`, supporting implicit embedding).
    #[inline]
    pub fn get(&self, i: usize) -> f64 {
        self.data.get(i).copied().unwrap_or(0.0)
    }

    /// Sets coordinate at axis `i`.
    #[inline]
    pub fn set(&mut self, i: usize, val: f64) {
        if i < self.data.len() {
            self.data[i] = val;
        }
    }

    /// Vector addition `self + other` in `R^{max(self.dim, other.dim)}`.
    pub fn add(&self, other: &Self) -> Self {
        let max_dim = self.dim().max(other.dim());
        let mut out = Vec::with_capacity(max_dim);
        for i in 0..max_dim {
            out.push(self.get(i) + other.get(i));
        }
        Self { data: out }
    }

    /// Scaled addition `self + other * scale`.
    pub fn add_scaled(&self, other: &Self, scale: f64) -> Self {
        let max_dim = self.dim().max(other.dim());
        let mut out = Vec::with_capacity(max_dim);
        for i in 0..max_dim {
            out.push(self.get(i) + other.get(i) * scale);
        }
        Self { data: out }
    }

    /// Vector subtraction `self - other`.
    pub fn sub(&self, other: &Self) -> Self {
        let max_dim = self.dim().max(other.dim());
        let mut out = Vec::with_capacity(max_dim);
        for i in 0..max_dim {
            out.push(self.get(i) - other.get(i));
        }
        Self { data: out }
    }

    /// Scalar multiplication `self * scalar`.
    pub fn scale(&self, scalar: f64) -> Self {
        Self {
            data: self.data.iter().map(|&x| x * scalar).collect(),
        }
    }

    /// Euclidean inner dot product `self . other`.
    pub fn dot(&self, other: &Self) -> f64 {
        let min_dim = self.dim().min(other.dim());
        let mut sum = 0.0;
        for i in 0..min_dim {
            sum += self.data[i] * other.data[i];
        }
        sum
    }

    /// Squared Euclidean norm `||self||^2`.
    pub fn norm_sq(&self) -> f64 {
        self.data.iter().map(|&x| x * x).sum()
    }

    /// Euclidean norm `||self||`.
    pub fn norm(&self) -> f64 {
        self.norm_sq().sqrt()
    }

    /// Squared Euclidean distance to `other`.
    pub fn distance_sq_to(&self, other: &Self) -> f64 {
        let max_dim = self.dim().max(other.dim());
        let mut sum = 0.0;
        for i in 0..max_dim {
            let d = self.get(i) - other.get(i);
            sum += d * d;
        }
        sum
    }

    /// Euclidean distance to `other`.
    pub fn distance_to(&self, other: &Self) -> f64 {
        self.distance_sq_to(other).sqrt()
    }

    /// Unit-normalized vector (or zero vector if `norm < 1e-15`).
    pub fn normalize(&self) -> Self {
        let n = self.norm();
        if n < 1e-15 {
            Self::zero(self.dim())
        } else {
            self.scale(1.0 / n)
        }
    }

    /// Linear interpolation `self * (1 - t) + other * t`.
    pub fn lerp(&self, other: &Self, t: f64) -> Self {
        let max_dim = self.dim().max(other.dim());
        let one_minus_t = 1.0 - t;
        let mut out = Vec::with_capacity(max_dim);
        for i in 0..max_dim {
            out.push(self.get(i) * one_minus_t + other.get(i) * t);
        }
        Self { data: out }
    }

    /// Orthogonal projection onto `R^target_dim` (truncates higher axes or pads zeros).
    pub fn project_to_dimension(&self, target_dim: usize) -> Self {
        let d = target_dim.max(1);
        let mut out = vec![0.0; d];
        let copy_len = self.dim().min(d);
        out[..copy_len].copy_from_slice(&self.data[..copy_len]);
        Self { data: out }
    }

    /// Embeds this vector in `R^target_dim` (`target_dim >= self.dim()`) by zero-padding.
    pub fn embed_in_dimension(&self, target_dim: usize) -> Self {
        assert!(
            target_dim >= self.dim(),
            "embed_in_dimension requires target_dim ({}) >= current dim ({})",
            target_dim,
            self.dim()
        );
        self.project_to_dimension(target_dim)
    }

    /// Approximate equality within tolerance `eps`.
    pub fn approx_eq(&self, other: &Self, eps: f64) -> bool {
        if self.dim() != other.dim() {
            return false;
        }
        self.data
            .iter()
            .zip(other.data.iter())
            .all(|(&a, &b)| (a - b).abs() <= eps)
    }
}

// ============================================================================
// REQ-MATH-002: N-Dimensional Bivector / Plane Rotations (RotorND)
// ============================================================================

/// N-dimensional Special Orthogonal rotation operator `SO(N)` stored as a row-major `N x N` matrix.
/// Supports rotations across all `N*(N-1)/2` coordinate planes `(i, j)`.
#[derive(Clone, Debug, PartialEq)]
pub struct RotorND {
    pub dim: usize,
    pub elements: Vec<f64>,
}

impl RotorND {
    /// Creates the `N x N` identity rotation matrix.
    pub fn identity(dim: usize) -> Self {
        let n = dim.max(1);
        let mut elements = vec![0.0; n * n];
        for i in 0..n {
            elements[i * n + i] = 1.0;
        }
        Self { dim: n, elements }
    }

    /// Returns all `N*(N-1)/2` canonical coordinate rotation planes `(i, j)` with `0 <= i < j < dim`.
    pub fn get_rotation_planes(dim: usize) -> Vec<(usize, usize)> {
        let mut planes = Vec::new();
        for i in 0..dim {
            for j in (i + 1)..dim {
                planes.push((i, j));
            }
        }
        planes
    }

    /// Creates an elementary Givens plane rotation in the `(axis_i, axis_j)` plane by `angle_rad`.
    pub fn plane_rotation(dim: usize, axis_i: usize, axis_j: usize, angle_rad: f64) -> Self {
        assert!(
            axis_i < dim && axis_j < dim && axis_i != axis_j,
            "Invalid rotation plane ({}, {}) for dimension {}",
            axis_i,
            axis_j,
            dim
        );
        let mut rotor = Self::identity(dim);
        let c = angle_rad.cos();
        let s = angle_rad.sin();
        rotor.elements[axis_i * dim + axis_i] = c;
        rotor.elements[axis_i * dim + axis_j] = -s;
        rotor.elements[axis_j * dim + axis_i] = s;
        rotor.elements[axis_j * dim + axis_j] = c;
        rotor
    }

    /// Matrix multiplication `self * other`.
    pub fn multiply(&self, other: &Self) -> Self {
        assert_eq!(self.dim, other.dim, "RotorND dimension mismatch");
        let n = self.dim;
        let mut out = vec![0.0; n * n];
        for i in 0..n {
            let row_offset = i * n;
            for k in 0..n {
                let aik = self.elements[row_offset + k];
                if aik == 0.0 {
                    continue;
                }
                let k_offset = k * n;
                for j in 0..n {
                    out[row_offset + j] += aik * other.elements[k_offset + j];
                }
            }
        }
        Self {
            dim: n,
            elements: out,
        }
    }

    /// Applies an in-place Givens plane rotation: `M <- R_{axis_i, axis_j}(angle_rad) * M`.
    pub fn rotate_plane_in_place(&mut self, axis_i: usize, axis_j: usize, angle_rad: f64) {
        let n = self.dim;
        if axis_i >= n || axis_j >= n || axis_i == axis_j || angle_rad == 0.0 {
            return;
        }
        let c = angle_rad.cos();
        let s = angle_rad.sin();
        let row_i = axis_i * n;
        let row_j = axis_j * n;
        for col in 0..n {
            let vi = self.elements[row_i + col];
            let vj = self.elements[row_j + col];
            self.elements[row_i + col] = c * vi - s * vj;
            self.elements[row_j + col] = s * vi + c * vj;
        }
    }

    /// Transforms an N-dimensional vector `v` by this rotation matrix.
    pub fn transform_vector(&self, v: &VectorND) -> VectorND {
        let n = self.dim;
        let out_dim = n.max(v.dim());
        let mut out = VectorND::zero(out_dim);
        for i in 0..n {
            let mut sum = 0.0;
            let row_offset = i * n;
            for j in 0..n {
                sum += self.elements[row_offset + j] * v.get(j);
            }
            out.data[i] = sum;
        }
        for i in n..out_dim {
            out.data[i] = v.get(i);
        }
        out
    }

    /// Returns the transpose `R^T` of this rotation matrix.
    pub fn transpose(&self) -> Self {
        let n = self.dim;
        let mut out = vec![0.0; n * n];
        for i in 0..n {
            for j in 0..n {
                out[j * n + i] = self.elements[i * n + j];
            }
        }
        Self {
            dim: n,
            elements: out,
        }
    }

    /// Returns the inverse `R^{-1} = R^T` in `SO(N)`.
    pub fn inverse(&self) -> Self {
        self.transpose()
    }

    /// Computes exact determinant `det(R)` via Gaussian elimination with partial pivoting.
    pub fn determinant(&self) -> f64 {
        let n = self.dim;
        let mut mat = self.elements.clone();
        let mut sign = 1.0_f64;

        for col in 0..n {
            let mut pivot_row = col;
            let mut max_val = mat[col * n + col].abs();
            for row in (col + 1)..n {
                let val = mat[row * n + col].abs();
                if val > max_val {
                    max_val = val;
                    pivot_row = row;
                }
            }

            if max_val < 1e-15 {
                return 0.0;
            }

            if pivot_row != col {
                sign = -sign;
                for k in col..n {
                    mat.swap(col * n + k, pivot_row * n + k);
                }
            }

            let pivot = mat[col * n + col];
            for row in (col + 1)..n {
                let factor = mat[row * n + col] / pivot;
                for k in (col + 1)..n {
                    mat[row * n + k] -= factor * mat[col * n + k];
                }
            }
        }

        let mut det = sign;
        for i in 0..n {
            det *= mat[i * n + i];
        }
        det
    }

    /// Embeds this rotor into `target_dim`, preserving existing block rotations and setting new axes to `1.0`.
    pub fn embed_in_dimension(&self, target_dim: usize) -> Self {
        let td = target_dim.max(1);
        let mut out = Self::identity(td);
        let copy_dim = self.dim.min(td);
        for i in 0..copy_dim {
            for j in 0..copy_dim {
                out.elements[i * td + j] = self.elements[i * self.dim + j];
            }
        }
        out
    }

    /// Checks whether `R * R^T == I` within tolerance `eps`.
    pub fn is_orthogonal(&self, eps: f64) -> bool {
        let n = self.dim;
        for i in 0..n {
            for j in i..n {
                let mut dot = 0.0;
                for k in 0..n {
                    dot += self.elements[i * n + k] * self.elements[j * n + k];
                }
                let expected = if i == j { 1.0 } else { 0.0 };
                if (dot - expected).abs() > eps {
                    return false;
                }
            }
        }
        true
    }

    /// Applies Modified Gram-Schmidt orthonormalization to rows in-place to eliminate numerical drift.
    pub fn reorthogonalize(&mut self) {
        let n = self.dim;
        for i in 0..n {
            let row_i = i * n;
            for j in 0..i {
                let row_j = j * n;
                let mut dot = 0.0;
                for k in 0..n {
                    dot += self.elements[row_i + k] * self.elements[row_j + k];
                }
                for k in 0..n {
                    self.elements[row_i + k] -= dot * self.elements[row_j + k];
                }
            }
            let mut norm_sq = 0.0;
            for k in 0..n {
                let v = self.elements[row_i + k];
                norm_sq += v * v;
            }
            if norm_sq > 1e-24 {
                let inv_norm = 1.0 / norm_sq.sqrt();
                for k in 0..n {
                    self.elements[row_i + k] *= inv_norm;
                }
            } else {
                for k in 0..n {
                    self.elements[row_i + k] = 0.0;
                }
                self.elements[row_i + i] = 1.0;
            }
        }
    }
}

// ============================================================================
// REQ-MATH-003: Procedural N-Dimensional Polytope Generation (PolytopeND)
// ============================================================================

/// Universal regular polytope families across arbitrary dimensions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolytopeFamily {
    Polygon,
    Simplex,
    Hypercube,
    Orthoplex,
}

impl PolytopeFamily {
    pub fn as_str(&self) -> &'static str {
        match self {
            PolytopeFamily::Polygon => "polygon",
            PolytopeFamily::Simplex => "simplex",
            PolytopeFamily::Hypercube => "hypercube",
            PolytopeFamily::Orthoplex => "orthoplex",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "polygon" => PolytopeFamily::Polygon,
            "simplex" => PolytopeFamily::Simplex,
            "hypercube" => PolytopeFamily::Hypercube,
            "orthoplex" => PolytopeFamily::Orthoplex,
            _ => PolytopeFamily::Simplex,
        }
    }
}

/// Complete topological and geometric description of a k-dimensional regular polytope.
#[derive(Clone, Debug)]
pub struct PolytopeData {
    pub family: PolytopeFamily,
    pub dimension: usize,
    pub ambient_dimension: usize,
    pub sides: usize,
    pub schlafli_symbol: String,
    pub vertices: Vec<VectorND>,
    pub edges: Vec<(usize, usize)>,
    pub facet_count: usize,
    pub facets: Vec<Vec<usize>>,
}

pub struct PolytopeND;

impl PolytopeND {
    /// Generates a regular 2D p-gon `{sides}` (`sides >= 3`) centered at origin with circumradius `radius`.
    pub fn generate_polygon_2d(sides: usize, radius: f64, ambient_dimension: usize) -> PolytopeData {
        let p = sides.max(3);
        let target_dim = ambient_dimension.max(2);
        let mut vertices = Vec::with_capacity(p);
        let mut edges = Vec::with_capacity(p);
        let mut facets = Vec::with_capacity(p);

        for i in 0..p {
            let theta = (2.0 * PI * (i as f64)) / (p as f64);
            let mut v = VectorND::zero(target_dim);
            v.data[0] = radius * theta.cos();
            v.data[1] = radius * theta.sin();
            vertices.push(v);
            let next = (i + 1) % p;
            edges.push((i, next));
            facets.push(vec![i, next]);
        }

        PolytopeData {
            family: PolytopeFamily::Polygon,
            dimension: 2,
            ambient_dimension: target_dim,
            sides: p,
            schlafli_symbol: format!("{{{}}}", p),
            vertices,
            edges,
            facet_count: p,
            facets,
        }
    }

    /// Generates a regular k-Simplex (`alpha_k`) in dimension `dim` (`dim + 1` vertices, `dim*(dim+1)/2` edges, `dim + 1` facets)
    /// using the exact Helmert simplex recurrence so all vertices lie on the sphere of radius `radius` and all edges are equal.
    pub fn generate_simplex(dim: usize, radius: f64, ambient_dimension: usize) -> PolytopeData {
        let k = dim.max(1);
        let target_dim = ambient_dimension.max(k);
        let num_vertices = k + 1;
        let mut raw_coords = vec![vec![0.0; target_dim]; num_vertices];

        for j in 0..k {
            let jf = j as f64;
            let denom = ((jf + 1.0) * (jf + 2.0)).sqrt();
            let neg_val = -1.0 / denom;
            let pos_val = (jf + 1.0) / denom;
            for i in 0..=j {
                raw_coords[i][j] = neg_val;
            }
            raw_coords[j + 1][j] = pos_val;
        }

        let scale_factor = radius * (((k + 1) as f64) / (k as f64)).sqrt();
        let vertices: Vec<VectorND> = raw_coords
            .into_iter()
            .map(|mut arr| {
                for j in 0..k {
                    arr[j] *= scale_factor;
                }
                VectorND { data: arr }
            })
            .collect();

        let mut edges = Vec::with_capacity((num_vertices * (num_vertices - 1)) / 2);
        for u in 0..num_vertices {
            for v in (u + 1)..num_vertices {
                edges.push((u, v));
            }
        }

        let mut facets = Vec::with_capacity(num_vertices);
        for omitted in 0..num_vertices {
            let facet: Vec<usize> = (0..num_vertices).filter(|&idx| idx != omitted).collect();
            facets.push(facet);
        }

        let schlafli_symbol = if k == 1 {
            "{}".to_string()
        } else {
            let threes: Vec<&str> = vec!["3"; k - 1];
            format!("{{{}}}", threes.join(","))
        };

        PolytopeData {
            family: PolytopeFamily::Simplex,
            dimension: k,
            ambient_dimension: target_dim,
            sides: num_vertices,
            schlafli_symbol,
            vertices,
            edges,
            facet_count: num_vertices,
            facets,
        }
    }

    /// Generates a regular k-Hypercube (`gamma_k`) in dimension `dim` with `2^dim` vertices, `dim * 2^(dim - 1)` edges,
    /// and `2 * dim` facets, normalized to circumradius `radius`.
    pub fn generate_hypercube(dim: usize, radius: f64, ambient_dimension: usize) -> PolytopeData {
        let k = dim.max(1);
        let target_dim = ambient_dimension.max(k);
        let num_vertices = 1usize << k;
        let coord_mag = radius / (k as f64).sqrt();
        let mut vertices = Vec::with_capacity(num_vertices);

        for mask in 0..num_vertices {
            let mut v = VectorND::zero(target_dim);
            for bit in 0..k {
                v.data[bit] = if (mask & (1 << bit)) != 0 {
                    coord_mag
                } else {
                    -coord_mag
                };
            }
            vertices.push(v);
        }

        let mut edges = Vec::with_capacity(k * (1usize << (k - 1)));
        for mask in 0..num_vertices {
            for bit in 0..k {
                if (mask & (1 << bit)) == 0 {
                    edges.push((mask, mask | (1 << bit)));
                }
            }
        }

        let mut facets = Vec::with_capacity(2 * k);
        for axis in 0..k {
            for sign_bit in 0..=1 {
                let mut facet = Vec::with_capacity(1usize << (k - 1));
                for mask in 0..num_vertices {
                    if ((mask >> axis) & 1) == sign_bit {
                        facet.push(mask);
                    }
                }
                facets.push(facet);
            }
        }

        let schlafli_symbol = if k == 1 {
            "{}".to_string()
        } else {
            let mut parts = vec!["4"];
            for _ in 0..(k.saturating_sub(2)) {
                parts.push("3");
            }
            format!("{{{}}}", parts.join(","))
        };

        PolytopeData {
            family: PolytopeFamily::Hypercube,
            dimension: k,
            ambient_dimension: target_dim,
            sides: 2 * k,
            schlafli_symbol,
            vertices,
            edges,
            facet_count: 2 * k,
            facets,
        }
    }

    /// Generates a regular k-Orthoplex / Cross-Polytope (`beta_k`) in dimension `dim` with `2*dim` vertices,
    /// `2*dim*(dim-1)` edges, and `2^dim` facets, with circumradius `radius`.
    pub fn generate_orthoplex(dim: usize, radius: f64, ambient_dimension: usize) -> PolytopeData {
        let k = dim.max(1);
        let target_dim = ambient_dimension.max(k);
        let num_vertices = 2 * k;
        let mut vertices = Vec::with_capacity(num_vertices);

        for axis in 0..k {
            let mut v_pos = VectorND::zero(target_dim);
            v_pos.data[axis] = radius;
            vertices.push(v_pos);

            let mut v_neg = VectorND::zero(target_dim);
            v_neg.data[axis] = -radius;
            vertices.push(v_neg);
        }

        let mut edges = Vec::new();
        if k == 1 {
            edges.push((0, 1));
        } else {
            for u in 0..num_vertices {
                for v in (u + 1)..num_vertices {
                    if (u >> 1) != (v >> 1) {
                        edges.push((u, v));
                    }
                }
            }
        }

        let num_facets = 1usize << k;
        let mut facets = Vec::with_capacity(num_facets);
        for mask in 0..num_facets {
            let mut facet = Vec::with_capacity(k);
            for axis in 0..k {
                let sign = (mask >> axis) & 1;
                facet.push(axis * 2 + sign);
            }
            facets.push(facet);
        }

        let schlafli_symbol = if k == 1 {
            "{}".to_string()
        } else if k == 2 {
            "{4}".to_string()
        } else {
            let mut parts = vec!["3"; k - 2];
            parts.push("4");
            format!("{{{}}}", parts.join(","))
        };

        PolytopeData {
            family: PolytopeFamily::Orthoplex,
            dimension: k,
            ambient_dimension: target_dim,
            sides: num_facets,
            schlafli_symbol,
            vertices,
            edges,
            facet_count: num_facets,
            facets,
        }
    }

    /// Generates a regular polytope by family, intrinsic dimension, polygon side count, radius, and ambient dimension.
    pub fn generate_by_family(
        family: PolytopeFamily,
        dim: usize,
        sides: usize,
        radius: f64,
        ambient_dimension: usize,
    ) -> PolytopeData {
        match family {
            PolytopeFamily::Polygon => {
                Self::generate_polygon_2d(sides.max(3), radius, ambient_dimension.max(2))
            }
            PolytopeFamily::Simplex => Self::generate_simplex(dim, radius, ambient_dimension),
            PolytopeFamily::Hypercube => Self::generate_hypercube(dim, radius, ambient_dimension),
            PolytopeFamily::Orthoplex => Self::generate_orthoplex(dim, radius, ambient_dimension),
        }
    }

    /// Transforms local polytope vertices by `rotor`, scales by `scale`, and translates by `position`.
    pub fn transform_vertices(
        polytope: &PolytopeData,
        position: &VectorND,
        rotor: &RotorND,
        scale: f64,
    ) -> Vec<VectorND> {
        let first_dim = polytope.vertices.first().map(|v| v.dim()).unwrap_or(2);
        let target_dim = position.dim().max(first_dim);

        polytope
            .vertices
            .iter()
            .map(|local_v| {
                let mut v = local_v.project_to_dimension(target_dim);
                v = rotor.transform_vector(&v);
                if (scale - 1.0).abs() > 1e-15 {
                    v = v.scale(scale);
                }
                v.add(position)
            })
            .collect()
    }
}

// ============================================================================
// REQ-MATH-004: Hyperplane Cross-Section & Slicing Engine (CrossSectionND)
// ============================================================================

/// Result of slicing an N-dimensional polytope with an orthogonal hyperplane `x_{slice_axis} = slice_value`.
#[derive(Clone, Debug)]
pub struct CrossSectionData {
    pub intersects: bool,
    pub points: Vec<VectorND>,
    pub ordered_polygon_2d: Vec<[f64; 2]>,
    pub segments: Vec<[usize; 2]>,
    pub centroid: Option<VectorND>,
    pub bounding_radius: f64,
}

pub struct CrossSectionND;

impl CrossSectionND {
    /// Computes the exact cross-section intersection of `world_vertices` and `edges` with the orthogonal
    /// hyperplane `x_{slice_axis} = slice_value`.
    pub fn slice_polytope(
        world_vertices: &[VectorND],
        edges: &[(usize, usize)],
        slice_axis: usize,
        slice_value: f64,
        eps: f64,
    ) -> CrossSectionData {
        if world_vertices.is_empty() {
            return CrossSectionData {
                intersects: false,
                points: Vec::new(),
                ordered_polygon_2d: Vec::new(),
                segments: Vec::new(),
                centroid: None,
                bounding_radius: 0.0,
            };
        }

        let dim = world_vertices[0].dim();
        if slice_axis >= dim {
            return CrossSectionData {
                intersects: false,
                points: Vec::new(),
                ordered_polygon_2d: Vec::new(),
                segments: Vec::new(),
                centroid: None,
                bounding_radius: 0.0,
            };
        }

        let mut points: Vec<VectorND> = Vec::new();
        let mut add_unique_point = |pt: VectorND| {
            if !points.iter().any(|existing| existing.distance_to(&pt) <= eps) {
                points.push(pt);
            }
        };

        for &(u, v) in edges {
            if u >= world_vertices.len() || v >= world_vertices.len() {
                continue;
            }
            let vu = &world_vertices[u];
            let vv = &world_vertices[v];
            let du = vu.get(slice_axis) - slice_value;
            let dv = vv.get(slice_axis) - slice_value;

            let u_on_plane = du.abs() <= eps;
            let v_on_plane = dv.abs() <= eps;

            if u_on_plane && v_on_plane {
                add_unique_point(vu.clone());
                add_unique_point(vv.clone());
            } else if u_on_plane {
                add_unique_point(vu.clone());
            } else if v_on_plane {
                add_unique_point(vv.clone());
            } else if (du < 0.0 && dv > 0.0) || (du > 0.0 && dv < 0.0) {
                let t = -du / (dv - du);
                let mut intersection = vu.lerp(vv, t);
                intersection.set(slice_axis, slice_value);
                add_unique_point(intersection);
            }
        }

        if points.is_empty() {
            return CrossSectionData {
                intersects: false,
                points: Vec::new(),
                ordered_polygon_2d: Vec::new(),
                segments: Vec::new(),
                centroid: None,
                bounding_radius: 0.0,
            };
        }

        // Compute N-D centroid of intersection points
        let mut centroid = VectorND::zero(dim);
        for p in &points {
            for d in 0..dim {
                centroid.data[d] += p.get(d);
            }
        }
        let inv_count = 1.0 / (points.len() as f64);
        for d in 0..dim {
            centroid.data[d] *= inv_count;
        }

        // Compute bounding radius from centroid
        let mut bounding_radius = 0.0_f64;
        for p in &points {
            let dist = p.distance_to(&centroid);
            if dist > bounding_radius {
                bounding_radius = dist;
            }
        }

        // Deduplicate 2D projected points and sort cyclically around (cx, cy)
        let cx = centroid.get(0);
        let cy = centroid.get(1);
        let mut unique_2d: Vec<[f64; 2]> = Vec::new();
        for pt in &points {
            let px = pt.get(0);
            let py = pt.get(1);
            let is_dup = unique_2d
                .iter()
                .any(|q| (q[0] - px).hypot(q[1] - py) <= eps);
            if !is_dup {
                unique_2d.push([px, py]);
            }
        }

        unique_2d.sort_by(|a, b| {
            let angle_a = (a[1] - cy).atan2(a[0] - cx);
            let angle_b = (b[1] - cy).atan2(b[0] - cx);
            angle_a
                .partial_cmp(&angle_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        points.sort_by(|a, b| {
            let angle_a = (a.get(1) - cy).atan2(a.get(0) - cx);
            let angle_b = (b.get(1) - cy).atan2(b.get(0) - cx);
            angle_a
                .partial_cmp(&angle_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut segments = Vec::new();
        let n2d = unique_2d.len();
        if n2d == 2 {
            segments.push([0, 1]);
        } else if n2d >= 3 {
            for i in 0..n2d {
                segments.push([i, (i + 1) % n2d]);
            }
        }

        CrossSectionData {
            intersects: true,
            points,
            ordered_polygon_2d: unique_2d,
            segments,
            centroid: Some(centroid),
            bounding_radius,
        }
    }
}
