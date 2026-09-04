#[expect(
    clippy::as_conversions,
    clippy::cast_precision_loss,
    reason = "Floating statistics use IEEE-754 rounded sample counts; integer lengths above 2^53 round as in Python."
)]
const fn sample_count(values: &[f64]) -> f64 {
    values.len() as f64
}

pub(super) fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / sample_count(values)
}

pub(super) fn sort_like_zig(values: &mut [f64]) {
    // Adjacent swaps preserve the legacy treatment of unordered NaN values.
    for outer in 0..values.len() {
        let limit = values.len().saturating_sub(outer);
        let Some(pass) = values.get_mut(..limit) else {
            break;
        };
        let mut entries = pass.iter_mut();
        let Some(mut previous) = entries.next() else {
            break;
        };
        for current in entries {
            if *previous > *current {
                std::mem::swap(previous, current);
            }
            previous = current;
        }
    }
}

pub(super) fn median(values: &mut [f64]) -> f64 {
    sort_like_zig(values);
    let middle = values.len() / 2;
    let high = values.get(middle).copied().unwrap_or(f64::NAN);
    if values.len() % 2 == 1 {
        high
    } else {
        values
            .get(middle.saturating_sub(1))
            .copied()
            .unwrap_or(f64::NAN)
            .midpoint(high)
    }
}

pub(super) fn median_low(values: &mut [f64]) -> f64 {
    sort_like_zig(values);
    values
        .get(values.len().saturating_sub(1) / 2)
        .copied()
        .unwrap_or(f64::NAN)
}

pub(super) fn median_high(values: &mut [f64]) -> f64 {
    sort_like_zig(values);
    values.get(values.len() / 2).copied().unwrap_or(f64::NAN)
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
    let denominator = sample_count(values) - f64::from(u8::from(sample));
    squared_difference_sum / denominator
}

#[cfg(test)]
mod tests {
    use super::{mean, median, median_high, median_low, variance};

    fn close(left: f64, right: f64) {
        assert!((left - right).abs() < 1e-12, "{left} != {right}");
    }

    #[test]
    fn median_avoids_overflow_and_empty_inputs_do_not_panic() {
        close(median(&mut [f64::MAX, f64::MAX]), f64::MAX);
        close(median(&mut [-f64::MAX, f64::MAX]), 0.0);
        assert!(median(&mut []).is_nan());
        assert!(median_low(&mut []).is_nan());
        assert!(median_high(&mut []).is_nan());
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
