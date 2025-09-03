//! Utility functions and common types for RustUX

pub mod error;

pub use error::{Error, Result};

/// Time utilities
pub mod time {
    use std::time::{Duration, Instant};

    /// A simple timer for tracking elapsed time
    #[derive(Debug, Clone)]
    pub struct Timer {
        start: Instant,
        duration: Duration,
    }

    impl Timer {
        /// Create a new timer with the specified duration
        pub fn new(duration: Duration) -> Self {
            Self {
                start: Instant::now(),
                duration,
            }
        }

        /// Check if the timer has elapsed
        pub fn is_elapsed(&self) -> bool {
            self.start.elapsed() >= self.duration
        }

        /// Get the remaining time
        pub fn remaining(&self) -> Duration {
            self.duration.saturating_sub(self.start.elapsed())
        }

        /// Reset the timer
        pub fn reset(&mut self) {
            self.start = Instant::now();
        }

        /// Get the progress as a value between 0.0 and 1.0
        pub fn progress(&self) -> f32 {
            let elapsed = self.start.elapsed().as_secs_f32();
            let total = self.duration.as_secs_f32();
            (elapsed / total).min(1.0)
        }
    }
}

/// File system utilities
pub mod fs {
    use std::path::{Path, PathBuf};
    use crate::util::Result;

    /// Get the data directory for the game
    pub fn get_data_dir() -> Result<PathBuf> {
        // Try to find data directory relative to executable
        let mut path = std::env::current_exe()?;
        path.pop(); // Remove executable name
        path.push("data");
        
        if path.exists() {
            Ok(path)
        } else {
            // Fallback to current directory
            Ok(PathBuf::from("data"))
        }
    }

    /// Check if a file exists and is readable
    pub fn file_exists<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().exists() && path.as_ref().is_file()
    }

    /// Get the extension of a file
    pub fn get_extension<P: AsRef<Path>>(path: P) -> Option<String> {
        path.as_ref()
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_lowercase())
    }
}

/// Math utilities
pub mod math {
    /// Clamp a value between min and max
    pub fn clamp<T: PartialOrd>(value: T, min: T, max: T) -> T {
        if value < min {
            min
        } else if value > max {
            max
        } else {
            value
        }
    }

    /// Linear interpolation between two values
    pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
        a + (b - a) * t
    }

    /// Check if two floating point numbers are approximately equal
    pub fn approx_eq(a: f32, b: f32, epsilon: f32) -> bool {
        (a - b).abs() < epsilon
    }
}