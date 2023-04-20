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
        let mut str = String::new();

        macro_rules! item {
            ($name:literal, $value:expr) => {
                let value = $value;
                if value >= 1.0 {
                    str = format!("{}{}{}", str, $name, $value)
                }
            };
        }

        let seconds = self.time.elapsed().as_secs_f64();

        item!("d", seconds / 86_400.);
        item!("h", seconds / 3_600.);
        item!("m", seconds / 60.);
        item!("s", seconds);
        item!("ms", seconds * 1_000.);
        item!("µs", seconds * 1_000_000.);
        item!("ns", seconds * 1_000_000_000.);

        str
    }
}
