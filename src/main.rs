#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]
#![feature(type_alias_impl_trait)]
#![allow(async_fn_in_trait)]

use defmt::{info, trace};
use edrv_st7735::{Display160x80Type2, ST7735};
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Duration, Timer};
use embedded_graphics::{
    framebuffer::Framebuffer,
    image::{Image, ImageRaw, ImageRawLE},
    mono_font::{ascii::FONT_9X18_BOLD, MonoTextStyle},
    pixelcolor::{raw::BigEndian, Rgb565},
    prelude::*,
    primitives::PrimitiveStyle,
    text::{Alignment, Text},
};
use esp_hal::{
    delay::Delay,
    dma::*,
    dma_buffers,
    gpio::{Level, Output},
    peripherals::TIMG0,
    prelude::*,
    spi::{
        master::{Config, Spi},
        SpiMode,
    },
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

    let sclk = peripherals.GPIO2;
    let mosi = peripherals.GPIO3;

    let dma = Dma::new(peripherals.DMA);
    let dma_channel = dma.channel0;

    let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) = dma_buffers!(32000);
    let dma_rx_buf = DmaRxBuf::new(rx_descriptors, rx_buffer).unwrap();
    let dma_tx_buf = DmaTxBuf::new(tx_descriptors, tx_buffer).unwrap();

    let spi = Spi::new_with_config(
        peripherals.SPI2,
        Config {
            frequency: 40.MHz(),
            mode: SpiMode::Mode0,
            ..Config::default()
        },
    )
    .with_sck(sclk)
    .with_mosi(mosi)
    .with_dma(dma_channel.configure(false, DmaPriority::Priority0))
    .with_buffers(dma_rx_buf, dma_tx_buf)
    .into_async();

    let dc = Output::new(peripherals.GPIO6, Level::Low);
    let cs = Output::new(peripherals.GPIO7, Level::High);
    let _rst = Output::new(peripherals.GPIO10, Level::High);

    let spi_bus = Mutex::<NoopRawMutex, _>::new(spi);
    let spi_dev = embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice::new(&spi_bus, cs);
    let mut display: ST7735<Display160x80Type2, _, _> = ST7735::new(spi_dev, dc);

    let mut fb = Framebuffer::<
        Rgb565,
        _,
        BigEndian,
        160,
        80,
        { embedded_graphics::framebuffer::buffer_size::<Rgb565>(80, 160) },
    >::new();

    display.init(&mut Delay::new()).await.unwrap();
    display.clear(Rgb565::BLACK).await.unwrap();

    fb.bounding_box()
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::RED, 2))
        .draw(&mut fb)
        .unwrap();

    let center_point = Point::new(160 / 2, 80 / 2);
    let image_raw: ImageRawLE<Rgb565> = ImageRaw::new(include_bytes!("../assets/rust.raw"), 64);
    Image::with_center(&image_raw, center_point)
        .draw(&mut fb)
        .unwrap();

    let style = MonoTextStyle::new(&FONT_9X18_BOLD, Rgb565::CSS_INDIAN_RED);
    Text::with_alignment("Hello Rust!", center_point, style, Alignment::Center)
        .draw(&mut fb)
        .unwrap();

    display.write_framebuffer(fb.data()).await.unwrap();

    info!("Done");
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
