pub fn wrap_axis(value: f64, size: f64) -> f64 {
    if size <= 0.0 {
        return value;
    }

    value.rem_euclid(size)
}

pub fn shortest_wrapped_delta(from: f64, to: f64, size: f64) -> f64 {
    if size <= 0.0 {
        return to - from;
    }

    let mut delta = to - from;

    if delta > size / 2.0 {
        delta -= size;
    } else if delta < -size / 2.0 {
        delta += size;
    }

    delta
}

#[cfg(test)]
mod tests {
    use super::shortest_wrapped_delta;

    #[test]
    fn shortest_wrapped_delta_wraps_forward_across_edge() {
        assert_eq!(shortest_wrapped_delta(980.0, 20.0, 1000.0), 40.0);
    }
}
