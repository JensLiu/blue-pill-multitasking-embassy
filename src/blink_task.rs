use defmt::info;
use embassy_futures::select::{select, Either};
use embassy_stm32::{
    gpio::{Level, Output, Speed},
    peripherals,
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Duration, Timer};

type MyLedPin = peripherals::PB13;

pub static STATUS_SIGNAL: Signal<CriticalSectionRawMutex, bool> = Signal::new();

#[embassy_executor::task]
pub async fn task(led_pin: MyLedPin) {
    let mut led = Output::new(led_pin, Level::High, Speed::Medium);

    loop {
        if let Either::First(running) = select(STATUS_SIGNAL.wait(), blink(&mut led)).await {
            if !running {
                on_wait(&mut led).await;
            }
        }
    }
}

async fn blink(led: &mut Output<'_, MyLedPin>) {
    // info!("BLINK TASK RUNNING");
    // info!("blink begin");
    led.set_high();
    // info!("high yeild");
    Timer::after(Duration::from_millis(100)).await;
    // info!("high yield done");
    led.set_low();
    // info!("low yield");
    Timer::after(Duration::from_millis(100)).await;
    // info!("low yield done")
}

async fn on_wait(led: &mut Output<'_, MyLedPin>) {
    info!("STOP BLINK TASK");
    led.set_low();
    loop {
        if STATUS_SIGNAL.wait().await {
            info!("RESUME BLINK TASK");
            break;
        }
    }
}
