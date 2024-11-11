#![no_std]

use arduino_hal::{
    delay_us,
    port::{
        mode::{Floating, Input, Output},
        Pin,
    },
};
use heapless::spsc::Queue;

pub struct SoftwareSerial {
    rx: Pin<Input<Floating>>,
    tx: Pin<Output>,
    rx_delay_centering: u32,
    rx_delay_intrabit: u32,
    rx_delay_stopbit: u32,
    tx_delay: u32,
    pub buffer: Queue<u8, 64>,
}

impl SoftwareSerial {
    pub fn new(rx: Pin<Input<Floating>>, mut tx: Pin<Output>, baud_rate: u32) -> Self {
        tx.set_high();
        SoftwareSerial {
            rx,
            tx,
            rx_delay_centering: 500_000 / baud_rate,
            rx_delay_intrabit: 1_000_000 / baud_rate,
            rx_delay_stopbit: 750_000 / baud_rate,
            tx_delay: 1_000_000 / baud_rate,
            buffer: Queue::new(),
        }
    }
    pub fn recv(&mut self) -> bool {
        if self.rx.is_high() {
            return false;
        }

        let mut byte = 0;
        delay_us(self.rx_delay_centering);
        for _ in 0..8 {
            delay_us(self.rx_delay_intrabit);
            byte >>= 1;
            if self.rx.is_high() {
                byte |= 0x80;
            }
        }
        delay_us(self.rx_delay_stopbit);
        self.buffer.enqueue(byte).is_ok()
    }
    pub fn read(&mut self) -> Option<u8> {
        self.buffer.dequeue()
    }
    pub fn flush(&mut self) {
        while self.buffer.dequeue().is_some() {}
    }
    pub fn bytes_to_read(&self) -> usize {
        self.buffer.len()
    }
    pub fn write(&mut self, mut byte: u8) {
        self.tx.set_low();
        delay_us(self.tx_delay);
        for _ in 0..8 {
            if byte & 1 == 1 {
                self.tx.set_high()
            } else {
                self.tx.set_low()
            }
            delay_us(self.tx_delay);
            byte >>= 1;
        }

        self.tx.set_high();
        delay_us(self.tx_delay);
    }
}
