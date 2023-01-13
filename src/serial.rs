

use core::arch::asm;
use core::fmt::{self, Write};
use spin::Mutex;
use lazy_static::lazy_static;


// from https://en.wikibooks.org/wiki/Serial_Programming/8250_UART_Programming#Serial_COM_Port_Memory_and_I/O_Allocation
/* 

Address	DLAB	I/O Access	Abbrv.	Register Name
+0	    0	Write	    THR	    Transmitter Holding Buffer
+0	    0	Read	    RBR	    Receiver Buffer
+0	    1	Read/Write	DLL	    Divisor Latch Low Byte
+1	    0	Read/Write	IER	    Interrupt Enable Register
+1	    1	Read/Write	DLH	    Divisor Latch High Byte
+2	    x	Read	    IIR	    Interrupt Identification Register
+2	    x	Write	    FCR	    FIFO Control Register
+3	    x	Read/Write	LCR	    Line Control Register
+4	    x	Read/Write	MCR	    Modem Control Register
+5	    x	Read	    LSR	    Line Status Register
+6	    x	Read	    MSR	    Modem Status Register
+7	    x	Read/Write	SR	    Scratch Register
*/


const THR: u8 = 0;
const RBR: u8 = 0; 
const DLL: u8 = 0; 
const IER: u8 = 1;
const DLH: u8 = 1; 
const LCR: u8 = 3;
const MCR: u8 = 4;
const LSR: u8 = 5;

const DR    :u8 = 1; //	Data Ready
const ETHR  :u8 = 1 << 5; //Empty Transmitter Holding Register

pub const COM1: u16 = 0x3F8;

const MAX_BAUD_RATE : u32 = 115_200;

pub struct SerialPort {
    base: u16
}

impl SerialPort {

    pub fn new(base: u16, baud_rate: u32) -> Self {
        let this = Self { base };

        let dlv = MAX_BAUD_RATE / baud_rate;
        let dll = dlv & 0xff;
        let dlh = dlv >> 8;


        // disable interrupts
        this._write(IER, 0);  

        // enable DLAB inorder to set the speed
        this._write(LCR, 0x80);

        // set baud rate
        this._write(DLL, dll as u8);
        this._write(DLH, dlh as u8);

        // disable DLAB and set the data: no parity, one stop bit, 8bit data
        this._write(LCR, 3);

        // enable auxilary output 2, RTS (Request to Send) and DTR (Data terminal Ready) 
        this._write(MCR, 0b1011);

        // enable interrupts
        this._write(IER, 1); 
        
        this
    }
    
    #[inline]
    fn _write(&self, offset: u8, value: u8) { 
        unsafe {
            asm!("out dx, al", in("al") value, in("dx") (self.base + offset as u16), options(nomem, nostack, preserves_flags));
        }
    }
    
    #[inline]
    fn _read(&self, offset: u8) -> u8 { 
        let value: u8;
        unsafe {
            asm!("in al, dx", out("al") value, in("dx") (self.base + offset as u16), options(nomem, nostack, preserves_flags));
        }
        value   
    }


    fn write(&self, byte: u8) {
        while (self._read(LSR) &  ETHR) == 0 {
            core::hint::spin_loop();
        }
        self._write(THR, byte);
    
    }

    #[allow(dead_code)]
    fn read(&self) -> u8{

        while (self._read(LSR) &  DR) == 0{
            core::hint::spin_loop();
        }
        self._read(RBR)
    }


}


impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.write(byte);
        }
        Ok(())
    }
}


lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let serial_port = SerialPort::new(COM1, MAX_BAUD_RATE);
        Mutex::new(serial_port)
    };
}

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    SERIAL1.lock().write_fmt(args).expect("Printing to serial failed");
}



/// Prints to the host through the serial interface.
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

/// Prints to the host through the serial interface, appending a newline.
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(concat!($fmt, "\n"), $($arg)*));
}