use embassy_stm32::{dma::NoDma, i2c};
use embassy_time::{with_timeout, Duration, Timer};

use crate::fmt::{debug, error};

// [address:read/write-bit]
pub const DHT20_ADDR: u8 = 0x38; // the I2C API shifits it by 1 bit, to 0x70

pub struct Dht20<'d, T, TXDMA = NoDma, RXDMA = NoDma>
where
    T: i2c::Instance,
{
    i2c: i2c::I2c<'d, T, TXDMA, RXDMA>,
    req_timeout: Duration,
}

impl<'d, T, TXDMA, RXDMA> Dht20<'d, T, TXDMA, RXDMA>
where
    T: i2c::Instance,
    RXDMA: i2c::RxDma<T>,
    TXDMA: i2c::TxDma<T>,
{
    pub async fn read(&mut self) -> Option<(f32, f32)> {
        // the "messure" command
        let mut send_buffer: [u8; 3] = [0xAC, 0x33, 0x00];
        debug!("SENSOR SENDS MEASSURE COMMAND");

        match with_timeout(
            self.req_timeout,
            self.i2c.write(DHT20_ADDR, &mut send_buffer),
        )
        .await
        {
            Ok(r) => match r {
                Ok(_) => (),
                Err(err) => {
                    error!("Sensor: error when reading result, {}", err);
                    None?;
                }
            },
            Err(_) => {
                error!("Sensor: timeout when reading result");
                None?;
            }
        }

        // wait for the messurement to finish
        Timer::after(Duration::from_millis(80)).await;

        // read the result
        let mut read_buffer: [u8; 6] = [0; 6];
        debug!("SENSOR START READ");

        match with_timeout(
            self.req_timeout,
            self.i2c.read(DHT20_ADDR, &mut read_buffer),
        )
        .await
        {
            Ok(r) => match r {
                Ok(_) => (),
                Err(err) => {
                    error!("Sensor: error when reading result, {}", err)
                }
            },
            Err(_) => {
                error!("Sensor: timeout when reading result...");
                None?;
            }
        }

        Self::valid_resault(&read_buffer).unwrap();

        Some((
            Self::parse_humidity(&read_buffer),
            Self::parse_temperature(&read_buffer),
        ))
    }
}

impl<'d, T, TXDMA, RXDMA> Dht20<'d, T, TXDMA, RXDMA>
where
    T: i2c::Instance,
    TXDMA: i2c::TxDma<T>,
{
    pub async fn init(&mut self) -> Option<()> {
        // wait for >= 100 ms
        Timer::after(Duration::from_millis(100)).await;

        // send initialisation command
        let mut send_buffer: [u8; 3] = [0xE1, 0x08, 0x00];

        // self.i2c.write(DHT20_ADDR, &mut send_buffer).await;
        match with_timeout(
            self.req_timeout,
            self.i2c.write(DHT20_ADDR, &mut send_buffer),
        )
        .await
        {
            Ok(r) => match r {
                Ok(_) => (),
                Err(err) => {
                    error!("Sensor: error when initialising, {}", err);
                    None?;
                }
            },
            Err(_) => {
                error!("Sensor: timeout when initialising");
                None?;
            }
        }

        Timer::after(Duration::from_millis(100)).await;

        Some(())
    }

    pub async fn _reset(&mut self) -> Option<()> {
        let send_buffer: [u8; 1] = [0xBA];
        match self.i2c.write(DHT20_ADDR, &send_buffer).await {
            Ok(_) => Some(()),
            Err(_) => None,
        }
    }
}

impl<'d, T, TXDMA, RXDMA> Dht20<'d, T, TXDMA, RXDMA>
where
    T: i2c::Instance,
{
    pub fn new(i2c: i2c::I2c<'d, T, TXDMA, RXDMA>, retry_duration: Duration) -> Self {
        Self {
            i2c,
            req_timeout: retry_duration,
        }
    }

    // helper functions

    fn valid_resault(buffer: &[u8]) -> Option<()> {
        assert!(buffer.len() >= 1);
        if buffer[0] & 0x68 == 0x08 {
            Some(())
        } else {
            None
        }
    }

    fn parse_humidity(buffer: &[u8]) -> f32 {
        assert!(buffer.len() == 6);
        let mut hum = buffer[1] as u32;
        hum = (hum << 8) | buffer[2] as u32;
        hum = (hum << 8) | buffer[3] as u32;
        hum >>= 4;
        let hum = hum as f32;
        (hum / (1 << 20) as f32) * 100f32
    }

    fn parse_temperature(buffer: &[u8]) -> f32 {
        assert!(buffer.len() == 6);
        let mut temp: u32 = (buffer[3] & 0x0f) as u32;
        temp = (temp << 8) | (buffer[4] as u32);
        temp = (temp << 8) | buffer[5] as u32;
        let temp = temp as f32;
        (temp / ((1 << 20) as f32)) * 200f32 - 50f32
    }
}
