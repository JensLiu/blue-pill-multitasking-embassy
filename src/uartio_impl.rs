use defmt::{error, info};
use embassy_stm32::{pac::usart::Uart, peripherals, usart::{self, UartRx, UartTx}};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, pipe::Pipe};

const READ_BUFFER_SIZE: usize = 64;
const AT_CMD: u8 = 0x0d;

pub struct UsartTask<'a> {
    uart: &'a mut Uart,

}

async fn read<'d, T, RxDma>(mut rx: usart::UartRx<'d, T, RxDma>)
where
    T: usart::BasicInstance,
    RxDma: usart::RxDma<T>
{
    let mut local_read_buf: [u8; READ_BUFFER_SIZE] = [0; READ_BUFFER_SIZE];
    if let Err(err) = rx.read(&mut local_read_buf).await {
        error!("Error when reading from UART: {}", err);
    };
    
    info!("buffer: {}", local_read_buf);

}


async fn write<'d, T, TxDma>(mut tx: usart::UartTx<'d, T, TxDma>)
where
    T: usart::BasicInstance,
    RxDma: usart::RxDma<T>
{
    
}
