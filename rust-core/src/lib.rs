//! `dimensionx-core` — Native N-Dimensional Space-Time Mathematics, Configuration & Proper-Time Engine.
//!
//! 100% pure Rust standard library (`std` only, zero external runtime dependencies), MIT License.

pub mod config;
pub mod math;
pub mod spacetime;
pub mod universe;
pub mod wasm_api;

#[cfg(test)]
mod tests {
    use crate::config::{BoundaryMode, SimulationConfig};
    use crate::math::{
        CrossSectionND, PolytopeFamily, PolytopeND, RotorND, SeededRandom, VectorND,
    };
    use crate::spacetime::{SpaceTimeClock, TIME_SCALE_LADDER};
    use crate::universe::{DimensionXUniverse, PhaseState};
    use crate::wasm_api::{
        universe_get_snapshot_len, universe_get_snapshot_ptr, universe_get_time_scale,
        universe_init, universe_jump_being_to_dimension, universe_refresh_snapshot,
        universe_seed_default, universe_set_include_worldlines, universe_set_time_scale,
        universe_slow_down_time, universe_spawn_being, universe_speed_up_time, universe_step,
        universe_step_ticks, universe_trigger_injection,
    };
    use std::f64::consts::PI;
    use std::time::Instant;

    #[test]
    fn test_seeded_random_determinism_and_distributions() {
        let mut rng_a = SeededRandom::new(20261031);
        let mut rng_b = SeededRandom::new(20261031);

        for _ in 0..50 {
            let va = rng_a.next_f64();
            let vb = rng_b.next_f64();
            assert!((0.0..1.0).contains(&va));
            assert_eq!(va.to_bits(), vb.to_bits());
        }

        for _ in 0..30 {
            let r = rng_a.range(-10.0, 25.0);
            assert!((-10.0..25.0).contains(&r));
            let k = rng_a.int(3, 9);
            assert!((3..=9).contains(&k));
            let g = rng_a.gaussian(0.0, 1.0);
            assert!(g.is_finite());
        }

        assert!(!rng_a.chance(0.0));
        assert!(rng_a.chance(1.0));
    }

    #[test]
    fn test_vector_nd_algebra_projections_and_embeddings() {
        let mut rng = SeededRandom::new(42);
        for &dim in &[2usize, 3, 4, 5, 7] {
            let z = VectorND::zero(dim);
            assert_eq!(z.dim(), dim);
            assert_eq!(z.norm(), 0.0);

            let e0 = VectorND::basis(dim, 0, 3.0);
            let e1 = VectorND::basis(dim, 1, 4.0);
            assert!((e0.dot(&e1)).abs() < 1e-12);
            let hyp = e0.add(&e1);
            assert!((hyp.norm() - 5.0).abs() < 1e-12);

            let u = VectorND::random_unit(dim, &mut rng, dim);
            let v = VectorND::random_unit(dim, &mut rng, dim);
            assert!((u.norm() - 1.0).abs() < 1e-12);
            assert!((v.norm() - 1.0).abs() < 1e-12);

            // Cauchy-Schwarz inequality: |u . v| <= ||u|| * ||v||
            let dot_uv = u.dot(&v).abs();
            assert!(dot_uv <= u.norm() * v.norm() + 1e-12);

            // Parallelogram law: ||u + v||^2 + ||u - v||^2 == 2*(||u||^2 + ||v||^2)
            let lhs = u.add(&v).norm_sq() + u.sub(&v).norm_sq();
            let rhs = 2.0 * (u.norm_sq() + v.norm_sq());
            assert!((lhs - rhs).abs() < 1e-11);

            // Lerp midpoint
            let mid = u.lerp(&v, 0.5);
            assert!(mid.approx_eq(&u.add(&v).scale(0.5), 1e-12));

            // Embedding into higher dimension and projecting back
            let embedded = u.embed_in_dimension(dim + 3);
            assert_eq!(embedded.dim(), dim + 3);
            assert!((embedded.norm() - u.norm()).abs() < 1e-12);
            let projected = embedded.project_to_dimension(dim);
            assert!(projected.approx_eq(&u, 1e-12));
        }
    }

