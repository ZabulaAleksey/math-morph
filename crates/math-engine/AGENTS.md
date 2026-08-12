# Math Engine Rules

- Computation and presentation are separate.
- Definition, equality and evaluation are distinct semantics.
- Keep Original AST immutable; presentation transformations produce Display AST/trace.
- Dependency graph/evaluation order must be deterministic.
- Separate numeric precision from display rounding.
- Complex-number conversions require quadrant/zero/tolerance edge tests.
- Do not add Word/MathType-specific logic here.
