/// Performance monitoring utilities
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Simple performance monitor for tracking frame times and event handling
#[derive(Debug)]
pub struct PerformanceMonitor {
    frame_times: VecDeque<Duration>,
    event_times: VecDeque<Duration>,
    max_samples: usize,
    last_frame_start: Option<Instant>,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new(max_samples: usize) -> Self {
        Self {
            frame_times: VecDeque::new(),
            event_times: VecDeque::new(),
            max_samples,
            last_frame_start: None,
        }
    }

    /// Record the start of a frame
    pub fn frame_start(&mut self) {
        self.last_frame_start = Some(Instant::now());
    }

    /// Record the end of a frame
    pub fn frame_end(&mut self) {
        if let Some(start) = self.last_frame_start.take() {
            let duration = start.elapsed();
            self.add_frame_time(duration);
        }
    }

    /// Add a frame time measurement
    pub fn add_frame_time(&mut self, duration: Duration) {
        self.frame_times.push_back(duration);
        if self.frame_times.len() > self.max_samples {
            self.frame_times.pop_front();
        }
    }

    /// Add an event processing time measurement
    pub fn add_event_time(&mut self, duration: Duration) {
        self.event_times.push_back(duration);
        if self.event_times.len() > self.max_samples {
            self.event_times.pop_front();
        }
    }

    /// Get average frame time
    pub fn average_frame_time(&self) -> Option<Duration> {
        if self.frame_times.is_empty() {
            None
        } else {
            let total: Duration = self.frame_times.iter().sum();
            Some(total / self.frame_times.len() as u32)
        }
    }

    /// Get average event processing time
    pub fn average_event_time(&self) -> Option<Duration> {
        if self.event_times.is_empty() {
            None
        } else {
            let total: Duration = self.event_times.iter().sum();
            Some(total / self.event_times.len() as u32)
        }
    }

    /// Get frames per second estimate
    pub fn fps(&self) -> Option<f64> {
        self.average_frame_time().map(|avg| 1.0 / avg.as_secs_f64())
    }

    /// Get performance statistics as a string
    pub fn stats_string(&self) -> String {
        let avg_frame = self
            .average_frame_time()
            .map(|d| format!("{:.2}ms", d.as_millis()))
            .unwrap_or_else(|| "N/A".to_string());

        let avg_event = self
            .average_event_time()
            .map(|d| format!("{:.2}ms", d.as_millis()))
            .unwrap_or_else(|| "N/A".to_string());

        let fps = self
            .fps()
            .map(|f| format!("{:.1}", f))
            .unwrap_or_else(|| "N/A".to_string());

        let capacity = avg_frame.len() + avg_event.len() + fps.len() + 25; // +25 for format strings
        let mut result = String::with_capacity(capacity);
        result.push_str("Frame: ");
        result.push_str(&avg_frame);
        result.push_str(" | Events: ");
        result.push_str(&avg_event);
        result.push_str(" | FPS: ");
        result.push_str(&fps);

        result
    }

    /// Check if performance is degraded
    pub fn is_performance_degraded(&self) -> bool {
        if let Some(fps) = self.fps() {
            fps < 30.0 // Consider sub-30 FPS as degraded
        } else {
            false
        }
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new(60) // Keep 60 samples by default (1 second at 60 FPS)
    }
}