    #[test]
    fn test_rotor_nd_planes_orthogonality_and_norm_preservation() {
        let mut rng = SeededRandom::new(1337);
        for &dim in &[2usize, 3, 4, 5, 6] {
            let planes = RotorND::get_rotation_planes(dim);
            let expected_planes = (dim * (dim - 1)) / 2;
            assert_eq!(planes.len(), expected_planes);

            let mut composite = RotorND::identity(dim);
            for (idx, &(i, j)) in planes.iter().enumerate() {
                let angle = 0.23 * ((idx + 1) as f64);
                let r_ij = RotorND::plane_rotation(dim, i, j, angle);
                assert!(r_ij.is_orthogonal(1e-12));
                assert!((r_ij.determinant() - 1.0).abs() < 1e-11);
                composite = composite.multiply(&r_ij);
            }

            assert!(composite.is_orthogonal(1e-11));
            assert!((composite.determinant() - 1.0).abs() < 1e-11);

            // Verify norm preservation and inverse across random vectors in R^dim
            let v = VectorND::random_unit(dim, &mut rng, dim).scale(7.5);
            let rotated = composite.transform_vector(&v);
            assert!((rotated.norm() - v.norm()).abs() < 1e-11);

            let inv = composite.inverse();
            let recovered = inv.transform_vector(&rotated);
            assert!(recovered.approx_eq(&v, 1e-11));

            // Test in-place plane rotations and Gram-Schmidt reorthogonalization
            for &(i, j) in &planes {
                composite.rotate_plane_in_place(i, j, 0.05);
            }
            composite.reorthogonalize();
            assert!(composite.is_orthogonal(1e-12));
            assert!((composite.determinant() - 1.0).abs() < 1e-11);
        }
    }

    #[test]
    fn test_polytope_nd_families_across_dimensions() {
        // 1. 2D Polygons {p}
        for &p in &[3usize, 4, 5, 6, 8] {
            let poly = PolytopeND::generate_polygon_2d(p, 2.0, 4);
            assert_eq!(poly.family, PolytopeFamily::Polygon);
            assert_eq!(poly.vertices.len(), p);
            assert_eq!(poly.edges.len(), p);
            assert_eq!(poly.facet_count, p);
            for v in &poly.vertices {
                assert!((v.norm() - 2.0).abs() < 1e-12);
            }
        }

        // 2. Simplex, Hypercube, Orthoplex across k in {2, 3, 4, 5, 6}
        for &k in &[2usize, 3, 4, 5, 6] {
            let radius = 3.0;

            // k-Simplex: k+1 vertices, (k+1)*k/2 edges, k+1 facets
            let simplex = PolytopeND::generate_simplex(k, radius, k + 1);
            assert_eq!(simplex.vertices.len(), k + 1);
            assert_eq!(simplex.edges.len(), (k * (k + 1)) / 2);
            assert_eq!(simplex.facet_count, k + 1);
            let expected_simplex_edge = radius * ((2.0 * ((k + 1) as f64)) / (k as f64)).sqrt();
            for v in &simplex.vertices {
                assert!((v.norm() - radius).abs() < 1e-11);
            }
            for &(u, v) in &simplex.edges {
                let d = simplex.vertices[u].distance_to(&simplex.vertices[v]);
                assert!((d - expected_simplex_edge).abs() < 1e-11);
            }

            // k-Hypercube: 2^k vertices, k * 2^(k-1) edges, 2*k facets
            let hypercube = PolytopeND::generate_hypercube(k, radius, k);
            assert_eq!(hypercube.vertices.len(), 1usize << k);
            assert_eq!(hypercube.edges.len(), k * (1usize << (k - 1)));
            assert_eq!(hypercube.facet_count, 2 * k);
            let expected_cube_edge = (2.0 * radius) / (k as f64).sqrt();
            for v in &hypercube.vertices {
                assert!((v.norm() - radius).abs() < 1e-11);
            }
            for &(u, v) in &hypercube.edges {
                let d = hypercube.vertices[u].distance_to(&hypercube.vertices[v]);
                assert!((d - expected_cube_edge).abs() < 1e-11);
            }

            // k-Orthoplex: 2*k vertices, 2*k*(k-1) edges, 2^k facets
            let orthoplex = PolytopeND::generate_orthoplex(k, radius, k);
            assert_eq!(orthoplex.vertices.len(), 2 * k);
            assert_eq!(orthoplex.edges.len(), 2 * k * (k - 1));
            assert_eq!(orthoplex.facet_count, 1usize << k);
            let expected_ortho_edge = radius * 2.0_f64.sqrt();
            for v in &orthoplex.vertices {
                assert!((v.norm() - radius).abs() < 1e-11);
            }
            for &(u, v) in &orthoplex.edges {
                let d = orthoplex.vertices[u].distance_to(&orthoplex.vertices[v]);
                assert!((d - expected_ortho_edge).abs() < 1e-11);
            }
        }
    }

