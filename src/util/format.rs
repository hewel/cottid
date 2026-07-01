pub fn format_bytes(bytes: u64) -> String {
    format_quantity(bytes, "")
}

pub fn format_speed(bytes_per_second: u64) -> String {
    format_quantity(bytes_per_second, "/s")
}

fn format_quantity(value: u64, suffix: &str) -> String {
    const UNITS: [&str; 4] = ["B", "KiB", "MiB", "GiB"];

    if value < 1024 {
        return format!("{value} B{suffix}");
    }

    let mut scaled = value as f64;
    let mut unit_index = 0;

    while scaled >= 1024.0 && unit_index < UNITS.len() - 1 {
        scaled /= 1024.0;
        unit_index += 1;
    }

    format!("{scaled:.1} {}{suffix}", UNITS[unit_index])
}

#[cfg(test)]
mod tests {
    use crate::util::format::{format_bytes, format_speed};

    #[test]
    fn formats_bytes_and_speeds_for_stats_display() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1_024), "1.0 KiB");
        assert_eq!(format_bytes(1_536), "1.5 KiB");
        assert_eq!(format_bytes(1_048_576), "1.0 MiB");

        assert_eq!(format_speed(0), "0 B/s");
        assert_eq!(format_speed(1_536), "1.5 KiB/s");
    }
}
