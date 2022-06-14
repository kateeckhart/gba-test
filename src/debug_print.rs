use crate::once::Lazy;
use core::cell::Cell;
use core::fmt;
use core::ptr;

pub struct DebugPrinterData {
    supported: bool,
    level: Cell<u8>,
    message_ptr: Cell<*mut u8>,
}

//FIXME make it irq safe
unsafe impl Sync for DebugPrinterData {}

impl DebugPrinterData {
    fn write_byte(&self, byte: u8) {
        if !self.supported {
            return;
        }
        let mut m_ptr = self.message_ptr.get();
        if m_ptr as usize >= 0x4FFF6FF && byte != b'\n' {
            self.write_byte(b'\n');
            m_ptr = self.message_ptr.get()
        }

        if byte == b'\n' {
            unsafe {
                ptr::write_volatile(0x4FFF700 as *mut u16, 0x100 | u16::from(self.level.get()))
            }
            m_ptr = 0x4FFF600 as *mut u8;
        } else {
            unsafe {
                ptr::write_volatile(m_ptr, byte);
                m_ptr = m_ptr.offset(1);
            }
        }

        self.message_ptr.set(m_ptr);
    }
}

pub static DEBUG_PRINTER: Lazy<DebugPrinterData> = Lazy::new(|| {
    unsafe {
        ptr::write_volatile(0x4FFF780 as *mut u16, 0xC0DE);
    }
    let raw_supported = unsafe { ptr::read_volatile(0x4FFF780 as *const u16) };
    let supported = raw_supported == 0x1DEA;
    DebugPrinterData {
        supported,
        level: Cell::new(0),
        message_ptr: Cell::new(0x4FFF600 as *mut u8),
    }
});

pub struct DebugPrinter(pub u8);

impl fmt::Write for DebugPrinter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        DEBUG_PRINTER.level.set(self.0);
        let bytes = s.as_bytes();
        for byte in bytes {
            DEBUG_PRINTER.write_byte(*byte)
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        use ::core::fmt::Write;
        let _ = ::core::write!($crate::debug_print::DebugPrinter(3), $($arg)*,); // Can't fail
    }}
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {{
        use ::core::fmt::Write;
        let _ = ::core::writeln!($crate::debug_print::DebugPrinter(3), $($arg)*,); // Can't fail
    }}
}