    #[test]
    fn test_cross_section_nd_slicing_3d_and_4d_polytopes() {
        // 1. Slice 3D Cube at z = 0 -> yields 4-vertex square cross-section
        let cube = PolytopeND::generate_hypercube(3, 3.0_f64.sqrt(), 3);
        let pos3 = VectorND::zero(3);
        let rot3 = RotorND::identity(3);
        let world_cube = PolytopeND::transform_vertices(&cube, &pos3, &rot3, 1.0);
        let slice_cube = CrossSectionND::slice_polytope(&world_cube, &cube.edges, 2, 0.0, 1e-9);
        assert!(slice_cube.intersects);
        assert_eq!(slice_cube.points.len(), 4);
        assert_eq!(slice_cube.ordered_polygon_2d.len(), 4);
        assert_eq!(slice_cube.segments.len(), 4);

        // Slice outside bounds (z = 10.0) -> empty intersection
        let miss = CrossSectionND::slice_polytope(&world_cube, &cube.edges, 2, 10.0, 1e-9);
        assert!(!miss.intersects);
        assert!(miss.points.is_empty());

        // 2. Slice 3D Tetrahedron at z = 0
        let tetra = PolytopeND::generate_simplex(3, 2.0, 3);
        let world_tetra = PolytopeND::transform_vertices(&tetra, &pos3, &rot3, 1.0);
        let slice_tetra = CrossSectionND::slice_polytope(&world_tetra, &tetra.edges, 2, 0.0, 1e-9);
        assert!(slice_tetra.intersects);
        assert!(slice_tetra.ordered_polygon_2d.len() >= 3);

        // 3. Slice 4D Tesseract at w = 0 (axis 3) -> 8 intersection points on edges crossing w = 0
        let tesseract = PolytopeND::generate_hypercube(4, 2.0, 4);
        let pos4 = VectorND::zero(4);
        let mut rot4 = RotorND::identity(4);
        rot4.rotate_plane_in_place(0, 3, PI / 7.0);
        let world_tesseract = PolytopeND::transform_vertices(&tesseract, &pos4, &rot4, 1.0);
        let slice_tesseract =
            CrossSectionND::slice_polytope(&world_tesseract, &tesseract.edges, 3, 0.0, 1e-9);
        assert!(slice_tesseract.intersects);
        assert!(slice_tesseract.points.len() >= 4);
        assert!(slice_tesseract.bounding_radius > 0.0);
    }

    #[test]
    fn test_simulation_config_and_procedural_axioms() {
        let cfg = SimulationConfig::default();
        assert_eq!(cfg.initial_dimension, 2);
        assert_eq!(cfg.max_universe_dimension, 6);
        assert_eq!(cfg.boundary_mode, BoundaryMode::Reflective);

        for target_dim in 3..=10 {
            let axioms = cfg.get_axioms_for_dimension(target_dim);
            assert_eq!(axioms.len(), cfg.axioms_per_dimension);
            assert!(axioms[0].starts_with(&format!("DIM{}_", target_dim)));
        }
    }

