use core::f32;
use std::usize;
use std::{error::Error};
use std::path::Path;
use ratatui_image::protocol::StatefulProtocol;

use crate::analyzer;
use crate::ui;
use crate::Args;

#[derive(Debug, PartialEq)]
pub enum ConfigState {
    Files,
    Mod,
    Demod,
    Plot,
}

pub enum CurrentlyEditingConfig {
    InputFile,
    OutputFile,
    ModFreq,
    DemodFreqError,
    DemodPhaseError,
    PlotMSG,
    PlotDSB,
    PlotUSB,
    PlotLSB,
    PlotDemod,
}

fn get_checkbox_index(checkbox: &CurrentlyEditingConfig) -> Option<usize> {
    match checkbox {
        CurrentlyEditingConfig::PlotMSG => Some(0),
        CurrentlyEditingConfig::PlotDSB => Some(1),
        CurrentlyEditingConfig::PlotUSB => Some(2),
        CurrentlyEditingConfig::PlotLSB => Some(3),
        CurrentlyEditingConfig::PlotDemod => Some(4),
        _ => None,
    }
}

fn get_checkbox_by_index(indx: usize) -> Option<CurrentlyEditingConfig> {
    match indx {
        0 => Some(CurrentlyEditingConfig::PlotMSG),
        1 => Some(CurrentlyEditingConfig::PlotDSB),
        2 => Some(CurrentlyEditingConfig::PlotUSB),
        3 => Some(CurrentlyEditingConfig::PlotLSB),
        4 => Some(CurrentlyEditingConfig::PlotDemod),
        _ => None
    }
}

pub struct App {
    tui_mode: bool,
    pub state: ConfigState,

    pub input_file: String,
    pub output_file: String,
    pub current_editing_config: Option<CurrentlyEditingConfig>,

    pub mod_freq: f32,
    pub demod_freq_error: f32,
    pub demod_phase_error: f32,
    pub checkboxes: Vec<bool>,

    pub has_error: bool,
    pub error_buffer: String,
    pub image: Option<StatefulProtocol>,
}

