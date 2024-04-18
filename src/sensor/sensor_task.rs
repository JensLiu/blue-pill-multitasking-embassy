use core::fmt;

use defmt::Format;
use embassy_futures::select::{select, Either};
use embassy_stm32::{
    bind_interrupts, i2c,
    peripherals::{self, DMA1_CH6, DMA1_CH7},
};

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{with_timeout, Duration, Timer};
use heapless::String;

use crate::{
    fmt::{error, info},
    io::writer_task,
    sensor::dht20_impl::Dht20,
};

// the peripherals, ports and DMA usage of this task is specified here
pub type MyI2cPeripheral = peripherals::I2C1;
pub type MyTxDma = DMA1_CH6;
pub type MyRxDma = DMA1_CH7;

pub type MyI2cType = i2c::I2c<'static, MyI2cPeripheral, MyTxDma, MyRxDma>;
pub type MySensor = Dht20<'static, MyI2cPeripheral, MyTxDma, MyRxDma>;

static TIMEOUT: Duration = Duration::from_millis(50);

pub static STATUS_SIGNAL: Signal<CriticalSectionRawMutex, bool> = Signal::new();

// binding interrupts
bind_interrupts!(pub struct I2cIrqs {
    I2C1_EV => i2c::EventInterruptHandler<MyI2cPeripheral>;
    I2C1_ER => i2c::ErrorInterruptHandler<MyI2cPeripheral>;
});

#[embassy_executor::task]
pub async fn task(i2c: MyI2cType) {
    let mut sensor = Dht20::new(i2c, TIMEOUT);
    if None == sensor.init().await {
        error!("Initialisation Failed")
    }

    loop {
        if let Either::First(running) =
            select(STATUS_SIGNAL.wait(), check_sensor(&mut sensor)).await
        {
            if !running {
                on_wait(&mut sensor).await
            }
        }
    }
}

async fn check_sensor(sensor: &mut MySensor) {
    // debug!("SENSOR TASK RUNNING");
    if let Some((hum, temp)) = sensor.read().await {
        let mut buf = [0u8; writer_task::BUFFER_SIZE];
        let s = format_no_std::show(
            &mut buf,
            format_args!("Humidity: {}, Temperature: {}", hum, temp),
        )
        .unwrap();
        let len = s.len();
        info!("{}", s);
        writer_task::DATAPIPE.write(&buf[..len]).await;
    }
    Timer::after(Duration::from_millis(500)).await;
}

async fn on_wait(sensor: &mut MySensor) {
    info!("STOP SENSOR TASK");
    loop {
        if STATUS_SIGNAL.wait().await {
            match with_timeout(TIMEOUT, sensor.init()).await {
                Ok(r) if r != None => (),
                _ => {
                    error!("Sensor: unable to init when woke up")
                }
            }
            info!("RESUME SENSOR TASK");
            break;
        }
    }
}