    #[test]
    fn test_spacetime_clock_base_slow_motion_hyper_warp_and_ladder() {
        let mut clock = SpaceTimeClock::new(1.0 / 60.0, 1.0, 600);

        // 1. Base rate: 1.0 real second at 1.0x -> 60 ticks = 1.0 simulated second
        let res_1x = clock.advance_real_time(1.0, |_, _, _| {});
        assert_eq!(res_1x.ticks_executed, 60);
        assert!((clock.universal_time - 1.0).abs() < 1e-12);

        // 2. Slow-motion: 0.25x for 1.0 real second -> 15 ticks = 0.25 simulated second
        clock.reset();
        clock.set_time_scale(0.25);
        let res_slow = clock.advance_real_time(1.0, |_, _, _| {});
        assert_eq!(res_slow.ticks_executed, 15);
        assert!((clock.universal_time - 0.25).abs() < 1e-12);

        // Sub-tick fractional slow-motion (0.1x over 1/60s real frame -> 0 ticks, interpolation_alpha = 0.1)
        clock.reset();
        clock.set_time_scale(0.1);
        let res_subtick = clock.advance_real_time(1.0 / 60.0, |_, _, _| {});
        assert_eq!(res_subtick.ticks_executed, 0);
        assert!((res_subtick.interpolation_alpha - 0.1).abs() < 1e-9);

        // 3. Hyper-warp: 10.0x for 1.0 real second -> 600 ticks = 10.0 simulated seconds
        clock.reset();
        clock.set_time_scale(10.0);
        let res_warp = clock.advance_real_time(1.0, |_, _, _| {});
        assert_eq!(res_warp.ticks_executed, 600);
        assert!((clock.universal_time - 10.0).abs() < 1e-11);

        // 4. `-` (slow_down_time) and `+` (speed_up_time) ladder stepping
        clock.set_time_scale(1.0);
        assert!((clock.slow_down_time() - 0.5).abs() < 1e-12);
        assert!((clock.slow_down_time() - 0.25).abs() < 1e-12);
        assert!((clock.slow_down_time() - 0.1).abs() < 1e-12);
        assert!((clock.slow_down_time() - 0.05).abs() < 1e-12);
        assert!((clock.slow_down_time() - 0.0).abs() < 1e-12);
        assert!(clock.paused);

        for &expected in &TIME_SCALE_LADDER[1..] {
            let next = clock.speed_up_time();
            assert!((next - expected).abs() < 1e-12);
        }
    }

    #[test]
    fn test_system_1_kinematics_nd_confinement_and_boundaries() {
        let mut cfg = SimulationConfig::default();
        cfg.world_bounds_radius = 10.0;
        let mut universe = DimensionXUniverse::new(cfg);

        // Spawn a 2D being moving rapidly toward +X boundary with non-zero higher-D components
        let pos = VectorND::from_slice(&[9.5, 0.0, 5.0, 3.0, 0.0, 0.0]);
        let vel = VectorND::from_slice(&[10.0, 0.0, 4.0, 2.0, 0.0, 0.0]);
        universe.spawn_being(
            2,
            Some(PolytopeFamily::Polygon),
            4,
            Some(pos),
            Some(vel),
            Vec::new(),
            1,
            None,
            25.0,
        );

        universe.step_kinematics_nd(0.2);
        let b = &universe.beings[0];

        // Axes >= 2 must be strictly confined to 0.0
        for axis in 2..b.position.dim() {
            assert_eq!(b.position.get(axis), 0.0);
            assert_eq!(b.velocity.get(axis), 0.0);
        }
        // Reflective boundary at R = 10.0 must clamp radius and invert X velocity
        assert!((b.position.get(0) - 10.0).abs() < 1e-9);
        assert!(b.velocity.get(0) < 0.0);
        assert!(b.orientation.is_orthogonal(1e-11));

        // Verify toroidal mode wraps to antipodal point
        universe.config.boundary_mode = BoundaryMode::Toroidal;
        universe.beings[0].position.set(0, 9.5);
        universe.beings[0].velocity.set(0, 10.0);
        universe.step_kinematics_nd(0.2);
        assert!(universe.beings[0].position.get(0) < -9.0);
    }

