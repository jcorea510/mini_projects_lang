use std::{error::Error, io};
use ratatui::style::{Modifier, Style};
use ratatui::{crossterm::{self, event}, layout::{Constraint, Rect}, prelude::Backend, text::Line, Terminal};
use ratatui_image::{picker::Picker, StatefulImage};
use tui_checkbox::Checkbox;
use crate::app::{App, ConfigState, CurrentlyEditingConfig};
use ratatui::layout::{Flex, Layout};

pub fn tui_mode(app: &mut App) -> Result<(), Box<dyn Error>> {
    crossterm::terminal::enable_raw_mode()?;
    let mut stderr = io::stderr();
    crossterm::execute!(
        stderr,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture)?;

    let backend = ratatui::backend::CrosstermBackend::new(stderr);
    let mut terminal = ratatui::Terminal::new(backend)?;
    
    run_app(&mut terminal, app)?;
    
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
        )?;

    if let Some(img) = &mut app.image && let Err(result_img) = img.last_encoding_result().unwrap() {
        eprintln!("Last enconding result of image has some error. I don't understand this: {result_img}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<bool> {
    loop {
        terminal.draw(|f| ui(f, app))?;
        if let crossterm::event::Event::Key(key) = event::read()? {
            if key.kind == crossterm::event::KeyEventKind::Release {
                continue;
            }

            // Global shortcuts (work in any state)
            if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                match key.code {
                    // CTRL-C: Exit
                    crossterm::event::KeyCode::Char('c') => {
                        return Ok(true);
                    }
                    // CTRL-R: Run simulation
                    crossterm::event::KeyCode::Char('r') => {
                        if app.execute_sim().is_err() {
                            return Ok(false);
                        }
                    }
                    // CTRL-P: Plot results
                    crossterm::event::KeyCode::Char('p') => {
                        let picker = Picker::from_fontsize((8, 12));
                        if let Some(filename) = app.image_loader()
                            && let Ok(dyn_img) = image::ImageReader::open(filename)?.decode() {
                                let image = picker.new_resize_protocol(dyn_img);
                                app.image = Some(image);
                        }
                    }
                    _ => {}
                }
                continue;
            }

            // Arrow navigation (works in all states)
            match key.code {
                crossterm::event::KeyCode::Down => {
                    app.next_editing_config();
                    continue;
                }
                crossterm::event::KeyCode::Up => {
                    app.prev_editing_config();
                    continue;
                }
                // 1: Go to Files state
                crossterm::event::KeyCode::Char('1') if app.current_editing_config.is_none() => {
                    app.change_state_to(ConfigState::Files);
                }
                // 2: Go to Mod state
                crossterm::event::KeyCode::Char('2') if app.current_editing_config.is_none() => {
                    app.change_state_to(ConfigState::Mod);
                }
                // 3: Go to Demod state
                crossterm::event::KeyCode::Char('3') if app.current_editing_config.is_none() => {
                    app.change_state_to(ConfigState::Demod);
                }
                // 4: Go to Plot state
                crossterm::event::KeyCode::Char('4') if app.current_editing_config.is_none() => {
                    app.change_state_to(ConfigState::Plot);
                }
                _ => {}
            }

            // State-specific key handling
            match app.state {
                ConfigState::Files => {
                    match key.code {
                        crossterm::event::KeyCode::Esc => {
                            app.current_editing_config = None;
                        }
                        crossterm::event::KeyCode::Backspace => {
                            if let Some(curr) = &app.current_editing_config {
                                match curr {
                                    CurrentlyEditingConfig::InputFile => { app.input_file.pop(); }
                                    CurrentlyEditingConfig::OutputFile => { app.output_file.pop(); }
                                    _ => {}
                                }
                            }
                        }
                        crossterm::event::KeyCode::Char(ch) => {
                            if let Some(curr) = &app.current_editing_config {
                                match curr {
                                    CurrentlyEditingConfig::InputFile => app.input_file.push(ch),
                                    CurrentlyEditingConfig::OutputFile => app.output_file.push(ch),
                                    _ => {}
                                }
                            }
                        }
                        _ => {}
                    }
                }
                ConfigState::Mod => {
                    match key.code {
                        crossterm::event::KeyCode::Esc => {
                            app.current_editing_config = None;
                        }
                        crossterm::event::KeyCode::Backspace => {
                            if let Some(curr) = &app.current_editing_config 
                                && let CurrentlyEditingConfig::ModFreq = curr {
                                    let mut current_input = app.mod_freq.to_string();
                                    current_input.pop();
                                    if let Ok(freq) = current_input.parse::<f32>() {
                                        app.mod_freq = freq;
                                    } else if current_input.is_empty() {
                                        app.mod_freq = 0.0;
                                }
                            }
                        }
                        crossterm::event::KeyCode::Char(ch) => {
                            if let Some(curr) = &app.current_editing_config 
                                && let CurrentlyEditingConfig::ModFreq = curr {
                                    let mut current_input = app.mod_freq.to_string();
                                    current_input.push(ch);
                                    if let Ok(freq) = current_input.parse::<f32>() {
                                        app.mod_freq = freq;
                                }
                            }
                        }
                        _ => {}
                    }
                }
                ConfigState::Demod => {
                    match key.code {
                        crossterm::event::KeyCode::Esc => {
                            app.current_editing_config = None;
                        }
                        crossterm::event::KeyCode::Backspace => {
                            if let Some(curr) = &app.current_editing_config {
                                match curr {
                                    CurrentlyEditingConfig::DemodFreqError => {
                                        let mut current_input = app.demod_freq_error.to_string();
                                        current_input.pop();
                                        if let Ok(freq_err) = current_input.parse::<f32>() {
                                            app.demod_freq_error = freq_err;
                                        } else if current_input.is_empty() {
                                            app.demod_freq_error = 0.0;
                                        }
                                    }
                                    CurrentlyEditingConfig::DemodPhaseError => {
                                        let mut current_input = app.demod_phase_error.to_string();
                                        current_input.pop();
                                        if let Ok(phase_err) = current_input.parse::<f32>() {
                                            app.demod_phase_error = phase_err;
                                        } else if current_input.is_empty() {
                                            app.demod_phase_error = 0.0;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        crossterm::event::KeyCode::Char(ch) => {
                            if let Some(curr) = &app.current_editing_config {
                                match curr {
                                    CurrentlyEditingConfig::DemodFreqError => {
                                        let mut current_input = app.demod_freq_error.to_string();
                                        current_input.push(ch);
                                        if let Ok(freq_err) = current_input.parse::<f32>() {
                                            app.demod_freq_error = freq_err;
                                        }
                                    }
                                    CurrentlyEditingConfig::DemodPhaseError => {
                                        let mut current_input = app.demod_phase_error.to_string();
                                        current_input.push(ch);
                                        if let Ok(phase_err) = current_input.parse::<f32>() {
                                            app.demod_phase_error = phase_err;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        _ => {}
                    }
                }
                ConfigState::Plot => {
                    match key.code {
                        crossterm::event::KeyCode::Enter | crossterm::event::KeyCode::Char(' ') => {
                            app.toggle_checkbox();
                        }
                        crossterm::event::KeyCode::Esc => {
                            app.current_editing_config = None;
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

fn ui(frame: &mut ratatui::Frame, app: &mut App) {
    let interfaze_log_layout = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints(vec![
            Constraint::Percentage(90),
            Constraint::Percentage(10), // this side is used to print logs
        ])
        .split(frame.area());

    let interfaze_app_layout = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints(vec![
            Constraint::Percentage(30), // this layout is to configure the simulator
            Constraint::Percentage(70), // this layout is to visualize the result
        ])
        .split(interfaze_log_layout[0]);

    let interfaze_config_layout = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints(vec![
            Constraint::Percentage(20), // files input/output
            Constraint::Percentage(20), // configuraction of modulator
            Constraint::Percentage(20), // configuraciton of demodulator
            Constraint::Percentage(40), // select what to show in plot widget
        ])
        .split(interfaze_app_layout[0]);

    file_in_out(frame, interfaze_config_layout[0], app);
    modulator_config(frame, interfaze_config_layout[1], app);
    demodulator_config(frame, interfaze_config_layout[2], app);
    show_config(frame, interfaze_config_layout[3], app);
    show_plot(frame, interfaze_app_layout[1], app);
    render_log_frame(frame, interfaze_log_layout[1], app);
}

fn file_in_out(frame: &mut ratatui::Frame, chunks: Rect, app: &App) {
    let is_frame_active = app.state == ConfigState::Files;
    let title_block = make_config_block("Inputs", is_frame_active);

    let mut is_editing_wave_input = false;
    let mut is_editing_wave_output = false;
    if let Some(curr) = &app.current_editing_config {
        match curr {
            CurrentlyEditingConfig::InputFile => is_editing_wave_input = true,
            CurrentlyEditingConfig::OutputFile => is_editing_wave_output = true,
            _ => {}
        }
    };

    let inputs_text = ratatui::text::Text::from(vec![
        ratatui::text::Line::from(vec![
            "Wave file: ".into(),
            app.input_file.clone().into(),
        ]).style(input_style(is_editing_wave_input)),
        ratatui::text::Line::from(vec![
            "Output name: ".into(),
            app.output_file.clone().into(),
        ]).style(input_style(is_editing_wave_output))])
        .style(ratatui::style::Style::default());

    let title = ratatui::widgets::Paragraph::new(inputs_text)
        .block(title_block);

    frame.render_widget(title, chunks);
}

fn modulator_config(frame: &mut ratatui::Frame, chunks: Rect, app: &App) {
    let is_frame_active = app.state == ConfigState::Mod;
    let title_block = make_config_block("Modulator", is_frame_active);

    let mut is_editing_mod_freq = false;
    if let Some(curr) = &app.current_editing_config &&
        let CurrentlyEditingConfig::ModFreq = curr { is_editing_mod_freq = true };

    let inputs_text = ratatui::text::Text::from(vec![
        ratatui::text::Line::from(vec![
            "Frequency [Hz]: ".into(),
            app.mod_freq.to_string().into(),
        ]).style(input_style(is_editing_mod_freq))])
        .style(ratatui::style::Style::default());

    let title = ratatui::widgets::Paragraph::new(inputs_text)
        .block(title_block);
    frame.render_widget(title, chunks);
}

fn demodulator_config(frame: &mut ratatui::Frame, chunks: Rect, app: &App) {
    let is_frame_active = app.state == ConfigState::Demod;
    let title_block = make_config_block("Demodulator", is_frame_active);

    let mut is_editing_freq_error = false;
    let mut is_editing_phase_error = false;
    if let Some(curr) = &app.current_editing_config {
        match curr {
            CurrentlyEditingConfig::DemodFreqError => is_editing_freq_error = true,
            CurrentlyEditingConfig::DemodPhaseError => is_editing_phase_error = true,
            _ => {}
        }
    };

    let inputs_text = ratatui::text::Text::from(vec![
        ratatui::text::Line::from(vec![
            "%Error Frequency: ".into(),
            app.demod_freq_error.to_string().into(),
        ]).style(input_style(is_editing_freq_error)),
        ratatui::text::Line::from(vec![
            "%Error Phase: ".into(),
            app.demod_phase_error.to_string().into(),
        ]).style(input_style(is_editing_phase_error))])
        .style(ratatui::style::Style::default());

    let title = ratatui::widgets::Paragraph::new(inputs_text)
        .block(title_block);

    frame.render_widget(title, chunks);
}

fn show_config(frame: &mut ratatui::Frame, chunks: Rect, app: &mut App) {
    let is_frame_active = app.state == ConfigState::Plot;
    let title_block = make_config_block("Plotter", is_frame_active);

    let inner = title_block.inner(chunks);
    frame.render_widget(title_block, chunks);

    let items_layout = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(inner);
        
    let mut is_message_sended_active = false;
    let mut is_double_side_band_active = false;
    let mut is_upper_side_band_active = false;
    let mut is_lower_side_band_active = false;
    let mut is_demod_active = false;
    if let Some(curr) = &app.current_editing_config {
        match curr {
            CurrentlyEditingConfig::PlotMSG => is_message_sended_active = true,
            CurrentlyEditingConfig::PlotDSB => is_double_side_band_active = true,
            CurrentlyEditingConfig::PlotUSB => is_upper_side_band_active = true,
            CurrentlyEditingConfig::PlotLSB => is_lower_side_band_active = true,
            CurrentlyEditingConfig::PlotDemod => is_demod_active = true,
            _ => {}
        }
    }

    let checkbox_message_send = Checkbox::new("Message sended", app.checkboxes[0])
        .checkbox_style(ratatui::style::Style::default()
            .fg(ratatui::style::Color::LightGreen)
            .add_modifier(Modifier::BOLD))
        .label_style(Style::default().fg(ratatui::style::Color::Gray))
        .checked_symbol("✅ ")
        .unchecked_symbol("⬜ ")
        .style(input_style(is_message_sended_active));
    frame.render_widget(checkbox_message_send, items_layout[0]);

    let checkbox_dsb = Checkbox::new("Double side band", app.checkboxes[1])
        .checkbox_style(ratatui::style::Style::default()
            .fg(ratatui::style::Color::LightGreen)
            .add_modifier(Modifier::BOLD))
        .label_style(Style::default().fg(ratatui::style::Color::Gray))
        .checked_symbol("✅ ")
        .unchecked_symbol("⬜ ")
        .style(input_style(is_double_side_band_active));
    frame.render_widget(checkbox_dsb, items_layout[1]);

    let checkbox_upper_side = Checkbox::new("Upper side band", app.checkboxes[2])
        .checkbox_style(ratatui::style::Style::default()
            .fg(ratatui::style::Color::LightGreen)
            .add_modifier(Modifier::BOLD))
        .label_style(Style::default().fg(ratatui::style::Color::Gray))
        .checked_symbol("✅ ")
        .unchecked_symbol("⬜ ")
        .style(input_style(is_upper_side_band_active));
    frame.render_widget(checkbox_upper_side, items_layout[2]);

    let checkbox_lower_side = Checkbox::new("Lower side band", app.checkboxes[3])
        .checkbox_style(ratatui::style::Style::default()
            .fg(ratatui::style::Color::LightGreen)
            .add_modifier(Modifier::BOLD))
        .label_style(Style::default().fg(ratatui::style::Color::Gray))
        .checked_symbol("✅ ")
        .unchecked_symbol("⬜ ")
        .style(input_style(is_lower_side_band_active));
    frame.render_widget(checkbox_lower_side, items_layout[3]);

    let checkbox_received = Checkbox::new("Received message", app.checkboxes[4])
        .checkbox_style(ratatui::style::Style::default()
            .fg(ratatui::style::Color::LightGreen)
            .add_modifier(Modifier::BOLD))
        .label_style(Style::default().fg(ratatui::style::Color::Gray))
        .checked_symbol("✅ ")
        .unchecked_symbol("⬜ ")
        .style(input_style(is_demod_active));
    frame.render_widget(checkbox_received, items_layout[4]);
}

fn show_plot(frame: &mut ratatui::Frame, chunks: Rect, app: &mut App) {
    let title_block = make_config_block("Plots", false);
    
    let inner = title_block.inner(chunks);
    frame.render_widget(title_block, chunks);

    let img_rec = center_image(inner, Constraint::Percentage(95), Constraint::Percentage(95));
    let imgage_default = StatefulImage::default();
    if let Some(img) = &mut app.image {
        frame.render_stateful_widget(imgage_default, img_rec, img);
    }
}

fn center_image(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}

fn make_config_block<'a>(title: &'a str, is_active: bool) -> ratatui::widgets::Block<'a> {
    let title_frame = Line::from(format!(" {} ", title))
        .style(Style::default().add_modifier(Modifier::BOLD));

    let style = if is_active {
        ratatui::style::Style::default().fg(ratatui::style::Color::Yellow)
    } else {
        ratatui::style::Style::default()
    };

    ratatui::widgets::Block::default()
        .title(title_frame.centered())
        .borders(ratatui::widgets::Borders::ALL)
        .style(style)
}

fn input_style(is_active: bool) -> ratatui::style::Style {
    if is_active {
        ratatui::style::Style::default().fg(ratatui::style::Color::Blue)
    } else {
        ratatui::style::Style::default()
    }
}

fn render_log_frame(frame: &mut ratatui::Frame, area: Rect, app: &App) {
    let title_frame = Line::from(" Logs ")
        .style(Style::default().add_modifier(Modifier::BOLD));

    let mut style = ratatui::style::Style::default();
    if app.has_error {
        style = style.fg(ratatui::style::Color::Red);
    }

    let block = ratatui::widgets::Block::default()
        .title(title_frame.centered())
        .borders(ratatui::widgets::Borders::ALL)
        .style(style);

    let log_text = if app.error_buffer.is_empty() {
        ratatui::text::Text::from("")
    } else {
        ratatui::text::Text::from(app.error_buffer.clone())
    };

    let paragraph = ratatui::widgets::Paragraph::new(log_text).block(block);
    frame.render_widget(paragraph, area);
}
