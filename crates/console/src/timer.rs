use std::time::Instant;

pub struct Timer {
    time: Instant,
}

impl Timer {
    pub fn start() -> Self {
        Timer {
            time: Instant::now()
        }
    }
    pub fn elapsed(&self) -> String {
        let seconds = self.time.elapsed().as_secs_f64();

        Self::reminder(seconds / 60., "m")
            .or(Self::reminder(seconds, "s"))
            .or(Self::reminder(seconds * 1_000., "ms"))
            .or(Self::reminder(seconds * 1_000_000., "µs"))
            .or(Self::reminder(seconds * 1_000_000_000., "ns"))
            .unwrap_or(format!("{} s", seconds))
    }

    fn reminder(time: f64, label: &str) -> Option<String> {
        if time >= 1.0 {
            Some(format!("{:.0} {}", time, label))
        } else {
            None
        }
    }
}
