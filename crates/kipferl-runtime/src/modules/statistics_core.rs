pub(super) fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

pub(super) fn sort_like_zig(values: &mut [f64]) {
    for outer in 0..values.len() {
        for inner in 0..values.len().saturating_sub(1 + outer) {
            if values[inner] > values[inner + 1] {
                values.swap(inner, inner + 1);
            }
        }
    }
}

pub(super) fn median(values: &mut [f64]) -> f64 {
    sort_like_zig(values);
    if values.len() % 2 == 1 {
        values[values.len() / 2]
    } else {
        (values[values.len() / 2 - 1] + values[values.len() / 2]) / 2.0
    }
}

pub(super) fn median_low(values: &mut [f64]) -> f64 {
    sort_like_zig(values);
    if values.len() % 2 == 1 {
        values[values.len() / 2]
    } else {
        values[values.len() / 2 - 1]
    }
}

pub(super) fn median_high(values: &mut [f64]) -> f64 {
    sort_like_zig(values);
    values[values.len() / 2]
}

pub(super) fn variance(values: &[f64], sample: bool) -> f64 {
    let average = mean(values);
    let squared_difference_sum = values
        .iter()
        .map(|value| {
            let difference = value - average;
            difference * difference
        })
        .sum::<f64>();
    let denominator = if sample {
        values.len() - 1
    } else {
        values.len()
    };
    squared_difference_sum / denominator as f64
}

#[cfg(test)]
mod tests {
    use super::{mean, median, median_high, median_low, variance};

    fn close(left: f64, right: f64) {
        assert!((left - right).abs() < 1e-12, "{left} != {right}");
    }

    #[test]
    fn numeric_vectors_match_zig() {
        close(mean(&[1.0, 2.0, 3.0, 4.0, 5.0]), 3.0);

        let mut odd = [3.0, 1.0, 2.0];
        close(median(&mut odd), 2.0);
        let mut even = [4.0, 1.0, 3.0, 2.0];
        close(median(&mut even), 2.5);
        let mut low = [4.0, 1.0, 3.0, 2.0];
        close(median_low(&mut low), 2.0);
        let mut high = [4.0, 1.0, 3.0, 2.0];
        close(median_high(&mut high), 3.0);

        close(variance(&[1.0, 2.0, 3.0, 4.0, 5.0], true), 2.5);
        close(variance(&[1.0, 2.0, 3.0, 4.0, 5.0], false), 2.0);
    }
}
