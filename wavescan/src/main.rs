use clap::{Parser, ArgAction};
use std::error::Error;

use crate::app::App;

mod analyzer;
mod app;
mod ui;

// simple program to analyse a wafeform from an audio file

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, action = ArgAction::SetTrue, default_value_t = false)]
    tui: bool,

    #[arg(short, long, default_value_t = String::from("./resources/fubuki.wav"))]
    input: String,

    #[arg(short, long, default_value_t = String::from("./resources/fubuki-noise.png"))]
    output: String,

    #[arg(long, default_value_t = 20000.0)]
    mod_freq: f32,

    #[arg(long, default_value_t = 0.0)]
    demod_freq: f32,

    #[arg(long, default_value_t = 0.0)]
    demod_phase: f32,
}

fn main() -> Result<(), Box<dyn Error>>{
    let args = Args::parse();

    let mut app = App::new(args);
    app.run()?;
    Ok(())
}
