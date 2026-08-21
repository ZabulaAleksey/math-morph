//! Finite, resource-bounded standalone complex-number engine.

use crate::PrecisionPolicy;
use std::fmt;

#[derive(Clone, Copy, PartialEq)]
pub struct ComplexValue {
    real: f64,
    imaginary: f64,
}
#[derive(Clone, Copy, PartialEq)]
pub struct AlgebraicRepresentation {
    real: f64,
    imaginary: f64,
}
#[derive(Clone, Copy, PartialEq)]
pub struct PolarRepresentation {
    magnitude: f64,
    angle: f64,
}
#[derive(Clone, Copy, PartialEq)]
pub struct Tolerance {
    absolute: f64,
    relative: f64,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct ComplexLimits {
    pub max_trace_steps: usize,
    pub max_formatted_output_bytes: usize,
}
impl ComplexLimits {
    pub const HARD_MAX_TRACE_STEPS: usize = 64;
    pub const HARD_MAX_FORMATTED_OUTPUT_BYTES: usize = 1024 * 1024;
    pub const fn new(max_trace_steps: usize, max_formatted_output_bytes: usize) -> Self {
        Self {
            max_trace_steps,
            max_formatted_output_bytes,
        }
    }
    fn validate(self) -> Result<(), ComplexError> {
        if self.max_trace_steps == 0
            || self.max_trace_steps > Self::HARD_MAX_TRACE_STEPS
            || self.max_formatted_output_bytes == 0
            || self.max_formatted_output_bytes > Self::HARD_MAX_FORMATTED_OUTPUT_BYTES
        {
            Err(ComplexError::InvalidLimits)
        } else {
            Ok(())
        }
    }
}
impl Default for ComplexLimits {
    fn default() -> Self {
        Self::new(8, 16 * 1024)
    }
}
impl fmt::Debug for ComplexLimits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ComplexLimits")
            .field("max_trace_steps", &self.max_trace_steps)
            .field(
                "max_formatted_output_bytes",
                &self.max_formatted_output_bytes,
            )
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComplexOperation {
    Multiplication,
    Division,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComplexOutputMode {
    Algebraic,
    Polar,
    Both,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComplexTraceStep {
    CartesianComputed,
    PolarComputed,
    ScaledDivisionComputed,
    ResultValidated,
}
#[derive(Clone, Eq, PartialEq)]
pub struct ComplexTrace {
    operation: ComplexOperation,
    steps: Vec<ComplexTraceStep>,
}
impl ComplexTrace {
    fn new(
        operation: ComplexOperation,
        steps: &[ComplexTraceStep],
        limits: ComplexLimits,
    ) -> Result<Self, ComplexError> {
        limits.validate()?;
        if steps.len() > limits.max_trace_steps {
            return Err(ComplexError::TraceLimitExceeded {
                limit: limits.max_trace_steps,
            });
        }
        Ok(Self {
            operation,
            steps: steps.to_vec(),
        })
    }
    pub const fn operation(&self) -> ComplexOperation {
        self.operation
    }
    pub fn steps(&self) -> &[ComplexTraceStep] {
        &self.steps
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct ComplexPresentation {
    algebraic: Option<String>,
    polar: Option<String>,
}
impl ComplexPresentation {
    pub fn algebraic(&self) -> Option<&str> {
        self.algebraic.as_deref()
    }
    pub fn polar(&self) -> Option<&str> {
        self.polar.as_deref()
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum ComplexError {
    NonFiniteValue,
    InvalidTolerance,
    DivisionByZero,
    InvalidLimits,
    TraceLimitExceeded { limit: usize },
    FormattedOutputLimitExceeded { limit: usize },
}

impl ComplexValue {
    pub fn new(real: f64, imaginary: f64) -> Result<Self, ComplexError> {
        if real.is_finite() && imaginary.is_finite() {
            Ok(Self { real, imaginary })
        } else {
            Err(ComplexError::NonFiniteValue)
        }
    }
    pub const fn real(self) -> f64 {
        self.real
    }
    pub const fn imaginary(self) -> f64 {
        self.imaginary
    }
    pub const fn algebraic(self) -> AlgebraicRepresentation {
        AlgebraicRepresentation {
            real: self.real,
            imaginary: self.imaginary,
        }
    }
    pub fn to_polar(self) -> Result<PolarRepresentation, ComplexError> {
        PolarRepresentation::new(
            self.real.hypot(self.imaginary),
            self.imaginary.atan2(self.real),
        )
    }
    pub fn multiply(self, other: Self) -> Result<(Self, ComplexTrace), ComplexError> {
        self.multiply_with_limits(other, ComplexLimits::default())
    }
    pub fn multiply_with_limits(
        self,
        other: Self,
        limits: ComplexLimits,
    ) -> Result<(Self, ComplexTrace), ComplexError> {
        limits.validate()?;
        let value = Self::new(
            self.real * other.real - self.imaginary * other.imaginary,
            self.real * other.imaginary + self.imaginary * other.real,
        )?;
        let _polar = value.to_polar()?;
        let trace = ComplexTrace::new(
            ComplexOperation::Multiplication,
            &[
                ComplexTraceStep::CartesianComputed,
                ComplexTraceStep::PolarComputed,
                ComplexTraceStep::ResultValidated,
            ],
            limits,
        )?;
        Ok((value, trace))
    }
    pub fn add_cartesian(self, other: Self) -> Result<Self, ComplexError> {
        Self::new(self.real + other.real, self.imaginary + other.imaginary)
    }
    pub fn subtract_cartesian(self, other: Self) -> Result<Self, ComplexError> {
        Self::new(self.real - other.real, self.imaginary - other.imaginary)
    }
    pub fn divide(self, other: Self) -> Result<Self, ComplexError> {
        self.divide_with_trace(other, ComplexLimits::default())
            .map(|v| v.0)
    }
    pub fn divide_with_trace(
        self,
        other: Self,
        limits: ComplexLimits,
    ) -> Result<(Self, ComplexTrace), ComplexError> {
        limits.validate()?;
        if other.real == 0.0 && other.imaginary == 0.0 {
            return Err(ComplexError::DivisionByZero);
        }
        let scale = other.real.abs().max(other.imaginary.abs());
        let scaled_real = other.real / scale;
        let scaled_imaginary = other.imaginary / scale;
        let denominator = scaled_real * scaled_real + scaled_imaginary * scaled_imaginary;
        let value = Self::new(
            ((self.real / scale) * scaled_real + (self.imaginary / scale) * scaled_imaginary)
                / denominator,
            ((self.imaginary / scale) * scaled_real - (self.real / scale) * scaled_imaginary)
                / denominator,
        )?;
        let trace = ComplexTrace::new(
            ComplexOperation::Division,
            &[
                ComplexTraceStep::ScaledDivisionComputed,
                ComplexTraceStep::ResultValidated,
            ],
            limits,
        )?;
        Ok((value, trace))
    }
    pub fn output_mode(
        self,
        mode: ComplexOutputMode,
    ) -> Result<(Option<AlgebraicRepresentation>, Option<PolarRepresentation>), ComplexError> {
        match mode {
            ComplexOutputMode::Algebraic => Ok((Some(self.algebraic()), None)),
            ComplexOutputMode::Polar => Ok((None, Some(self.to_polar()?))),
            ComplexOutputMode::Both => Ok((Some(self.algebraic()), Some(self.to_polar()?))),
        }
    }
    pub fn present(
        self,
        mode: ComplexOutputMode,
        precision: PrecisionPolicy,
        limits: ComplexLimits,
    ) -> Result<ComplexPresentation, ComplexError> {
        limits.validate()?;
        let digits = usize::from(precision.display_digits());
        let algebraic = matches!(mode, ComplexOutputMode::Algebraic | ComplexOutputMode::Both)
            .then(|| format_algebraic(self, digits));
        let polar = if matches!(mode, ComplexOutputMode::Polar | ComplexOutputMode::Both) {
            Some(format_polar(self.to_polar()?, digits))
        } else {
            None
        };
        let bytes = algebraic
            .as_ref()
            .map_or(0, String::len)
            .checked_add(polar.as_ref().map_or(0, String::len))
            .ok_or(ComplexError::NonFiniteValue)?;
        if bytes > limits.max_formatted_output_bytes {
            return Err(ComplexError::FormattedOutputLimitExceeded {
                limit: limits.max_formatted_output_bytes,
            });
        }
        Ok(ComplexPresentation { algebraic, polar })
    }
}

impl AlgebraicRepresentation {
    pub const fn real(self) -> f64 {
        self.real
    }
    pub const fn imaginary(self) -> f64 {
        self.imaginary
    }
}
impl PolarRepresentation {
    pub fn new(magnitude: f64, angle: f64) -> Result<Self, ComplexError> {
        if !magnitude.is_finite() || !angle.is_finite() || magnitude < 0.0 {
            return Err(ComplexError::NonFiniteValue);
        }
        if magnitude == 0.0 {
            return Ok(Self {
                magnitude: 0.0,
                angle: 0.0,
            });
        }
        let angle =
            (angle + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU) - std::f64::consts::PI;
        Ok(Self { magnitude, angle })
    }
    pub const fn magnitude(self) -> f64 {
        self.magnitude
    }
    pub const fn angle(self) -> f64 {
        self.angle
    }
    pub fn to_complex(self) -> Result<ComplexValue, ComplexError> {
        ComplexValue::new(
            self.magnitude * self.angle.cos(),
            self.magnitude * self.angle.sin(),
        )
    }
}
impl Tolerance {
    pub fn new(absolute: f64, relative: f64) -> Result<Self, ComplexError> {
        if absolute.is_finite() && relative.is_finite() && absolute >= 0.0 && relative >= 0.0 {
            Ok(Self { absolute, relative })
        } else {
            Err(ComplexError::InvalidTolerance)
        }
    }
    pub const fn absolute(self) -> f64 {
        self.absolute
    }
    pub const fn relative(self) -> f64 {
        self.relative
    }
    pub fn approx_eq(self, left: ComplexValue, right: ComplexValue) -> bool {
        scalar_approx(left.real, right.real, self)
            && scalar_approx(left.imaginary, right.imaginary, self)
    }
}
fn scalar_approx(left: f64, right: f64, tolerance: Tolerance) -> bool {
    let difference = (left - right).abs();
    if difference <= tolerance.absolute {
        return true;
    }
    let scale = left.abs().max(right.abs());
    scale > 0.0 && (difference - tolerance.absolute) / scale <= tolerance.relative
}
fn formatted(value: f64, digits: usize) -> String {
    let value = if value == 0.0 { 0.0 } else { value };
    let mut output = format!("{value:.digits$}");
    if output.starts_with('-') && output[1..].chars().all(|c| c == '0' || c == '.') {
        output.remove(0);
    }
    output
}
fn format_algebraic(value: ComplexValue, digits: usize) -> String {
    let real = formatted(value.real, digits);
    let signed_imaginary = formatted(value.imaginary, digits);
    let (sign, imaginary) = if let Some(magnitude) = signed_imaginary.strip_prefix('-') {
        ('-', magnitude)
    } else {
        ('+', signed_imaginary.as_str())
    };
    format!("{real}{sign}{imaginary}i")
}
fn format_polar(value: PolarRepresentation, digits: usize) -> String {
    format!(
        "{}∠{}",
        formatted(value.magnitude, digits),
        formatted(value.angle, digits)
    )
}

macro_rules! redacted_debug { ($($t:ty),+ $(,)?) => {$(
    impl fmt::Debug for $t { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(stringify!($t)) } }
)+}; }
redacted_debug!(
    ComplexValue,
    AlgebraicRepresentation,
    PolarRepresentation,
    Tolerance,
    ComplexPresentation
);
impl fmt::Debug for ComplexTrace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ComplexTrace")
            .field("operation", &self.operation)
            .field("step_count", &self.steps.len())
            .finish()
    }
}
impl fmt::Debug for ComplexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NonFiniteValue => "NonFiniteValue",
            Self::InvalidTolerance => "InvalidTolerance",
            Self::DivisionByZero => "DivisionByZero",
            Self::InvalidLimits => "InvalidLimits",
            Self::TraceLimitExceeded { .. } => "TraceLimitExceeded",
            Self::FormattedOutputLimitExceeded { .. } => "FormattedOutputLimitExceeded",
        })
    }
}
impl fmt::Display for ComplexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "complex operation failed: {self:?}")
    }
}
impl std::error::Error for ComplexError {}
