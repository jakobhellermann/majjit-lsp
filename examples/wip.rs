use std::ops::Range;

fn flatten_nested_ranges(ranges: Vec<Range<usize>>) -> Vec<Range<usize>> {
    let mut result = Vec::new();
    let mut stack: Vec<Range<usize>> = Vec::new();

    for range in ranges {
        // While the current range starts after the top of the stack ends, pop from the stack
        while let Some(top) = stack.last() {
            if range.start >= top.end {
                result.push(stack.pop().unwrap());
            } else {
                break;
            }
        }
        // Push the current range onto the stack
        stack.push(range);
    }

    // Add any remaining ranges from the stack to the result
    while let Some(range) = stack.pop() {
        result.push(range);
    }

    result
}

fn main() {
    // Example usage
    let ranges = vec![
        0..10, // A
        2..5,  // B
        7..9,  // C
    ];

    let flattened = flatten_nested_ranges(ranges);

    for range in flattened {
        println!("{:?}", range);
    }
}
