#![no_std]
#![no_main]

use alloc::string::ToString;
use defmt::info;
use destore::{export_schema, Storer};
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::timer::systimer::SystemTimer;
use example::{BlockingAsync, Record, Sub};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern crate alloc;
export_schema!(Record);

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    rtt_target::rtt_init_defmt!();
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    esp_alloc::heap_allocator!(size: 72 * 1024);
    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);

    info!("Embassy initialized!");

    let mut s: Storer<_, Record> = Storer::new(
        BlockingAsync::new(esp_storage::FlashStorage::new()),
        0x620000..(0x620000 + 0x1E0000),
    )
    .await
    .unwrap();

    let r1 = Record::Sub(Sub {
        first_name: "Alice".to_string(),
        last_name: "Summers".to_string(),
        age: 20,
        brothers: 2,
    });
    s.write(&r1).await.unwrap();
    s.write(&Record::Panic("help".to_string())).await.unwrap();

    loop {
        info!("Hello world!");
        Timer::after(Duration::from_secs(1)).await;
    }
}
