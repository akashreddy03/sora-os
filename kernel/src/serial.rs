use spin::{Mutex, Once};
use uart_16550::{Config, Uart16550Tty, backend::PioBackend};

static SERIAL: Once<Mutex<Uart16550Tty<PioBackend>>> = Once::new();

pub fn get_serial() -> &'static Mutex<Uart16550Tty<PioBackend>> {
    SERIAL
        .call_once(|| {
            Mutex::new(unsafe { 
                Uart16550Tty::new_port(0x3f8, Config::default())
                .expect("Serial couldn't be initialized")
            })
        })
}

#[macro_export]
macro_rules! serial_println {
    ($($arg:tt)*) => {
        let mut uart = serial::get_serial().lock();
        let _ = writeln!(uart, $($arg)*);
        drop(uart);
    };
}

#[macro_export]
macro_rules! serial_print {
   ($($arg:tt)*) => {
        use core::fmt::Write;
        let mut uart = serial::get_serial().lock();
        let _ = writeln!(uart, $($arg)*);
    } 
}