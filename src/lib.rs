//! Implement a [CMSIS-Pack] flash algorithm in Rust
//!
//! [CMSIS-Pack]: https://open-cmsis-pack.github.io/Open-CMSIS-Pack-Spec/main/html/flashAlgorithm.html
//!
//! # Feature flags
//!
//! - `panic-handler` this is enabled by default and includes a simple abort-on-panic
//!   panic handler. Disable this feature flag if you would prefer to use a different
//!   handler.

#![no_std]
#![no_main]
#![macro_use]

#[cfg(all(not(test), feature = "panic-handler"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe {
        core::arch::asm!("udf #0");
        core::hint::unreachable_unchecked();
    }
}

pub const FUNCTION_ERASE: u32 = 1;
pub const FUNCTION_PROGRAM: u32 = 2;
pub const FUNCTION_VERIFY: u32 = 3;

pub type ErrorCode = core::num::NonZeroU32;

pub trait FlashAlgorithm: Sized + 'static {
    /// Initialize the flash algorithm.
    ///
    /// It can happen that the flash algorithm does not need any specific initialization
    /// for the function to be executed or no initialization at all. It is up to the implementor
    /// to decide this.
    ///
    /// # Arguments
    ///
    /// * `address` - The start address of the flash region to program.
    /// * `clock` - The clock speed in Hertz for programming the device.
    /// * `function` - The function for which this initialization is for.
    fn new(address: u32, clock: u32, function: Function) -> Result<Self, ErrorCode>;

    /// Erase entire chip. Will only be called after [`FlashAlgorithm::new()`] with [`Function::Erase`].
    #[cfg(feature = "erase-chip")]
    fn erase_all(&mut self) -> Result<(), ErrorCode>;

    /// Erase sector. Will only be called after [`FlashAlgorithm::new()`] with [`Function::Erase`].
    ///
    /// # Arguments
    ///
    /// * `address` - The start address of the flash sector to erase.
    fn erase_sector(&mut self, address: u32) -> Result<(), ErrorCode>;

    /// Program bytes. Will only be called after [`FlashAlgorithm::new()`] with [`Function::Program`].
    ///
    /// # Arguments
    ///
    /// * `address` - The start address of the flash page to program.
    /// * `data` - The data to be written to the page.
    fn program_page(&mut self, address: u32, data: &[u8]) -> Result<(), ErrorCode>;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Function {
    Erase = 1,
    Program = 2,
    Verify = 3,
}

/// A macro to define a new flash algoritm.
///
/// It takes care of placing the functions in the correct linker sections
/// and checking the flash algorithm initialization status.
#[macro_export]
macro_rules! algorithm {
    ($type:ty, {
        flash_address: $flash_address:expr,
        flash_size: $flash_size:expr,
        page_size: $page_size:expr,
        empty_value: $empty_value:expr,
        sectors: [$({
            size: $size:expr,
            address: $address:expr,
        }),+]
    }) => {
        static mut _IS_INIT: bool = false;
        static mut _ALGO_INSTANCE: MaybeUninit<$type> = MaybeUninit::uninit();

        #[no_mangle]
        #[link_section = ".entry"]
        pub unsafe extern "C" fn Init(addr: u32, clock: u32, function: u32) -> u32 {
            if _IS_INIT {
                UnInit();
            }
            _IS_INIT = true;
            let function = match function {
                1 => $crate::Function::Erase,
                2 => $crate::Function::Program,
                3 => $crate::Function::Verify,
                _ => panic!("This branch can only be reached if the host library sent an unknown function code.")
            };
            match <$type as FlashAlgorithm>::new(addr, clock, function) {
                Ok(inst) => {
                    _ALGO_INSTANCE.as_mut_ptr().write(inst);
                    _IS_INIT = true;
                    0
                }
                Err(e) => e.get(),
            }
        }
        #[no_mangle]
        #[link_section = ".entry"]
        pub unsafe extern "C" fn UnInit() -> u32 {
            if !_IS_INIT {
                return 1;
            }
            _ALGO_INSTANCE.as_mut_ptr().drop_in_place();
            _IS_INIT = false;
            0
        }
        #[no_mangle]
        #[link_section = ".entry"]
        pub unsafe extern "C" fn EraseSector(addr: u32) -> u32 {
            if !_IS_INIT {
                return 1;
            }
            let this = &mut *_ALGO_INSTANCE.as_mut_ptr();
            match <$type as FlashAlgorithm>::erase_sector(this, addr) {
                Ok(()) => 0,
                Err(e) => e.get(),
            }
        }
        #[no_mangle]
        #[link_section = ".entry"]
        pub unsafe extern "C" fn ProgramPage(addr: u32, size: u32, data: *const u8) -> u32 {
            if !_IS_INIT {
                return 1;
            }
            let this = &mut *_ALGO_INSTANCE.as_mut_ptr();
            let data_slice: &[u8] = unsafe { core::slice::from_raw_parts(data, size as usize) };
            match <$type as FlashAlgorithm>::program_page(this, addr, data_slice) {
                Ok(()) => 0,
                Err(e) => e.get(),
            }
        }
        $crate::erase_chip!($type);

        #[allow(non_upper_case_globals)]
        #[no_mangle]
        #[used]
        #[link_section = "DeviceData"]
        pub static FlashDevice: FlashDeviceDescription = FlashDeviceDescription {
            // The version is never read by probe-rs and can be fixed.
            vers: 0x0,
            // The device name here can be customized but it really has no real use
            // appart from identifying the device the ELF is intended for which we have
            // in our YAML.
            dev_name: [0u8; 128],
            // The specification does not specify the values that can go here,
            // but this value means internal flash device.
            dev_type: 5,
            dev_addr: $flash_address,
            device_size: $flash_size,
            page_size: $page_size,
            _reserved: 0,
            // The empty state of a byte in flash.
            empty: $empty_value,
            // This value can be used to estimate the amount of time the flashing procedure takes worst case.
            program_time_out: 1000,
            // This value can be used to estimate the amount of time the erasing procedure takes worst case.
            erase_time_out: 2000,
            flash_sectors: [
                $(
                    FlashSector {
                        size: $size,
                        address: $address,
                    }
                ),+,
                // This marks the end of the flash sector list.
                FlashSector {
                    size: 0xffff_ffff,
                    address: 0xffff_ffff,
                }
            ],
        };

        #[repr(C)]
        pub struct FlashDeviceDescription {
            vers: u16,
            dev_name: [u8; 128],
            dev_type: u16,
            dev_addr: u32,
            device_size: u32,
            page_size: u32,
            _reserved: u32,
            empty: u8,
            program_time_out: u32,
            erase_time_out: u32,

            flash_sectors: [FlashSector; $crate::count!($($size)*) + 1],
        }

        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct FlashSector {
            size: u32,
            address: u32,
        }
    };
}

#[doc(hidden)]
#[macro_export]
#[cfg(not(feature = "erase-chip"))]
macro_rules! erase_chip {
    ($type:ty) => {}
}
#[doc(hidden)]
#[macro_export]
#[cfg(feature = "erase-chip")]
macro_rules! erase_chip {
    ($type:ty) => {
        #[no_mangle]
        #[link_section = ".entry"]
        pub unsafe extern "C" fn EraseChip() -> u32 {
            if !_IS_INIT {
                return 1;
            }
            let this = &mut *_ALGO_INSTANCE.as_mut_ptr();
            match <$type as FlashAlgorithm>::erase_all(this) {
                Ok(()) => 0,
                Err(e) => e.get(),
            }
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! count {
    () => (0usize);
    ( $x:tt $($xs:tt)* ) => (1usize + count!($($xs)*));
}
