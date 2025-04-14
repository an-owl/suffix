use suffix;

/// This indicates 2 things
///
/// 1. That lower case prefixes will not be parsed
/// 2. That unrecognized suffixes will not be modified
fn main() {
    let _ = suffix::metric!(1ki);
}