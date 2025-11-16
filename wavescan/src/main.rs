use wavers::{read, Samples};
use rustfft::{num_complex::Complex, FftPlanner};
use plotters::prelude::*;
use clap::Parser;
use std::error::Error;

// simple program to analyse a wafeform from an audio file

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input: String,

    #[arg(short, long, default_value_t = String::from("output.png"))]
    output: String,

    // #[arg(short, long, default_value_t = false)]
    // verbose: bool,
}

struct TimeDomain {
    dataset: Vec<(f32, f32)>    
}

impl TimeDomain {
    fn get_amplitude(&self) -> (f32, f32) {
        let min_amplitude = self.dataset
            .iter()
            .min_by_key(|&(_, value)| value);
        let max_amplitude = self.dataset
            .iter()
            .max_by_key(|&(_, value)| value)
            .unwrap();
        (min_amplitude, max_amplitude)
    }
}

fn main() -> Result<(), Box<dyn Error>>{
    let args = Args::parse();
    
    println!("Waveform analyzer");

    // audio sampling
    let fp = args.input;
    let out_file_name = args.output;
    let (samples, sample_rate): (Samples<f32>, i32) = read::<f32, _>(fp).unwrap();
    let samples: Vec<f32> = samples.to_vec();
 
    let dt:f32 = 1.0 / sample_rate as f32;
    let mut coodinates_time_domain: Vec<(f32, f32)> = Vec::new();
    let mut min_amplitude:f32 = 0.0;
    let mut max_amplitude:f32 = 0.0;
    let max_time:f32 = samples.len() as f32*dt;

    for (i, val) in samples.iter().enumerate(){
        if min_amplitude > samples[i] {
            min_amplitude = samples[i];
        }
        if max_amplitude < samples[i] {
            max_amplitude = samples[i];
        }
        coodinates_time_domain.push((i as f32*dt, *val));
    }

    // fft calculation
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(samples.len());

    let mut buffer: Vec<Complex<f32>> = samples.iter()
        .map(|&x| Complex { re: x, im: 0.0f32 } )
        .collect();

    fft.process(&mut buffer);

    let magnitude: Vec<f32> = buffer.iter()
        .map(|x| x.norm())
        .collect();

    let half = buffer.len()/2;
    let n = buffer.len() as f32;
    let frequencies: Vec<f32> = (0..half)
        .map(|x| x as f32 * sample_rate as f32 / n)
        .collect();

    let spectrum: Vec<(f32, f32)> = frequencies
        .iter()
        .zip(magnitude.iter())
            .map(|(&f, &m)| (f, m))
            .collect();

    let mut min_power_spectrum:f32 = 0.0;
    let mut max_power_spectrum:f32 = 0.0;
    let mut min_freq_spectrum:f32 = 0.0;
    let mut max_freq_spectrum:f32 = 0.0;
    for val in spectrum.iter() {
        if min_freq_spectrum > val.0 {min_freq_spectrum = val.0; }
        if max_freq_spectrum < val.0 {max_freq_spectrum = val.0; }
        if min_power_spectrum > val.1 {min_power_spectrum = val.1; }
        if max_power_spectrum < val.1 {max_power_spectrum = val.1; }
    }
 
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
                coodinates_time_domain,
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
        .label("time");

    chart_lower
        .configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()?;

    root.present()?;

    Ok(())
}
