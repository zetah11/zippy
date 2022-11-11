pub fn min_range_size(lo: i64, hi: i64) -> usize {
    let range = if 0 < lo {
        hi as usize
    } else if 0 > hi {
        -(lo as i128) as usize
    } else {
        (hi - lo) as usize
    };

    ((range as f64).log2() / 8.0).ceil() as usize
}
