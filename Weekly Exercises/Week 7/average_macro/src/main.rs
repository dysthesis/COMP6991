// YOUR MACRO HERE
#[macro_export]
macro_rules! avg {
    // Match at least one expression followed by `,` and zero or more expressions
    ($first:expr, $($rest:expr),*) => {{
        // Initialize sum with the first expression cast to f64 for floating-point arithmetic
        let mut sum = $first as f64;
        // Count the first expression
        let mut count = 1usize;

        // Iterate over the rest of the expressions
        $(
            sum += $rest as f64; // Add each to the sum, casting to f64
            count += 1;          // Increment the count for each additional expression
        )*

        // Perform floating-point division to get the average
        sum / count as f64
    }};
}

// DO NOT CHANGE
fn main() {
    let a = avg!(1, 2, 3, 4, 5);
    println!("a = {}", a);

    assert_eq!(a, 3);

    let b = avg!(a, 10, 20);
    println!("b = {}", b);
    assert_eq!(b, 11);
}
