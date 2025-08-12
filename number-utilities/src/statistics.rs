pub struct Statistics {
    pub sum: f64,
    pub mean: f64,
    pub mode: f64,
    pub median: f64,
}

pub fn statistics(items: &[f64]) -> Statistics {
    let mut sum: f64 = 0.;
    let length = items.len();

    let mut items = items.to_owned();
    items.sort_by(f64::total_cmp);
    let median = items[length / 2];

    let mut frequency = std::collections::HashMap::<u64, u64>::new();

    let mut largest: (f64, u64) = (items[0], 1);

    for value in items {
        sum += value;
        let count = frequency
            .entry(value.to_bits())
            .and_modify(|c| *c += 1)
            .or_insert(1);
        if *count > largest.1 {
            largest = (value, *count);
        }
    }

    let mean = sum / (length as f64);
    let mode = largest.0;

    Statistics {
        sum,
        mean,
        mode,
        median,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::OnceLock;

    static ITEMS: &[f64] = &[1.2, 5.6, 3.12, 5.6, 7., 1.2, 7.8, 1.2, 8.9];
    static RESULTS: OnceLock<super::Statistics> = OnceLock::new();

    #[test]
    fn mode() {
        let statistics = RESULTS.get_or_init(|| super::statistics(ITEMS));
        assert_eq!(statistics.mode, 1.2);
    }

    #[test]
    fn median() {
        let statistics = RESULTS.get_or_init(|| super::statistics(ITEMS));
        assert_eq!(statistics.median, 5.6);
    }

    #[test]
    fn mean() {
        let statistics = RESULTS.get_or_init(|| super::statistics(ITEMS));
        assert_eq!(statistics.mean, 4.624444444444444);
    }
}
