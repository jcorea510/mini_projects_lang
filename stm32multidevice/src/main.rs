#![no_std]
#![no_main]

mod fmt;

use core::fmt::Write;
use defmt::{Debug2Format, error};
use embedded_graphics::{
    image::{Image, ImageRaw},
    mono_font::{MonoTextStyleBuilder, ascii::FONT_6X10},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle},
    text::{Baseline, Text},
};
use heapless::String;
#[cfg(not(feature = "defmt"))]
use panic_halt as _;
#[cfg(feature = "defmt")]
use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;
use embassy_stm32::{
    Peri,
    adc::Adc,
    bind_interrupts,
    exti::{AnyChannel, ExtiInput},
    gpio::{AnyPin, Level, Output, OutputType, Pull, Speed},
    i2c::{self, I2c, Master},
    mode::Async,
    peripherals::{self, ADC1},
    time::Hertz,
    timer::{
        low_level::CountingMode,
        simple_pwm::{PwmPin, SimplePwm},
    },
};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Instant, Timer};
// use fmt::info; //used to send debug information via programmer
use ssd1306::{I2CDisplayInterface, Ssd1306, prelude::*};
use static_cell::StaticCell;

static DATA: StaticCell<Mutex<NoopRawMutex, App>> = StaticCell::new();

#[derive(Default)]
struct DataSensor {
    pub temperature: String<64>,
    pub distance: String<64>,
}

fn init() -> &'static Mutex<NoopRawMutex, App> {
    DATA.init(Mutex::new(App::new()))
}

enum AppState {
    Intro,
    DataDisplay,
    Image,
}

struct App {
    pub state: AppState,
    pub data: DataSensor,
}

impl App {
    fn new() -> Self {
        let data = DataSensor::default();
        Self {
            state: AppState::Intro,
            data,
        }
    }

    fn change_state(&mut self) {
        match self.state {
            AppState::Intro => self.state = AppState::DataDisplay,
            AppState::DataDisplay => self.state = AppState::Image,
            AppState::Image => self.state = AppState::DataDisplay,
        };
    }

    fn get_state(&self) -> AppState {
        match self.state {
            AppState::Intro => AppState::Intro,
            AppState::DataDisplay => AppState::DataDisplay,
            AppState::Image => AppState::Image,
        }
    }
}

#[embassy_executor::task]
async fn display_task(
    data: &'static Mutex<NoopRawMutex, App>,
    i2c_device: I2c<'static, Async, Master>,
) {
    let interface = I2CDisplayInterface::new(i2c_device);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();

    display.clear_buffer();
    if let Err(e) = display.flush() {
        error!("Failed to flush (continuing): {}", Debug2Format(&e));
    }

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();
    let rect_style = PrimitiveStyleBuilder::new()
        .stroke_width(1)
        .stroke_color(BinaryColor::On)
        .build();

    loop {
        display.clear(BinaryColor::Off).unwrap();

        {
            let state = {
                let guard = data.lock().await;
                guard.get_state()
            };

            match state {
                AppState::Intro => {
                    Text::with_baseline("Hello world!", Point::zero(), text_style, Baseline::Top)
                        .draw(&mut display)
                        .unwrap();
                    display.flush().unwrap();

                    Rectangle::new(Point::new(50, 12), Size::new(10, 10))
                        .into_styled(rect_style)
                        .draw(&mut display)
                        .unwrap();
                    {
                        let mut guard = data.lock().await;
                        guard.change_state();
                    }
                }
                AppState::DataDisplay => {
                    Rectangle::new(Point::new(1, 1), Size::new(127, 63))
                        .into_styled(rect_style)
                        .draw(&mut display)
                        .unwrap();

                    let distance_text = {
                        let guard = data.lock().await;
                        guard.data.distance.clone()
                    };
                    Text::with_baseline(
                        distance_text.as_str(),
                        Point::new(4, 4),
                        text_style,
                        Baseline::Top,
                    )
                    .draw(&mut display)
                    .unwrap();

                    let temperature_text = {
                        let guard = data.lock().await;
                        guard.data.temperature.clone()
                    };
                    Text::with_baseline(
                        temperature_text.as_str(),
                        Point::new(4, 15),
                        text_style,
                        Baseline::Top,
                    )
                    .draw(&mut display)
                    .unwrap();
                }
                AppState::Image => {
                    let raw: ImageRaw<BinaryColor> =
                        ImageRaw::new(include_bytes!("./rust.raw"), 64);
                    let im = Image::new(&raw, Point::new(32, 0));
                    im.draw(&mut display).unwrap();
                }
            }
        }

        if let Err(e) = display.flush() {
            error!("Flush failed: {}", Debug2Format(&e));
        }

        Timer::after_secs(2).await;
    }
}

#[embassy_executor::task]
async fn led_task(pin: Peri<'static, AnyPin>) {
    let mut led = Output::new(pin, Level::High, Speed::Low);
    loop {
        // info!("Hello, World!");
        led.set_high();
        Timer::after_millis(500).await;
        led.set_low();
        Timer::after_millis(500).await;
    }
}

