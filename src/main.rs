#![no_std]
#![no_main]

mod fmt;

#[cfg(not(feature = "defmt"))]
use panic_halt as _;

#[cfg(feature = "defmt")]
use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;
use embassy_stm32::{bind_interrupts, i2c, peripherals, time, usart};

mod blink_task;
mod io;
mod sensor;

use io::{reader_task, writer_task};
use sensor::sensor_task;

// uart interrupts
bind_interrupts!(struct UartIrqs {
    USART1 => usart::InterruptHandler<peripherals::USART1>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // initialising peripherals
    let p = embassy_stm32::init(Default::default());

    // spawn the sensor task
    let i2c = i2c::I2c::new(
        p.I2C1,               // periphral: an I2C peripheral
        p.PB6,                // scl: a GPIO pin
        p.PB7,                // sda: a GPIO pin
        sensor_task::I2cIrqs, // irq: handle to an interrupt
        p.DMA1_CH6,           // tx_dma
        p.DMA1_CH7,           // rx_dma
        time::hz(100_000),    // frequency
        Default::default(),   // config
    );
    let sensor_task_token = sensor_task::task(i2c);

    // spawn the blink task
    let led_pin = p.PB13;
    let blink_task_token = blink_task::task(led_pin);

    // spawn the writer and reader task
    let usart = usart::Uart::new(
        p.USART1,   // uart peripheral
        p.PA10,     // rx
        p.PA9,      // tx
        UartIrqs,   // irqs
        p.DMA1_CH4, // tx dma
        p.DMA1_CH5, // rx dma
        Default::default(),
    )
    .unwrap();

    let (tx, rx) = usart.split();

    let reader_task_token = reader_task::task(rx);
    let writer_task_token = writer_task::task(tx);

    spawner.spawn(sensor_task_token).unwrap();
    spawner.spawn(blink_task_token).unwrap();
    spawner.spawn(writer_task_token).unwrap();
    spawner.spawn(reader_task_token).unwrap();
}
