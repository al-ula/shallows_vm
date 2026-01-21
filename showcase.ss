// ============================================================================
// GLOBALS (provided by VM metadata, NOT declared in source)
// These behave like pre-initialized variables.
// They cannot be assigned to, but can be shadowed by `let`.
// ============================================================================
//
// dt_sec        : uint        // time delta in seconds
// ground_type   : uint        // 0 = air, 2 = dirt, 4 = ice
// translate_x   : fn(float) -> (float, float)
// print         : fn(string) -> void
//
// ============================================================================

//
// Top-level script body
//

// ---------------------------
// Local declarations
// ---------------------------

// Integer literal defaults to `int`, coerced to `float` by explicit type.
let speed: float = 10;

// Uninitialized local (definite-assignment will track this).
let friction: float;

// Shadowing a global with a local (legal).
let ground_type: uint = 2u;

// ---------------------------
// If STATEMENT (no value)
// ---------------------------

// Truthiness: ground_type is numeric; non-zero means true.
if (ground_type) {
    print("On ground\n");
} else {
    print("In air\n");
};

// ---------------------------
// If EXPRESSION with yield
// All branches must yield same type(s): here, `float`
// ---------------------------

let friction = if (ground_type == 2u) {
    yield 0.5f;
} elif (ground_type == 4u) {
    yield 0.7;
} else {
    yield 1;
};

// ---------------------------
// Numeric conversions
// ---------------------------

// Integer literal coerced into float because of explicit type.
let dt_sec_f: float = dt_sec;

// Explicit conversion required for runtime value.
let scaled_dt: float = as_float(dt_sec);

// ---------------------------
// Arithmetic (same-type operands only)
// ---------------------------

let distance: float = speed * friction * scaled_dt;

// ---------------------------
// Destructuring assignment from multi-return function
// ---------------------------

let x, y = translate_x(distance);

// ---------------------------
// Block expression with multiple yields (same type)
// ---------------------------

let quadrant: int = {
    if (x > 0.0 && y > 0.0) {
        yield 1;
    } else {
        yield -1;
    }
};

// ---------------------------
// Reading globals (always initialized)
// ---------------------------

print("dt_sec = ");
print(dt_sec);
print("\n");

// ---------------------------
// Return with multiple values
// ---------------------------

return x, y;