impl App {
    pub fn new(args: Args) -> Self {
        Self {
            tui_mode: args.tui,
            state: ConfigState::Files,
            input_file: args.input,
            output_file: args.output,
            current_editing_config: None,
            mod_freq: args.mod_freq,
            demod_freq_error: args.demod_freq,
            demod_phase_error: args.demod_phase,
            checkboxes: vec![
                false,
                false,
                false,
                false,
                false,
            ],
            has_error: false,
            error_buffer: String::new(),
            image: None,
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        if self.tui_mode {
            ui::tui_mode(self)?;
        }
        else {
            cmd_mode(self)?;
        }
        Ok(())
    }

    pub fn change_state_to(&mut self, state: ConfigState) {
        self.state = state;
        self.current_editing_config = None;
        self.validate_inputs();
    }

    pub fn next_editing_config(&mut self) {
        self.current_editing_config = match self.state {
            ConfigState::Files => {
                match &self.current_editing_config {
                    None => Some(CurrentlyEditingConfig::InputFile),
                    Some(CurrentlyEditingConfig::InputFile) => Some(CurrentlyEditingConfig::OutputFile),
                    Some(CurrentlyEditingConfig::OutputFile) => None,
                    _ => None,
                }
            }
            ConfigState::Mod => {
                match &self.current_editing_config {
                    None => Some(CurrentlyEditingConfig::ModFreq),
                    Some(CurrentlyEditingConfig::ModFreq) => None,
                    _ => None,
                }
            }
            ConfigState::Demod => {
                match &self.current_editing_config {
                    None => Some(CurrentlyEditingConfig::DemodFreqError),
                    Some(CurrentlyEditingConfig::DemodFreqError) => Some(CurrentlyEditingConfig::DemodPhaseError),
                    Some(CurrentlyEditingConfig::DemodPhaseError) => None,
                    _ => None,
                }
            }
            ConfigState::Plot => {
                match &self.current_editing_config {
                    None => Some(CurrentlyEditingConfig::PlotMSG),
                    Some(CurrentlyEditingConfig::PlotMSG) => Some(CurrentlyEditingConfig::PlotDSB),
                    Some(CurrentlyEditingConfig::PlotDSB) => Some(CurrentlyEditingConfig::PlotUSB),
                    Some(CurrentlyEditingConfig::PlotUSB) => Some(CurrentlyEditingConfig::PlotLSB),
                    Some(CurrentlyEditingConfig::PlotLSB) => Some(CurrentlyEditingConfig::PlotDemod),
                    Some(CurrentlyEditingConfig::PlotDemod) => None,
                    _ => None,
                }
            }
        };
        self.validate_inputs();
    }

    pub fn prev_editing_config(&mut self) {
        self.current_editing_config = match self.state {
            ConfigState::Files => {
                match &self.current_editing_config {
                    None => Some(CurrentlyEditingConfig::OutputFile),
                    Some(CurrentlyEditingConfig::OutputFile) => Some(CurrentlyEditingConfig::InputFile),
                    Some(CurrentlyEditingConfig::InputFile) => None,
                    _ => None,
                }
            }
            ConfigState::Mod => {
                match &self.current_editing_config {
                    None => Some(CurrentlyEditingConfig::ModFreq),
                    Some(CurrentlyEditingConfig::ModFreq) => None,
                    _ => None,
                }
            }
            ConfigState::Demod => {
                match &self.current_editing_config {
                    None => Some(CurrentlyEditingConfig::DemodPhaseError),
                    Some(CurrentlyEditingConfig::DemodPhaseError) => Some(CurrentlyEditingConfig::DemodFreqError),
                    Some(CurrentlyEditingConfig::DemodFreqError) => None,
                    _ => None,
                }
            }
            ConfigState::Plot => {
                match &self.current_editing_config {
                    None => Some(CurrentlyEditingConfig::PlotDemod),
                    Some(CurrentlyEditingConfig::PlotDemod) => Some(CurrentlyEditingConfig::PlotLSB),
                    Some(CurrentlyEditingConfig::PlotLSB) => Some(CurrentlyEditingConfig::PlotUSB),
                    Some(CurrentlyEditingConfig::PlotUSB) => Some(CurrentlyEditingConfig::PlotDSB),
                    Some(CurrentlyEditingConfig::PlotDSB) => Some(CurrentlyEditingConfig::PlotMSG),
                    Some(CurrentlyEditingConfig::PlotMSG) => None,
                    _ => None,
                }
            }
        };
        self.validate_inputs();
    }

    pub fn clear_error(&mut self) {
        self.has_error = false;
        self.error_buffer.clear();
    }

    pub fn set_error<S: Into<String>>(&mut self, msg: S) {
        self.has_error = true;
        self.error_buffer = msg.into();
    }

    /// Basic validation for current inputs (files, output, frequency, etc.)
    pub fn validate_inputs(&mut self) {
        self.clear_error();

        // validate input wave file
        let input_trimmed = self.input_file.trim();
        if input_trimmed.is_empty() {
            self.set_error("Input wave file path is empty.");
            return;
        }

        if !Path::new(input_trimmed).exists() {
            self.set_error(format!("Input wave file does not exist: {}", input_trimmed));
            return;
        }

        // validate output file (png)
        if !self.output_file.is_empty() && !self.output_file.ends_with(".png") {
            self.set_error(format!("Output file should have .png extension: {}", self.output_file));
            return;
        }

        // validate modulator frequency
        if self.mod_freq <= 0.0 {
            self.set_error("Carrier frequency must be greater than 0 Hz.");
        }
    }

    pub fn toggle_checkbox(&mut self) {
        if let Some(curr) = &self.current_editing_config {
            match curr {
                CurrentlyEditingConfig::PlotMSG | CurrentlyEditingConfig::PlotDSB | 
                    CurrentlyEditingConfig::PlotUSB | CurrentlyEditingConfig::PlotLSB |
                    CurrentlyEditingConfig::PlotDemod => {
                    let mut empty_checkbox: Vec<bool> = self.checkboxes.iter().map(|_| false).collect();
                    let indx = get_checkbox_index(curr).unwrap();
                    empty_checkbox[indx] = true;
                    self.checkboxes = empty_checkbox;
                }
                _ => {}
            }
        }
    }

    pub fn execute_sim(&self) -> Result<(), Box<dyn Error>> {
        let (samples, sample_rate): (wavers::Samples<f32>, i32) = 
            wavers::read::<f32, _>(self.input_file.clone()).unwrap();
        let samples: Vec<f32> = samples.to_vec();
        
        // Plot original signal
        analyzer::plot_signals(self.output_file.clone(), &samples, sample_rate)?;
        
        // Double Side Band
        let dsb = analyzer::double_side_band(&samples, sample_rate, self.mod_freq);
        let dsb_out_name = self.output_file.replace(".png", "_dsb.png");
        analyzer::plot_signals(dsb_out_name, &dsb, sample_rate)?;
        
        // Single Sideband (Upper)
        let ssb_upper = analyzer::single_side_band(&samples, sample_rate, self.mod_freq, true);
        let ssb_upper_out_name = self.output_file.replace(".png", "_ssb_upper.png");
        analyzer::plot_signals(ssb_upper_out_name, &ssb_upper, sample_rate)?;
        
        // Single Sideband (Lower)
        let ssb_lower = analyzer::single_side_band(&samples, sample_rate, self.mod_freq, false);
        let ssb_lower_out_name = self.output_file.replace(".png", "_ssb_lower.png");
        analyzer::plot_signals(ssb_lower_out_name, &ssb_lower, sample_rate)?;
        
        // Demodulate USB
        let demod_upper = analyzer::demodulate_ssb(&ssb_upper, sample_rate, self.mod_freq);
        // let demod_upper_out_name = self.output_file.replace(".png", "_demod_upper.png");
        let demod_upper_out_name = self.output_file.replace(".png", "_demod.png");
        analyzer::plot_signals(demod_upper_out_name, &demod_upper, sample_rate)?;
        
        // // Demodulate LSB
        // let demod_lower = analyzer::demodulate_ssb(&ssb_lower, sample_rate, self.mod_freq);
        // let demod_lower_out_name = self.output_file.replace(".png", "_demod_lower.png");
        // analyzer::plot_signals(demod_lower_out_name, &demod_lower, sample_rate)?;
        
        Ok(())
    }

    pub fn image_loader(&self) -> Option<String> {
        let mut name = self.output_file.clone();
        let mut output: Option<String> = None;
        for indx in self.checkboxes.iter().enumerate() {
            if *indx.1 {
                let plot_type = get_checkbox_by_index(indx.0); 
                if let Some(plot) = plot_type {
                    match plot {
                        CurrentlyEditingConfig::PlotMSG => output = Some(name.clone()),
                        CurrentlyEditingConfig::PlotDSB => {
                            name = name.replace(".png", "_dsb.png");
                            output = Some(name.clone());
                        },
                        CurrentlyEditingConfig::PlotUSB => {
                            name = name.replace(".png", "_ssb_upper.png");
                            output = Some(name.clone());
                        }
                        CurrentlyEditingConfig::PlotLSB => {
                            name = name.replace(".png", "_ssb_lower.png");
                            output = Some(name.clone());
                        }
                        CurrentlyEditingConfig::PlotDemod => {
                            name = name.replace(".png", "_demod.png");
                            output = Some(name.clone());
                        }
                        _ => {}
                    };
                }
            }
        }
        output
    }
}

fn cmd_mode(app: &mut App) -> Result<(), Box<dyn Error>> {
    app.execute_sim()?;
    Ok(())
}

// #[cfg(test)]
//     #[test]
//     fn terminal_mode() -> Result<(), Box<dyn Error>>{
//         let mut app = App::new();
//         app.run()?;
//         assert_eq!(app.state, ConfigState::Main);
//         Ok(())
//     }
