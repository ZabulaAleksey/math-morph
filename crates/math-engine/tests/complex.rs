use math_engine::{
    ComplexError, ComplexLimits, ComplexOperation, ComplexOutputMode, ComplexTraceStep,
    ComplexValue, PolarRepresentation, PrecisionPolicy, Tolerance,
};
#[test]
fn values_are_finite_and_algebraic_components_are_preserved() {
    let value = ComplexValue::new(-0.0, 2.5).expect("finite");
    assert_eq!(value.algebraic().real().to_bits(), (-0.0f64).to_bits());
    assert_eq!(value.algebraic().imaginary(), 2.5);
    assert_eq!(
        ComplexValue::new(f64::NAN, 0.0),
        Err(ComplexError::NonFiniteValue)
    );
    assert!(Tolerance::new(0.0, 1e-6).is_ok());
}
#[test]
fn polar_normalizes_origin_and_quadrants() {
    let origin = ComplexValue::new(0.0, 0.0).unwrap().to_polar().unwrap();
    assert_eq!(origin.magnitude(), 0.0);
    assert_eq!(origin.angle(), 0.0);
    let q2 = ComplexValue::new(-1.0, 1.0).unwrap().to_polar().unwrap();
    assert!(q2.angle() > 0.0);
    let q3 = ComplexValue::new(-1.0, -1.0).unwrap().to_polar().unwrap();
    assert!(q3.angle() < 0.0);
    assert!(PolarRepresentation::new(-1.0, 0.0).is_err());
}

#[test]
fn polar_to_algebraic_preserves_axes_without_rounding() {
    let value = PolarRepresentation::new(2.0, 0.0)
        .unwrap()
        .to_complex()
        .unwrap();
    assert_eq!(value.real(), 2.0);
    assert_eq!(value.imaginary(), 0.0);
    let origin = PolarRepresentation::new(0.0, 1.0)
        .unwrap()
        .to_complex()
        .unwrap();
    assert_eq!(origin.real(), 0.0);
    assert_eq!(origin.imaginary(), 0.0);
}

#[test]
fn multiplication_uses_cartesian_formula_and_redacted_trace() {
    let (value, trace) = ComplexValue::new(1.0, 2.0)
        .unwrap()
        .multiply(ComplexValue::new(3.0, 4.0).unwrap())
        .unwrap();
    assert_eq!((value.real(), value.imaginary()), (-5.0, 10.0));
    assert_eq!(
        trace.steps(),
        &[
            ComplexTraceStep::CartesianComputed,
            ComplexTraceStep::PolarComputed,
            ComplexTraceStep::ResultValidated,
        ]
    );
    assert!(!format!("{trace:?}").contains("10"));
}

#[test]
fn cartesian_arithmetic_and_zero_division_are_typed() {
    let left = ComplexValue::new(4.0, 2.0).unwrap();
    let right = ComplexValue::new(1.0, -3.0).unwrap();
    assert_eq!(
        (
            left.add_cartesian(right).unwrap().real(),
            left.add_cartesian(right).unwrap().imaginary()
        ),
        (5.0, -1.0)
    );
    assert_eq!(
        (
            left.subtract_cartesian(right).unwrap().real(),
            left.subtract_cartesian(right).unwrap().imaginary()
        ),
        (3.0, 5.0)
    );
    assert_eq!(
        left.divide(ComplexValue::new(0.0, 0.0).unwrap()),
        Err(ComplexError::DivisionByZero)
    );
}

#[test]
fn output_modes_change_only_the_presentation_view() {
    let value = ComplexValue::new(3.0, 4.0).unwrap();
    assert!(
        value
            .output_mode(ComplexOutputMode::Algebraic)
            .unwrap()
            .0
            .is_some()
    );
    assert!(
        value
            .output_mode(ComplexOutputMode::Polar)
            .unwrap()
            .1
            .is_some()
    );
    assert_eq!((value.real(), value.imaginary()), (3.0, 4.0));
}

#[test]
fn edge_cases_reject_nonfinite_arithmetic_and_invalid_tolerance() {
    assert_eq!(
        Tolerance::new(-1.0, 0.0),
        Err(ComplexError::InvalidTolerance)
    );
    assert_eq!(
        Tolerance::new(f64::INFINITY, 0.0),
        Err(ComplexError::InvalidTolerance)
    );
    let huge = ComplexValue::new(f64::MAX, 0.0).unwrap();
    assert_eq!(huge.multiply(huge), Err(ComplexError::NonFiniteValue));
    assert_eq!(huge.divide(ComplexValue::new(1.0, 0.0).unwrap()), Ok(huge));
}

#[test]
fn scaled_division_handles_extreme_finite_denominators_and_has_bounded_trace() {
    let numerator = ComplexValue::new(f64::MAX, 0.0).unwrap();
    let denominator = ComplexValue::new(f64::MAX, f64::MAX).unwrap();
    let (value, trace) = numerator
        .divide_with_trace(denominator, ComplexLimits::default())
        .expect("scaled division");
    assert_eq!((value.real(), value.imaginary()), (0.5, -0.5));
    assert_eq!(trace.operation(), ComplexOperation::Division);
    assert_eq!(trace.steps().len(), 2);
    assert_eq!(
        numerator.divide_with_trace(denominator, ComplexLimits::new(1, 100)),
        Err(ComplexError::TraceLimitExceeded { limit: 1 })
    );
}

#[test]
fn tolerance_boundaries_and_polar_round_trip_are_explicit() {
    let tolerance = Tolerance::new(0.01, 0.001).unwrap();
    assert!(tolerance.approx_eq(
        ComplexValue::new(1.0, 2.0).unwrap(),
        ComplexValue::new(1.01, 2.0).unwrap()
    ));
    assert!(!tolerance.approx_eq(
        ComplexValue::new(1.0, 2.0).unwrap(),
        ComplexValue::new(1.02, 2.0).unwrap()
    ));
    let original = ComplexValue::new(-3.0, 4.0).unwrap();
    let round_trip = original.to_polar().unwrap().to_complex().unwrap();
    assert!(
        Tolerance::new(1e-12, 1e-12)
            .unwrap()
            .approx_eq(original, round_trip)
    );
}

#[test]
fn formatted_presentation_rounds_only_output_normalizes_zero_and_honors_bytes() {
    let value = ComplexValue::new(-0.0, -0.0001).unwrap();
    let precision = PrecisionPolicy::new(15, 2).unwrap();
    let presentation = value
        .present(ComplexOutputMode::Both, precision, ComplexLimits::default())
        .unwrap();
    assert_eq!(presentation.algebraic(), Some("0.00+0.00i"));
    assert!(presentation.polar().is_some());
    assert_eq!(value.real().to_bits(), (-0.0f64).to_bits());
    assert_eq!(
        value.present(ComplexOutputMode::Both, precision, ComplexLimits::new(8, 1)),
        Err(ComplexError::FormattedOutputLimitExceeded { limit: 1 })
    );
}
