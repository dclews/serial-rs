#![no_std]

extern crate pio;
use self::pio::Port as IoPort;
use core::fmt;

pub enum KnownPorts {
    COM4 = 0x2e8,
    COM2 = 0x2f8,
    COM3 = 0x3e8,
    COM1 = 0x3f8,
}

#[repr(C)]
pub enum DivisorSpeed {
    BAUD115200 = 1,
    BAUD57600 = 2,
    BAUD38400 = 3,
}

#[repr(C)]
pub enum Parity {
    None = 0x0,
    Odd = 0x4,
    Even = 0x12,
    Mark = 0x14,
    Space = 0x1C,
}

#[repr(C)]
enum Interrupt {
    DataAvailable = 0x0,
    TransmitterEmpty = 0x2,
    BreakError = 0x4,
    StatusChange = 0x8,
}

#[repr(C)]
enum Registers {
    Data_DLAB_LSB = 0,
    Interrupt_DLAB_MSB,
    InterruptIdent_FifoControl,
    LineControl,
    ModemControl,
    LineStatus,
    ModemStatus,
    Scratch,
}
pub struct Port {
    data_dlab_lsb: IoPort,
    interrupt_dlab_msb: IoPort,
    interrupt_ident_fifo: IoPort,
    line_control: IoPort,
    modem_control: IoPort,
    line_status: IoPort,
    modem_status: IoPort,
    scratch: IoPort,
}

impl Port {
    pub fn new(io_base: u16) -> Port {
        Port{
            data_dlab_lsb: IoPort::new(io_base),
            interrupt_dlab_msb: IoPort::new(io_base + (Registers::Interrupt_DLAB_MSB as u16)),
            interrupt_ident_fifo: IoPort::new(io_base + (Registers::InterruptIdent_FifoControl as u16)),
            line_control: IoPort::new(io_base + (Registers::LineControl as u16)),
            modem_control: IoPort::new(io_base + (Registers::ModemControl as u16)),
            line_status: IoPort::new(io_base + (Registers::LineStatus as u16)),
            modem_status: IoPort::new(io_base + (Registers::ModemStatus as u16)),
            scratch: IoPort::new(io_base + (Registers::Scratch as u16)),
        }
    }
    pub unsafe fn init(&mut self) {
        self.set_interrupt_mask(0x0);
        self.set_divisor_speed(DivisorSpeed::BAUD38400);
        self.set_line_options(7, true, Parity::None);
        //self.set_interrupt_mask(...);
        self.set_fifo_options(0xC7); // Straight from osdev wiki. Supposedly enables FIFOs and clears them.
    }
    pub unsafe fn set_fifo_options(&mut self, options: u8) {
        self.interrupt_ident_fifo.write_u8(options);
    }
    pub unsafe fn set_interrupt_mask(&mut self, mask: u8) {
        self.interrupt_dlab_msb.write_u8(mask);
    }
    pub unsafe fn set_divisor_speed(&mut self, div: DivisorSpeed) {
        let div = div as u16;
        let dlab = self.line_control.read_u8();
        self.line_control.write_u8(dlab & 0x80); //Enable divisor access.
        self.data_dlab_lsb.write_u8(div as u8);
        self.interrupt_dlab_msb.write_u8((div >> 8) as u8);
        self.line_control.write_u8(dlab & 0x7F); //Disable divisor access.
    }
    pub unsafe fn set_line_options(&mut self, data_bits: u8, stop_bits: bool, parity: Parity) {
        self.line_control.write_u8(data_bits | ((stop_bits as u8) << 1) | (parity as u8));
    }
    pub unsafe fn packet_recieved(&mut self) -> bool {
        (self.line_status.read_u8() & 1) != 0
    }
    pub unsafe fn transmit_empty(&mut self) -> bool {
        (self.line_status.read_u8() & 0x20) != 0
    }
    pub unsafe fn write_str(&mut self, s: &str) {
        for c in s.chars() {
            self.write_char(c);
        }
    }
    pub unsafe fn write_char(&mut self, c: char) {
        while !self.transmit_empty() { }
        self.data_dlab_lsb.write_u8(c as u8);
    }
}

impl fmt::Write for Port {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        unsafe {self.write_str(s); }
        Ok(())
    }
}
