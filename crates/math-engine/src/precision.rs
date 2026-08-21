//! Backend-neutral precision configuration; it never rewrites an AST.
use std::fmt;
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct PrecisionPolicy {
    computation_digits: u16,
    display_digits: u16,
}
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum PrecisionPolicyError {
    InvalidPrecision,
}
impl PrecisionPolicy {
    pub const MAX_DIGITS: u16 = 1_000;
    pub const fn new(
        computation_digits: u16,
        display_digits: u16,
    ) -> Result<Self, PrecisionPolicyError> {
        if computation_digits == 0
            || display_digits == 0
            || computation_digits > Self::MAX_DIGITS
            || display_digits > Self::MAX_DIGITS
        {
            Err(PrecisionPolicyError::InvalidPrecision)
        } else {
            Ok(Self {
                computation_digits,
                display_digits,
            })
        }
    }
    pub const fn computation_digits(self) -> u16 {
        self.computation_digits
    }
    pub const fn display_digits(self) -> u16 {
        self.display_digits
    }
}
impl fmt::Debug for PrecisionPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PrecisionPolicy")
            .field("computation_digits", &self.computation_digits)
            .field("display_digits", &self.display_digits)
            .finish()
    }
}
impl fmt::Debug for PrecisionPolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("InvalidPrecision")
    }
}
impl fmt::Display for PrecisionPolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("precision policy is invalid")
    }
}
impl std::error::Error for PrecisionPolicyError {}
