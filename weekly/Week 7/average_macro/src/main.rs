// YOUR MACRO HERE
#[macro_export]
macro_rules! avg {
    ($($num:expr),+) => {{
        let mut sum = 0f64; // Use f64 for intermediate calculations to preserve precision.
        let mut count = 0usize; // Count the number of elements.

        // Iterate over the provided numbers, summing them and counting.
        $(
            sum += $num as f64; // Cast each number to f64 for the addition.
            count += 1; // Increment the count for each number.
        )*

        // Calculate the average, round it to the nearest whole number, and then cast to i32.
        (sum / count as f64).round() as i32
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
