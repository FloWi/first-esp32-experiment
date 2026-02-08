#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use core::fmt::UpperHex;

use defmt::info;
use esp_hal::clock::CpuClock;
use esp_hal::main;
use esp_hal::rmt::Rmt;
use esp_hal::rng::Rng;
use esp_hal::time::{Duration, Instant, Rate};
use esp_hal_smartled::{SmartLedsAdapter, smart_led_buffer};
use smart_leds::{RGB8, SmartLedsWrite as _};
use {esp_backtrace as _, esp_println as _};

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[main]
fn main() -> ! {
    // generator version: 1.2.0

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    let rmt = Rmt::new(peripherals.RMT, Rate::from_mhz(80)).unwrap();

    // led_buffer needs to be mutable - docs are wrong
    let mut led_buffer = smart_led_buffer!(1);
    let mut led = SmartLedsAdapter::new(rmt.channel0, peripherals.GPIO8, &mut led_buffer);

    const LEVEL: u8 = 10;
    const DIM_FACTOR: f32 = LEVEL as f32 / u8::max_value() as f32;

    fn protect_retina(value: u8) -> u8 {
        (value as f32 * DIM_FACTOR) as u8
    }

    // start with all leds off
    let mut color = RGB8::default();

    let rng = Rng::new();

    // one array with 3 u8 values act as buffer for generating 3 u8 values at once
    let mut rng_buffer = [0u8; 16];
    loop {
        // generate 3 u8 values at once
        rng.read(&mut rng_buffer);

        info!(
            "Blink in color #{:02X}{:02X}{:02X}",
            color.r, color.g, color.b
        );

        led.write([color].into_iter()).unwrap();

        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(500) {}

        // set all primary colors at one - YES, rust can assign multiple props of a struct like this
        (color.r, color.b, color.g) = (
            protect_retina(rng_buffer[0]),
            protect_retina(rng_buffer[1]),
            protect_retina(rng_buffer[2]),
        )
    }
}
