use core::fmt::{Arguments, Write};
use spin::{LazyLock, Mutex};
use uart_16550::{Config, Uart16550Tty, backend::PioBackend};

static SERIAL: LazyLock<Mutex<Uart16550Tty<PioBackend>>> = LazyLock::new(|| {
        Mutex::new(unsafe {
            Uart16550Tty::new_port(0x3f8, Config::default())
                .expect("Serial couldn't be initialized")
        })
    }
);

pub fn _print(arg: Arguments) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let mut uart = SERIAL.lock();
        uart.write_fmt(arg).expect("Printing to serial failed");
    });
}
