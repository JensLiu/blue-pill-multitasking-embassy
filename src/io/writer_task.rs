use embassy_stm32::{
    peripherals::{DMA1_CH4, USART1},
    usart::UartTx,
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, pipe::Pipe};

pub const BUFFER_SIZE: usize = 64;
pub static DATAPIPE: Pipe<CriticalSectionRawMutex, BUFFER_SIZE> = Pipe::new();

type MyUsart = USART1;
type MyTxDma = DMA1_CH4;

#[embassy_executor::task]
pub async fn task(mut tx: UartTx<'static, MyUsart, MyTxDma>) {
    let mut read_buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
    loop {
        let len = DATAPIPE.read(&mut read_buffer).await;
        if len < 40 {   // skip corrupted data
            // error!("corrupted");
            continue;
        }
        if len < BUFFER_SIZE - 1 {
            read_buffer[len] = '\0' as u8;
            tx.write(&read_buffer[..len + 1]).await.unwrap();
        } else {
            tx.write(&read_buffer).await.unwrap();
        };
    }
}
