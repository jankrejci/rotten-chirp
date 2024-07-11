#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]
#![feature(type_alias_impl_trait)]
#![allow(async_fn_in_trait)]

use defmt::{info, trace};
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::{
    peripherals::TIMG0,
    prelude::*,
    time,
    timer::timg::{MwdtStage, TimerGroup, Wdt},
};

use esp_backtrace as _;
use esp_println as _;

defmt::timestamp!("{=f32}", time::now().ticks() as f32 / 1_000_000.0);

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    info!("Runtime initialized");

    spawner
        .spawn(watchdog_feed_task(timg0.wdt))
        .expect("BUG: Failed to spawn watchdog task");
}

#[embassy_executor::task]
pub async fn watchdog_feed_task(mut watchdog: Wdt<TIMG0>) {
    const WATCHDOG_TIMEOUT_SECS: u64 = 2;
    const WATCHDOG_FEED_PERIOD: Duration = Duration::from_millis(500);

    watchdog.enable();
    watchdog.set_timeout(MwdtStage::Stage0, WATCHDOG_TIMEOUT_SECS.secs());

    info!("Watchdog feeding task started");
    loop {
        watchdog.feed();
        trace!("Watchdoog feed");
        Timer::after(WATCHDOG_FEED_PERIOD).await;
    }
}