#[embassy_executor::task]
async fn distance_sensor_task(
    data: &'static Mutex<NoopRawMutex, App>,
    trigger: Peri<'static, AnyPin>,
    echo: Peri<'static, AnyPin>,
    channel: Peri<'static, AnyChannel>,
) {
    let mut trigger = Output::new(trigger, Level::Low, Speed::Low);
    let mut echo = ExtiInput::new(echo, channel, Pull::None);

    loop {
        trigger.set_low();
        Timer::after_millis(5).await;

        trigger.set_high();
        Timer::after_millis(10).await;
        trigger.set_low();

        echo.wait_for_high().await;
        let start = Instant::now();

        echo.wait_for_low().await;
        let duration = start.elapsed();
        let duration_us = duration.as_micros();
        let distance_cm = duration_us / 58;

        {
            let mut guard = data.lock().await;
            guard.data.distance.clear();
            writeln!(guard.data.distance, "Distance: {} cm\r", distance_cm).unwrap();
        }

        Timer::after_millis(100).await;
    }
}

#[embassy_executor::task]
async fn motor_task(
    pwm_pin: Peri<'static, peripherals::PA9>,
    ptim1: Peri<'static, peripherals::TIM1>,
) {
    let servo_motor = PwmPin::new(pwm_pin, OutputType::PushPull);
    let mut pwm = SimplePwm::new(
        ptim1,
        None,
        Some(servo_motor),
        None,
        None,
        Hertz::hz(50),
        CountingMode::EdgeAlignedUp,
    );
    let max_duty = pwm.ch1().max_duty_cycle();

    let min_duty = max_duty / 20; // ~5% (0°)
    let center_duty = (max_duty / 40) * 3; // ~7.5% (90°)
    let max_duty_servo = max_duty / 10; // ~10% (180°)

    pwm.ch2().enable(); // Enable immediately

    loop {
        // Move to 0°
        pwm.ch2().set_duty_cycle(min_duty);
        Timer::after_secs(1).await;

        // Move to 90°
        pwm.ch2().set_duty_cycle(center_duty);
        Timer::after_secs(1).await;

        // Move to 180°
        pwm.ch2().set_duty_cycle(max_duty_servo);
        Timer::after_secs(1).await;
    }
}

#[embassy_executor::task]
async fn temperature_sensor_task(
    data: &'static Mutex<NoopRawMutex, App>,
    adc_channel: Peri<'static, ADC1>,
    mut adc_pin: Peri<'static, peripherals::PA1>,
) {
    let mut adc = Adc::new(adc_channel);
    const R0: f64 = 5000.0; // Fixed: 5kOhm thermistor
    const R_FIXED: f64 = 5000.0; // 5kOhm resistor to ground
    const B: f64 = 3975.0; // Fixed: B coefficient
    const T0: f64 = 298.15; // 25 C in Kelvin

    loop {
        let sample = adc.read(&mut adc_pin).await;

        let r_thermistor = R_FIXED * (4095.0 / sample as f64 - 1.0);

        // Temperature from Steinhart-Hart simplified (B parameter equation)
        let temperature = 1.0 / (libm::log(r_thermistor / R0) / B + 1.0 / T0) - 273.15;

        {
            let mut guard = data.lock().await;
            guard.data.temperature.clear();
            writeln!(guard.data.temperature, "Temp: {:.1} C", temperature).unwrap();
        }

        Timer::after_millis(1000).await;
    }
}

#[embassy_executor::task]
async fn input_handler(
    app: &'static Mutex<NoopRawMutex, App>,
    pin: Peri<'static, AnyPin>,
    channel: Peri<'static, AnyChannel>,
) {
    let mut button = ExtiInput::new(pin, channel, Pull::Down);

    loop {
        button.wait_for_rising_edge().await;
        //debounce dealy
        Timer::after_millis(50).await;

        if button.is_high() {
            let mut guard = app.lock().await;
            guard.change_state();
        }

        button.wait_for_falling_edge().await;
        Timer::after_millis(50).await;
    }
}

bind_interrupts!(struct Irqs {
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>; // important to add for i2c
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
    ADC1_2 => embassy_stm32::adc::InterruptHandler<peripherals::ADC1>; // important to add for adc
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let i2c = i2c::I2c::new(
        p.I2C1,
        p.PB6,
        p.PB7,
        Irqs,
        p.DMA1_CH6,
        p.DMA1_CH7,
        Default::default(),
    );

    let app = init();

    spawner.spawn(display_task(app, i2c)).unwrap();
    spawner.spawn(led_task(p.PA0.into())).unwrap();
    spawner
        .spawn(temperature_sensor_task(app, p.ADC1, p.PA1))
        .unwrap();
    spawner
        .spawn(distance_sensor_task(
            app,
            p.PB0.into(),
            p.PB1.into(),
            p.EXTI1.into(),
        ))
        .unwrap();
    spawner.spawn(motor_task(p.PA9, p.TIM1)).unwrap();
    spawner
        .spawn(input_handler(app, p.PB8.into(), p.EXTI8.into()))
        .unwrap();
}