    #[test]
    fn test_system_2_higher_dim_injection_and_wavefront_absorption() {
        let mut universe = DimensionXUniverse::default();
        let pos_2d = VectorND::from_slice(&[1.0, 1.0, 0.0, 0.0, 0.0, 0.0]);
        let vel_zero = VectorND::zero(6);
        let id_2d = universe.spawn_being(
            2,
            Some(PolytopeFamily::Polygon),
            3,
            Some(pos_2d),
            Some(vel_zero.clone()),
            Vec::new(),
            1,
            None,
            20.0,
        );
        assert!(universe.beings[0].known_axioms.is_empty());

        // Trigger pulse at origin in 2D from 3D
        let pulse_id = universe.inject_pulse(
            None,
            3,
            2,
            VectorND::zero(6),
            Some("DIM3_ORTHOGONAL_NORMAL".to_string()),
        );
        assert!(pulse_id > 0);
        assert_eq!(universe.pulses.len(), 1);

        // Advance pulse wavefront so it crosses distance sqrt(2) ~= 1.414
        universe.step_higher_dim_injection(0.2);
        let b = universe.beings.iter().find(|b| b.id == id_2d).unwrap();
        assert!(b
            .known_axioms
            .contains(&"DIM3_ORTHOGONAL_NORMAL".to_string()));
        assert!(b.energy > 20.0);
    }

