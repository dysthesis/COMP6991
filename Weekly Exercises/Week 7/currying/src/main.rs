//TODO: Your curry! macro here:
macro_rules! curry {
    // Match a function signature with at least one argument
    (($first_param_name:ident: $first_param_type:ty) $(, $param_name:ident: $param_type:ty)*) => $body:block) => {
        {
            // Generate the first layer of the curried function
            move |$first_param_name: $first_param_type| {
                // Recursively apply the macro to handle subsequent parameters
                curry!(@inner $($param_name: $param_type),* => $body)
            }
        }
    };

    // Helper macro for handling subsequent parameters
    (@inner $param_name:ident: $param_type:ty $(, $rest_param_name:ident: $rest_param_type:ty)*) => $body:block) => {
        move |$param_name: $param_type| {
            curry!(@inner $($rest_param_name: $rest_param_type),* => $body)
        }
    };

    // Base case: no more parameters, execute the computation block
    (@inner => $body:block) => {
        $body
    };
}
}


////////// DO NOT CHANGE BELOW HERE /////////

fn print_numbers(nums: &Vec<i32>) {
    println!("{nums:#?}");
}

fn get_example_vec() -> Vec<i32> {
    vec![1, 3, 5, 6, 7, 9]
}

fn main() {
    let is_between = curry!((min: i32) => (max: i32) => (item: &i32) => _, {
        min < *item && *item < max
    });

    let curry_filter_between = curry!((min: i32) => (max:i32) => (vec: &Vec<i32>) => _, {
        let filter_between = is_between(min)(max);
        vec.iter().filter_map(|i| if filter_between(i) { Some(*i) } else { None }).collect()
    });

    let between_3_7 = curry_filter_between(3)(7);
    let between_5_10 = curry_filter_between(5)(10);

    let my_vec = get_example_vec();

    let some_numbers: Vec<i32> = between_3_7(&my_vec);
    print_numbers(&some_numbers);

    let more_numbers: Vec<i32> = between_5_10(&my_vec);
    print_numbers(&more_numbers);
}
