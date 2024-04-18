use embassy_stm32::{
    peripherals::{DMA1_CH5, USART1},
    usart::UartRx,
};

use crate::{blink_task, fmt::info, sensor::sensor_task};

type MyUsart = USART1;
type MyRxDma = DMA1_CH5;

#[embassy_executor::task]
pub async fn task(mut rx: UartRx<'static, MyUsart, MyRxDma>) {
    loop {
        let mut buffer: [u8; 1] = [0];

        info!("UART TASK BLOCK TO READ");
        rx.read(&mut buffer).await.unwrap(); // blocks here
        info!("UART TASK READ");

        let running = match buffer[0] {
            b'1' => true,
            b'0' => false,
            _ => false,
        };

        info!("SIGNAL SENSOR TASK: running={}", running);
        sensor_task::STATUS_SIGNAL.signal(running);
        info!("SIGNAL BLINK TASK: running={}", running);
        blink_task::STATUS_SIGNAL.signal(running);
    }
}