    #[test]
    fn test_system_3_peer_exchange_and_links() {
        let mut universe = DimensionXUniverse::default();
        let p1 = VectorND::from_slice(&[0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        let p2 = VectorND::from_slice(&[2.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        let v0 = VectorND::zero(6);

        universe.spawn_being(
            2,
            Some(PolytopeFamily::Polygon),
            3,
            Some(p1),
            Some(v0.clone()),
            vec!["DIM3_ORTHOGONAL_NORMAL".to_string()],
            1,
            None,
            30.0,
        );
        universe.spawn_being(
            2,
            Some(PolytopeFamily::Polygon),
            4,
            Some(p2),
            Some(v0),
            vec!["DIM3_EULER_CLOSURE".to_string()],
            1,
            None,
            30.0,
        );

        universe.step_peer_exchange(1.0 / 60.0);

        assert_eq!(universe.beings[0].known_axioms.len(), 2);
        assert_eq!(universe.beings[1].known_axioms.len(), 2);
        assert!(!universe.beings[0].collaboration_links.is_empty());
        assert!(!universe.beings[1].collaboration_links.is_empty());
        assert!(universe.beings[0].energy > 30.0);
    }

    #[test]
    fn test_system_4_inheritance_and_mutation() {
        let mut cfg = SimulationConfig::default();
        cfg.mutation_probability = 1.0;
        cfg.spontaneous_axiom_mutation_probability = 1.0;
        let mut universe = DimensionXUniverse::new(cfg);

        let parent_id = universe.spawn_being(
            2,
            Some(PolytopeFamily::Polygon),
            4,
            Some(VectorND::zero(6)),
            Some(VectorND::zero(6)),
            vec!["DIM3_ORTHOGONAL_NORMAL".to_string()],
            1,
            None,
            120.0,
        );

        universe.step_inheritance_mutation(1.0 / 60.0);
        assert_eq!(universe.beings.len(), 2);

        let child = &universe.beings[1];
        assert_eq!(child.parent_id, Some(parent_id));
        assert_eq!(child.generation, 2);
        assert!(child
            .inherited_axioms
            .contains(&"DIM3_ORTHOGONAL_NORMAL".to_string()));
        assert!(child.known_axioms.len() >= 2);
        assert!(child.symmetry_order > 4);
        assert!(child.mutation_count >= 1);
    }

    #[test]
    fn test_system_5_collaborative_folding_2d_to_3d_to_4d() {
        let mut universe = DimensionXUniverse::default();
        let dim3_axioms = universe.config.get_axioms_for_dimension(3);
        let p1 = VectorND::from_slice(&[0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        let p2 = VectorND::from_slice(&[2.5, 0.0, 0.0, 0.0, 0.0, 0.0]);
        let v0 = VectorND::zero(6);

        // Two 2D beings holding complementary halves of DIM3 axioms
        let id1 = universe.spawn_being(
            2,
            Some(PolytopeFamily::Polygon),
            4,
            Some(p1),
            Some(v0.clone()),
            dim3_axioms[0..2].to_vec(),
            1,
            None,
            80.0,
        );
        let _id2 = universe.spawn_being(
            2,
            Some(PolytopeFamily::Polygon),
            4,
            Some(p2),
            Some(v0.clone()),
            dim3_axioms[2..4].to_vec(),
            1,
            None,
            40.0,
        );

        universe.step_collaborative_folding(1.0 / 60.0);

        let leader = universe.beings.iter().find(|b| b.id == id1).unwrap();
        assert_eq!(leader.active_dimension, 3);
        assert_eq!(leader.max_dimension, 3);
        assert!(leader.position.get(2) > 0.0);
        assert_eq!(universe.max_unlocked_dimension, 3);
        assert_eq!(universe.ascension_log.len(), 1);

        // Now add a second 3D collaborator holding the remaining DIM4 axioms to fold 3D -> 4D
        let dim4_axioms = universe.config.get_axioms_for_dimension(4);
        let leader_pos = leader.position.clone();
        universe.spawn_being(
            3,
            Some(PolytopeFamily::Hypercube),
            6,
            Some(leader_pos),
            Some(v0),
            dim4_axioms,
            1,
            None,
            50.0,
        );

        universe.step_collaborative_folding(1.0 / 60.0);
        assert_eq!(universe.max_unlocked_dimension, 4);
        assert!(universe.beings.iter().any(|b| b.max_dimension == 4));
    }

    #[test]
    fn test_system_6_dimensional_phasing_and_live_cross_section() {
        let mut universe = DimensionXUniverse::default();
        let id_3d = universe.spawn_being(
            3,
            Some(PolytopeFamily::Hypercube),
            6,
            None,
            None,
            Vec::new(),
            1,
            None,
            50.0,
        );

        // Jump 3D being down into 2D hyperplane
        let jumped = universe.jump_being_to_dimension(id_3d, 2);
        assert!(jumped);

        let b = universe.beings.iter().find(|b| b.id == id_3d).unwrap();
        assert_eq!(b.active_dimension, 2);
        assert_eq!(b.max_dimension, 3);
        assert_eq!(b.phase_state, PhaseState::PhasedDown);
        assert!(b.last_cross_section.is_some());
        assert!(b.last_cross_section.as_ref().unwrap().intersects);

        // Advance past phase_residence_time -> should automatically jump back up to 3D
        let res_time = universe.config.phase_residence_time;
        universe.step_dimensional_phasing(res_time + 0.5);

        let b_after = universe.beings.iter().find(|b| b.id == id_3d).unwrap();
        assert_eq!(b_after.active_dimension, 3);
        assert_eq!(b_after.phase_state, PhaseState::Native);
    }

    #[test]
    fn test_snapshot_json_and_wasm_api_end_to_end() {
        universe_init(6, 1.0, 20261031);
        universe_seed_default(8, 2, 1);

        let spawned_id = universe_spawn_being(3, 2, 6, 55.0);
        assert!(spawned_id > 0);

        let ticks = universe_step(0.1);
        assert!(ticks > 0);
        assert_eq!(universe_step_ticks(5), 5);

        // Test time scale controls via C-ABI
        assert!((universe_set_time_scale(1.0) - 1.0).abs() < 1e-9);
        assert!((universe_slow_down_time() - 0.5).abs() < 1e-9);
        assert!((universe_get_time_scale() - 0.5).abs() < 1e-9);
        assert!((universe_speed_up_time() - 1.0).abs() < 1e-9);

        // Test pulse injection and dimensional phase jump via C-ABI
        let pulse_id = universe_trigger_injection(3, 2);
        assert!(pulse_id > 0);
        assert_eq!(universe_jump_being_to_dimension(spawned_id, 2), 1);

        // Test JSON snapshot serialization buffer
        let len = universe_refresh_snapshot();
        assert!(len > 100);
        assert_eq!(universe_get_snapshot_len(), len);
        let ptr = universe_get_snapshot_ptr();
        assert!(!ptr.is_null());

        let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
        let json_str = std::str::from_utf8(slice).expect("Valid UTF-8 JSON snapshot");
        assert!(json_str.starts_with('{') && json_str.ends_with('}'));
        assert!(json_str.contains("\"universalTime\":"));
        assert!(json_str.contains("\"beings\":["));
        assert!(json_str.contains("\"pulses\":["));
        assert!(json_str.contains("\"populationByDimension\":{"));

        // Verify worldline toggle in snapshot
        universe_set_include_worldlines(0);
        let len_no_wl = universe_refresh_snapshot();
        let slice_no_wl =
            unsafe { std::slice::from_raw_parts(universe_get_snapshot_ptr(), len_no_wl) };
        let json_no_wl = std::str::from_utf8(slice_no_wl).unwrap();
        assert!(json_no_wl.contains("\"worldline\":[]"));
        universe_set_include_worldlines(1);
    }

    #[test]
    fn test_sustained_proper_time_performance_and_bounded_complexity() {
        let mut universe = DimensionXUniverse::default();
        universe.seed_initial_population(14, 4, 2);

        // Spawn 5D and 6D hypercubes and orthoplexes to exercise multi-axis hyperplane slicing
        let id_5d = universe.spawn_being(
            5,
            Some(PolytopeFamily::Hypercube),
            10,
            Some(VectorND::zero(6)),
            None,
            Vec::new(),
            1,
            None,
            60.0,
        );
        let id_6d = universe.spawn_being(
            6,
            Some(PolytopeFamily::Hypercube),
            12,
            Some(VectorND::zero(6)),
            None,
            Vec::new(),
            1,
            None,
            60.0,
        );
        assert!(id_5d > 0 && id_6d > 0);

        // 1. Verify compute_being_cross_section on a 6D hypercube runs in < 2 ms and returns <= 32 vertices
        let b6 = universe.beings.iter().find(|b| b.id == id_6d).unwrap();
        let t_slice_start = Instant::now();
        let cs = DimensionXUniverse::compute_being_cross_section(b6, 2, 0.0);
        let slice_elapsed = t_slice_start.elapsed();
        assert!(
            slice_elapsed.as_secs_f64() * 1000.0 < 2.0,
            "6D cross-section took {:.3} ms (must be < 2.0 ms)",
            slice_elapsed.as_secs_f64() * 1000.0
        );
        assert!(cs.is_some());
        let cs_data = cs.unwrap();
        assert!(cs_data.intersects);
        assert!(cs_data.points.len() <= 32);

        // Also verify fast O(1) bounding-sphere early exit when slice_value is far outside radius
        let cs_far = DimensionXUniverse::compute_being_cross_section(b6, 2, 25.0);
        assert!(cs_far.is_none());

        // 2. Run 600 ticks (10.0 seconds of proper time) + snapshot serialization and verify < 250 ms
        let t_sim_start = Instant::now();
        universe.step_ticks(600);
        let json = universe.to_snapshot_json();
        let sim_elapsed_ms = t_sim_start.elapsed().as_secs_f64() * 1000.0;

        assert!(
            sim_elapsed_ms < 250.0,
            "600 ticks + snapshot took {:.2} ms (must be < 250 ms)",
            sim_elapsed_ms
        );
        assert!(!json.is_empty());
        assert!(!json.contains("\"collaborationLinks\":"));
        assert!(!json.contains("\"activeCrossSection\":"));
        assert!(!json.contains("\"activePulses\":"));

        // 3. Verify every being has collaboration_links.len() <= 8 and ascension_log.len() <= 64
        for b in &universe.beings {
            assert!(
                b.collaboration_links.len() <= 8,
                "Being {} has {} collaboration_links (max 8)",
                b.id,
                b.collaboration_links.len()
            );
        }
        assert!(universe.ascension_log.len() <= 64);
    }
}
