use rustfft::{num_complex::Complex, FftPlanner};
use plotters::prelude::*;
use std::error::Error;

pub struct TimeDomain {
    /// Sample period in seconds
    dt: f32,
    /// Raw samples in time order
    samples: Vec<f32>,
}

impl TimeDomain {
    pub fn new(samples: Vec<f32>, sample_rate: i32) -> Self {
        let dt = 1.0 / sample_rate as f32;
        Self { dt, samples }
    }

    /// Returns (min_amplitude, max_amplitude)
    pub fn amplitude_range(&self) -> (f32, f32) {
        let mut iter = self.samples.iter();
        let first = match iter.next() {
            Some(v) => *v,
            None => return (0.0, 0.0),
        };

        let mut min_a = first;
        let mut max_a = first;

        for &v in iter {
            if v < min_a {
                min_a = v;
            }
            if v > max_a {
                max_a = v;
            }
        }

        (min_a, max_a)
    }

    /// Returns (time, value) pairs for plotting
    pub fn coordinates(&self) -> Vec<(f32, f32)> {
        self.samples
            .iter()
            .enumerate()
            .map(|(i, &v)| (i as f32 * self.dt, v))
            .collect()
    }

    pub fn max_time(&self) -> f32 {
        self.samples.len() as f32 * self.dt
    }
}

pub struct FrequencyDomain {
    /// (frequency, magnitude) pairs
    pub spectrum: Vec<(f32, f32)>,
}

impl FrequencyDomain {
    pub fn new(samples: &[f32], sample_rate: i32) -> Self {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(samples.len());

        let mut buffer: Vec<Complex<f32>> = samples
            .iter()
            .map(|&x| Complex { re: x, im: 0.0f32 })
            .collect();

        fft.process(&mut buffer);

        let magnitude: Vec<f32> = buffer.iter().map(|x| x.norm()).collect();

        let half = buffer.len() / 2;
        let n = buffer.len() as f32;
        let frequencies: Vec<f32> = (0..half)
            .map(|x| x as f32 * sample_rate as f32 / n)
            .collect();

        let spectrum: Vec<(f32, f32)> = frequencies
            .into_iter()
            .zip(magnitude)
            .collect();

        Self { spectrum }
    }

    /// Returns (min_freq, max_freq, min_power, max_power)
    pub fn ranges(&self) -> (f32, f32, f32, f32) {
        if self.spectrum.is_empty() {
            return (0.0, 0.0, 0.0, 0.0);
        }

        let mut min_freq = self.spectrum[0].0;
        let mut max_freq = self.spectrum[0].0;
        let mut min_pow = self.spectrum[0].1;
        let mut max_pow = self.spectrum[0].1;

        for &(f, p) in &self.spectrum {
            if f < min_freq {
                min_freq = f;
            }
            if f > max_freq {
                max_freq = f;
            }
            if p < min_pow {
                min_pow = p;
            }
            if p > max_pow {
                max_pow = p;
            }
        }

        (min_freq, max_freq, min_pow, max_pow)
    }
}

pub fn double_side_band(samples: &[f32], sample_rate: i32, carrier_freq: f32) -> Vec<f32> {
    let sample_rate_f = sample_rate as f32;
    let two_pi = 2.0 * std::f32::consts::PI;
    
    samples.iter().enumerate().map(|(i, &samp)| {
        let time = i as f32 / sample_rate_f;
        let carrier = (two_pi * carrier_freq * time).cos();
        samp * carrier
    }).collect()
}

pub fn hilbert_transform(samples: &[f32]) -> Vec<f32> {
    let n = samples.len();
    
    // Create FFT planner
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(n);
    let ifft = planner.plan_fft_inverse(n);
    
    // Convert to complex
    let mut buffer: Vec<Complex<f32>> = samples.iter()
        .map(|&x| Complex::new(x, 0.0))
        .collect();
    
    // Forward FFT
    fft.process(&mut buffer);
    
    // Apply Hilbert transform in frequency domain
    // Multiply positive frequencies by -j, negative by +j
    for i in 0..n {
        if i == 0 || i == n/2 {
            // DC and Nyquist remain unchanged
            buffer[i] = Complex::new(0.0, 0.0);
        } else if i < n/2 {
            // Positive frequencies: multiply by -j (rotate -90°)
            let temp = buffer[i];
            buffer[i] = Complex::new(temp.im, -temp.re) * 2.0;
        } else {
            // Negative frequencies: multiply by +j (rotate +90°)
            let temp = buffer[i];
            buffer[i] = Complex::new(-temp.im, temp.re) * 2.0;
        }
    }
    
    // Inverse FFT
    ifft.process(&mut buffer);
    
    // Extract real part and normalize
    buffer.iter()
        .map(|c| c.re / n as f32)
        .collect()
}

