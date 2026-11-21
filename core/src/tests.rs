use super::*;

const C: (i32, i32) = (168, 200);

fn rad(a: i32) -> f64 { a as f64 * core::f64::consts::TAU / TURN as f64 }

#[test]
fn closes_after_one_period_for_every_ratio() {
    for a in 1..=FREQ_MAX {
        for b in 1..=FREQ_MAX {
            for ph in 0..PHASE_STEPS {
                let c = Curve::new(Params::new(a, b, ph * PHASE_UNIT), C, (100, 100));
                assert_eq!(c.point(0), c.point(c.len()), "{a}:{b} phase {ph}");
            }
        }
    }
}

#[test]
fn audit_overdraw_no_retrace() {
    // Old code computed lcm turns: 9:9 -> 9216 points. Reduced it is the 1:1 figure.
    let nine = Curve::new(Params::new(9, 9, 0), C, (100, 100));
    let one = Curve::new(Params::new(1, 1, 0), C, (100, 100));
    assert_eq!(nine.len(), one.len());
    assert!((0..=nine.len()).all(|i| nine.point(i) == one.point(i)));
    // 2:4 is the 1:2 figure, same segment count.
    let a = Curve::new(Params::new(2, 4, 0), C, (100, 100));
    let b = Curve::new(Params::new(1, 2, 0), C, (100, 100));
    assert_eq!(a.len(), b.len());
}

#[test]
fn point_count_is_bounded() {
    for a in 1..=FREQ_MAX {
        for b in 1..=FREQ_MAX {
            let n = Curve::new(Params::new(a, b, 0), C, (100, 100)).len();
            assert!((1..=2048).contains(&n), "{a}:{b} -> {n}");
        }
    }
}

#[test]
fn gcd_cases() {
    assert_eq!(gcd(9, 6), 3);
    assert_eq!(gcd(5, 7), 1);
    assert_eq!(gcd(0, 4), 4);
    assert_eq!(gcd(-6, 4), 2);
    assert_eq!(gcd(0, 0), 1);
}

#[test]
fn circle_preset_has_constant_radius() {
    // Audit: the old sine drew the 1:1 / 90 degree preset as a diagonal cross.
    let p = PRESETS[0].params;
    let amp = 120;
    let c = Curve::new(p, C, (amp, amp));
    for i in 0..c.len() {
        let (x, y) = c.point(i);
        let r = (((x - C.0).pow(2) + (y - C.1).pow(2)) as f64).sqrt();
        assert!((r - amp as f64).abs() <= 1.5, "i={i} r={r}");
    }
}

#[test]
fn matches_float_reference_within_a_pixel() {
    for pr in PRESETS {
        let c = Curve::new(pr.params, C, (140, 140));
        let (a, b) = pr.params.reduced();
        for i in (0..=c.len()).step_by(7) {
            let s = rad(i as i32 * (TURN / c.len() as i32));
            let x = C.0 as f64 + 140.0 * (a as f64 * s + rad(pr.params.phase)).sin();
            let y = C.1 as f64 + 140.0 * (b as f64 * s).sin();
            let (px, py) = c.point(i);
            assert!((px as f64 - x).abs() <= 1.5 && (py as f64 - y).abs() <= 1.5, "{} i={i}", pr.name);
        }
    }
}

#[test]
fn figure_eight_known_points() {
    let c = Curve::with_step(PRESETS[1].params, C, (100, 50), 16); // 1:2, phase 0
    assert_eq!(c.len(), 256);
    assert_eq!(c.point(0), C);
    assert_eq!(c.point(64), (C.0 + 100, C.1)); // s = quarter turn: x peak, y zero
    assert_eq!(c.point(32), (C.0 + 71, C.1 + 50)); // s = eighth: y = sin(90 deg)
    assert_eq!(c.point(128), C);
}

#[test]
fn points_stay_inside_amplitude_box() {
    for pr in PRESETS {
        let c = Curve::new(pr.params, C, (140, 90));
        for i in 0..=c.len() {
            let (x, y) = c.point(i);
            assert!((x - C.0).abs() <= 140 && (y - C.1).abs() <= 90, "{}", pr.name);
        }
    }
}

#[test]
fn segment_length_stays_small() {
    for a in 1..=FREQ_MAX {
        for b in 1..=FREQ_MAX {
            let c = Curve::new(Params::new(a, b, 0), C, (140, 140));
            for (p, q) in c.segments(0..c.len()) {
                let d = (p.0 - q.0).abs().max((p.1 - q.1).abs());
                assert!(d <= 16, "{a}:{b} segment {d}px");
            }
        }
    }
}

#[test]
fn tracer_yields_each_segment_once() {
    let c = Curve::new(Params::new(5, 6, 0), C, (100, 100));
    let mut t = Tracer::new(c);
    let mut seen = vec![false; c.len()];
    while !t.done() {
        for i in t.advance(SEGMENTS_PER_TICK) {
            assert!(!seen[i], "segment {i} drawn twice");
            seen[i] = true;
        }
    }
    assert!(seen.iter().all(|&s| s));
    assert!(t.advance(10).is_empty());
    assert_eq!(t.drawn(), c.len());
}

#[test]
fn params_clamp_and_wrap() {
    let mut p = Params::new(0, 12, -PHASE_UNIT);
    assert_eq!((p.a, p.b, p.phase), (1, FREQ_MAX, TURN - PHASE_UNIT));
    p.step_a(-1);
    assert_eq!(p.a, 1);
    p.step_b(1);
    assert_eq!(p.b, FREQ_MAX);
    p.phase = 0;
    p.step_phase(-1);
    assert_eq!(p.phase_degrees(), 337);
    p.step_phase(1);
    assert_eq!(p.phase, 0);
    for _ in 0..PHASE_STEPS {
        p.step_phase(1);
    }
    assert_eq!(p.phase, 0);
}

#[test]
fn presets_are_valid_and_distinct() {
    for (i, a) in PRESETS.iter().enumerate() {
        assert_eq!(a.params, Params::new(a.params.a, a.params.b, a.params.phase));
        for b in &PRESETS[i + 1..] {
            assert!(a.params != b.params || a.name != b.name);
            assert_ne!(a.name, b.name);
        }
    }
    assert_eq!(PRESETS[0].params.phase_degrees(), 90);
}

#[test]
fn thumbnail_is_small_and_closed() {
    let t = thumbnail(Params::new(3, 2, 1024), (30, 30), 20);
    assert_eq!(t.len(), 64);
    assert_eq!(t.point(0), t.point(t.len()));
}
