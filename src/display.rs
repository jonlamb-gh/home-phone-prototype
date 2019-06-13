#![allow(deprecated)]

use crate::display_data::{DisplayData, Row as DisplayRow};
use rppal::gpio::{Gpio, Mode};
use rppal::i2c::{Error, I2c};
use std::thread;
use std::time::Duration;

const GPIO_LCD_SDA: u8 = 2;
const GPIO_LCD_SCL: u8 = 3;

// TODO - Error/Result

const SLAVE_ADDRESS: u16 = 0x68;

const COLS: usize = 20;

const LCD_ACTION_GO_BIT: u8 = 1 << 7;

const LCD_EMPTY: u8 = 0x20;

const LCD_WAIT_ROW_UPDATE: Duration = Duration::from_millis(5);

const LCD_END_PACKET_DELAY: Duration = Duration::from_millis(1);

// TODO - use a single Row type
#[derive(Debug, Copy, Clone)]
pub enum Row {
    R0,
    R1,
    R2,
    R3,
}

pub struct Display {
    i2c: I2c,
}

impl Display {
    pub fn new() -> Result<Self, Error> {
        let gpio = Gpio::new().unwrap();
        let _pin_lcd_sda = gpio.get(GPIO_LCD_SDA).unwrap().into_io(Mode::Alt0);
        let _pin_lcd_scl = gpio.get(GPIO_LCD_SCL).unwrap().into_io(Mode::Alt0);

        let mut i2c = I2c::new().unwrap();
        i2c.set_slave_address(SLAVE_ADDRESS).expect("Slave address");

        Ok(Display { i2c })
    }

    pub fn display(&mut self, data: &DisplayData) -> Result<(), Error> {
        self.set_row(Row::R0, data.row(DisplayRow::Zero))?;
        self.set_row(Row::R1, data.row(DisplayRow::One))?;
        self.set_row(Row::R2, data.row(DisplayRow::Two))?;
        self.set_row(Row::R3, data.row(DisplayRow::Three))?;
        Ok(())
    }

    // TODO - make this better
    pub fn set_row(&mut self, row: Row, chars: &str) -> Result<(), Error> {
        let mut data: [u8; COLS] = [LCD_EMPTY; COLS];
        for (idx, c) in chars.bytes().enumerate() {
            if idx < data.len() {
                data[idx] = c;
            }
        }

        for (idx, c) in data.iter().enumerate() {
            let offset = idx as u8;
            self.write(u8::from(Register::LcdDataStart) + offset, *c)?;
        }

        self.write(
            u8::from(Register::LcdAction),
            u8::from(row) | LCD_ACTION_GO_BIT,
        )?;

        thread::sleep(LCD_WAIT_ROW_UPDATE);

        Ok(())
    }

    fn write(&mut self, register: u8, data: u8) -> Result<(), Error> {
        self.i2c.block_write(register, &[data])?;
        thread::sleep(LCD_END_PACKET_DELAY);
        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
enum Register {
    LcdAction,
    LcdDataStart,
}

impl From<Register> for u8 {
    fn from(r: Register) -> u8 {
        match r {
            Register::LcdAction => 21,
            Register::LcdDataStart => 22,
        }
    }
}

impl From<Row> for u8 {
    fn from(r: Row) -> u8 {
        match r {
            Row::R0 => 0,
            Row::R1 => 1,
            Row::R2 => 2,
            Row::R3 => 3,
        }
    }
}