pub fn single_side_band(samples: &[f32], sample_rate: i32, carrier_freq: f32, upper_sideband: bool) -> Vec<f32> {
    let sample_rate_f = sample_rate as f32;
    let two_pi = 2.0 * std::f32::consts::PI;
    
    // Get Hilbert transform (90-degree phase shift)
    let hilbert = hilbert_transform(samples);
    
    samples.iter().enumerate().map(|(i, &samp)| {
        let time = i as f32 / sample_rate_f;
        let carrier_cos = (two_pi * carrier_freq * time).cos();
        let carrier_sin = (two_pi * carrier_freq * time).sin();
        
        if upper_sideband {
            // USB: m(t)cos(ωt) - hilbert(m(t))sin(wt)
            samp * carrier_cos - hilbert[i] * carrier_sin
        } else {
            // LSB: m(t)cos(ωt) + hilbert(m(t))sin(wt)
            samp * carrier_cos + hilbert[i] * carrier_sin
        }
    }).collect()
}

pub fn lowpass_filter(samples: &[f32], cutoff_freq: f32, sample_rate: i32) -> Vec<f32> {
    // Simple moving average low-pass filter
    let sample_rate_f = sample_rate as f32;
    let window_size = (sample_rate_f / (2.0 * cutoff_freq)) as usize;
    let window_size = window_size.max(3); // Minimum window size
    
    let mut result = Vec::with_capacity(samples.len());
    
    for i in 0..samples.len() {
        let start = i.saturating_sub(window_size / 2);
        let end = (i + window_size / 2 + 1).min(samples.len());
        
        let sum: f32 = samples[start..end].iter().sum();
        let count = (end - start) as f32;
        result.push(sum / count);
    }
    
    result
}

pub fn demodulate_ssb(modulated: &[f32], sample_rate: i32, carrier_freq: f32) -> Vec<f32> {
    let sample_rate_f = sample_rate as f32;
    let two_pi = 2.0 * std::f32::consts::PI;
    
    // Multiply by carrier (synchronous detection)
    let demod: Vec<f32> = modulated.iter().enumerate().map(|(i, &samp)| {
        let time = i as f32 / sample_rate_f;
        let carrier = (two_pi * carrier_freq * time).cos();
        samp * carrier * 2.0 // Factor of 2 to compensate for mixing
    }).collect();
    
    // Low-pass filter to remove high-frequency components
    // Cutoff should be just above the highest message frequency
    let cutoff = carrier_freq * 0.1; // Adjust based on your signal bandwidth
    lowpass_filter(&demod, cutoff, sample_rate)
}


pub fn plot_signals(out_file_name: String, samples: &[f32], sample_rate: i32) -> Result<(), Box<dyn Error>> {
    // time-domain representation
    let time_domain = TimeDomain::new(samples.to_vec(), sample_rate);
    let (min_amplitude, max_amplitude) = time_domain.amplitude_range();
    let coordinates_time_domain = time_domain.coordinates();
    let max_time = time_domain.max_time();

    // frequency-domain representation
    let freq_domain = FrequencyDomain::new(samples, sample_rate);
    let (min_freq_spectrum, max_freq_spectrum, min_power_spectrum, max_power_spectrum) =
        freq_domain.ranges();
    let spectrum = freq_domain.spectrum;

    // plotting results
    let root = BitMapBackend::new(&out_file_name, (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    let (upper, lower) = root.split_vertically(512);

    let mut chart_upper = ChartBuilder::on(&upper)
        .caption("Time domain", ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(50)
        .y_label_area_size(50)
        .build_cartesian_2d(-0f32..max_time, min_amplitude..max_amplitude)?;

    chart_upper.configure_mesh().draw()?;

    chart_upper
        .draw_series(LineSeries::new(
                coordinates_time_domain,
                &RED,
        ))?
        .label("time");

    chart_upper
        .configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()?;


    let mut chart_lower = ChartBuilder::on(&lower)
        .caption("Frequency domain", ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(50)
        .y_label_area_size(50)
        .build_cartesian_2d(min_freq_spectrum..max_freq_spectrum, min_power_spectrum..max_power_spectrum)?;

    chart_lower.configure_mesh().draw()?;

    chart_lower
        .draw_series(LineSeries::new(
                spectrum,
                &RED,
        ))?
        .label("Frequency");

    chart_lower
        .configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()?;

    root.present()?;

    Ok(())
}
