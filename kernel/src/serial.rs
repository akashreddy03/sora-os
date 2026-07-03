use core::fmt::{Arguments, Write};
use spin::{Mutex, Once};
use uart_16550::{Config, Uart16550Tty, backend::PioBackend};

static SERIAL: Once<Mutex<Uart16550Tty<PioBackend>>> = Once::new();

pub fn get_serial() -> &'static Mutex<Uart16550Tty<PioBackend>> {
    SERIAL.call_once(|| {
        Mutex::new(unsafe {
            Uart16550Tty::new_port(0x3f8, Config::default())
                .expect("Serial couldn't be initialized")
        })
    })
}

pub fn _print(arg: Arguments) {
    let mut uart = get_serial().lock();
    let _ = uart.write_fmt(arg);
    drop(uart);
}
