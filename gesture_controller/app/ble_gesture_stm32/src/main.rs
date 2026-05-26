#![no_std]
#![no_main]

mod fmt;

use defmt::info;
#[cfg(not(feature = "defmt"))]
use panic_halt as _;
#[cfg(feature = "defmt")]
use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    gpio::{AnyPin, Level, Output, Speed},
    peripherals,
    Peri,
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::Timer;
use embassy_stm32::usart::{Config as UartConfig, Uart};
use embassy_stm32::mode::Async;
use embassy_stm32::rcc::*;

// ── Gesture ──────────────────────────────────────────────────────────────────
//
//  Byte values match the Linux sender (serial_controller.hpp):
//    Left  = 0x01  →  LEDs: 0 0 1
//    Right = 0x02  →  LEDs: 0 1 0
//    Close = 0x03  →  LEDs: 0 1 1
//    Open  = 0x04  →  LEDs: 1 0 0
//
//  The 3-LED binary encoding is simply the numeric value of the byte
//  spread across (led2 = bit2, led1 = bit1, led0 = bit0).

#[derive(Clone, Copy, defmt::Format)]
#[repr(u8)]
enum Gesture {
    Left  = 0x01,
    Right = 0x02,
    Close = 0x03,
    Open  = 0x04,
}

impl Gesture {
    fn from_byte(b: u8) -> Option<Self> {
        match b {
            0x01 => Some(Self::Left),
            0x02 => Some(Self::Right),
            0x03 => Some(Self::Close),
            0x04 => Some(Self::Open),
            _    => None,
        }
    }

    /// Returns (led2, led1, led0) where led2 is the MSB.
    fn led_bits(self) -> (bool, bool, bool) {
        let v = self as u8;
        (v & 0b100 != 0, v & 0b010 != 0, v & 0b001 != 0)
    }
}

// ── Shared signal (USB task -> LED task) ──────────────────────────────────────
//
static GESTURE: Signal<CriticalSectionRawMutex, Gesture> = Signal::new();

bind_interrupts!(struct Irqs {
    USART1 => embassy_stm32::usart::InterruptHandler<peripherals::USART1>;
});

// ── Tasks ────────────────────────────────────────────────────────────────────
#[embassy_executor::task]
async fn uart_task(
    mut uart: Uart<'static, Async>,
) {
    let mut buf = [0u8; 1];

    loop {
        match uart.read(&mut buf).await {
            Ok(_) => {
                info!("RX byte: 0x{:02X}", buf[0]);

                // Echo back to PC
                if uart.write(&buf).await.is_err() {
                    info!("UART TX failed");
                }

                if let Some(g) = Gesture::from_byte(buf[0]) {
                    GESTURE.signal(g);
                } else {
                    info!("Invalid gesture");
                }
            }

            Err(_) => {
                info!("UART RX error");
            }
        }
    }
}

/// Drives 3 LEDs in binary to display the last received gesture.
///
/// Pin assignment (suggested — change to suit your wiring):
///   PB12 → LED0 (bit 0, LSB)
///   PB13 → LED1 (bit 1)
///   PB14 → LED2 (bit 2, MSB)
#[embassy_executor::task]
async fn gesture_led_task(
    pin0: Peri<'static, AnyPin>, // bit 0
    pin1: Peri<'static, AnyPin>, // bit 1
    pin2: Peri<'static, AnyPin>, // bit 2
) {
    let mut led0 = Output::new(pin0, Level::Low, Speed::Low);
    let mut led1 = Output::new(pin1, Level::Low, Speed::Low);
    let mut led2 = Output::new(pin2, Level::Low, Speed::Low);

    loop {
        let gesture = GESTURE.wait().await;
        let (b2, b1, b0) = gesture.led_bits();

        led0.set_level(if b0 { Level::High } else { Level::Low });
        led1.set_level(if b1 { Level::High } else { Level::Low });
        led2.set_level(if b2 { Level::High } else { Level::Low });

        info!(
            "LEDs [led2 led1 led0] = [{} {} {}]",
            b2 as u8, b1 as u8, b0 as u8
        );

        Timer::after_millis(500).await;
    }
}

/// Blinks the onboard LED (PC13) as a heartbeat so you can see the
/// firmware is alive even before the USB host connects.
#[embassy_executor::task]
async fn heartbeat_task(pin: Peri<'static, AnyPin>) {
    let mut led = Output::new(pin, Level::High, Speed::Low);
    loop {
        led.toggle();
        Timer::after_millis(500).await;
    }
}

// ── Entry point ──────────────────────────────────────────────────────────────

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut rcc = embassy_stm32::Config::default();
    rcc.rcc.hse = Some(Hse {
        freq: embassy_stm32::time::Hertz(8_000_000),
        mode: HseMode::Oscillator,
    });
    rcc.rcc.pll = Some(Pll {
        src: PllSource::HSE,
        prediv: PllPreDiv::DIV1,
        mul: PllMul::MUL9,   // 8MHz * 9 = 72MHz
    });
    rcc.rcc.sys = Sysclk::PLL1_P;
    rcc.rcc.apb1_pre = APBPrescaler::DIV2;   // 36MHz
    rcc.rcc.apb2_pre = APBPrescaler::DIV1;   // 72MHz  ← USART1 is on APB2

    let p = embassy_stm32::init(rcc);

    let mut cfg = UartConfig::default();
    cfg.baudrate = 9600;

    let uart = Uart::new(
        p.USART1,
        p.PB7,
        p.PB6,
        Irqs,
        p.DMA1_CH4,
        p.DMA1_CH5,
        cfg,
    ).unwrap();

    if spawner.spawn(uart_task(uart)).is_err() {
        info!("Spaned uart task failed");
    }
    spawner
        .spawn(gesture_led_task(
            p.PA10.into(), // LED0 – bit 0
            p.PB8.into(), // LED1 – bit 1
            p.PB9.into(), // LED2 – bit 2
        ))
        .unwrap();
    spawner.spawn(heartbeat_task(p.PC13.into())).unwrap();
}
