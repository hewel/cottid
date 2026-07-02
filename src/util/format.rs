pub fn format_bytes(bytes: u64) -> String {
    format_quantity(bytes, "")
}

pub fn format_speed(bytes_per_second: u64) -> String {
    format_quantity(bytes_per_second, "/s")
}

pub fn format_count(label: &str, count: usize) -> String {
    format!("{label} {count}")
}

pub fn format_progress(completed_bytes: u64, total_bytes: u64) -> String {
    if total_bytes == 0 {
        return format!("{} / unknown", format_bytes(completed_bytes));
    }

    let percentage = completed_bytes as f64 * 100.0 / total_bytes as f64;

    format!(
        "{percentage:.0}% | {} / {}",
        format_bytes(completed_bytes),
        format_bytes(total_bytes)
    )
}

pub fn format_eta(remaining_bytes: u64, bytes_per_second: u64) -> String {
    let duration = format_eta_duration(remaining_bytes, bytes_per_second);

    if duration == "Done" {
        return duration;
    }

    format!("ETA {duration}")
}

pub fn format_eta_duration(remaining_bytes: u64, bytes_per_second: u64) -> String {
    if remaining_bytes == 0 {
        return "Done".to_owned();
    }

    if bytes_per_second == 0 {
        return "unknown".to_owned();
    }

    let seconds = remaining_bytes.div_ceil(bytes_per_second);
    if seconds < 60 {
        return format!("{seconds}s");
    }

    let minutes = seconds.div_ceil(60);
    if minutes < 60 {
        return format!("{minutes}m");
    }

    let hours = minutes.div_ceil(60);
    format!("{hours}h")
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
    use crate::util::format::{
        format_bytes, format_count, format_eta, format_eta_duration, format_progress, format_speed,
    };

    #[test]
    fn formats_bytes_and_speeds_for_stats_display() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1_024), "1.0 KiB");
        assert_eq!(format_bytes(1_536), "1.5 KiB");
        assert_eq!(format_bytes(1_048_576), "1.0 MiB");

        assert_eq!(format_speed(0), "0 B/s");
        assert_eq!(format_speed(1_536), "1.5 KiB/s");
    }

    #[test]
    fn formats_progress_eta_and_counts_consistently() {
        assert_eq!(format_progress(1_024, 2_048), "50% | 1.0 KiB / 2.0 KiB");
        assert_eq!(format_progress(1_024, 0), "1.0 KiB / unknown");
        assert_eq!(format_eta(0, 512), "Done");
        assert_eq!(format_eta(1_024, 0), "ETA unknown");
        assert_eq!(format_eta(1_024, 512), "ETA 2s");
        assert_eq!(format_eta_duration(1_024, 512), "2s");
        assert_eq!(format_eta(61 * 512, 512), "ETA 2m");
        assert_eq!(format_count("Active", 3), "Active 3");
    }
}
