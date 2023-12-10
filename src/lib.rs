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

    /// Verify the firmware that has been programmed.  Will only be called after [`FlashAlgorithm::new()`] with [`Function::Verify`].
    ///
    /// # Arguments
    ///
    /// * `address` - The start address of the flash to verify.
    /// * `size` - The length of the data to verify.
    /// * `data` - The data to compare with.
    #[cfg(feature = "verify")]
    fn verify(&mut self, address: u32, size: u32, data: Option<&[u8]>) -> Result<(), ErrorCode>;
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
        device_name: $device_name:expr,
        device_type: $device_type:expr,
        flash_address: $flash_address:expr,
        flash_size: $flash_size:expr,
        page_size: $page_size:expr,
        empty_value: $empty_value:expr,
        program_time_out: $program_time_out:expr,
        erase_time_out: $erase_time_out:expr,
        sectors: [$({
            size: $size:expr,
            address: $address:expr,
        }),+]
    }) => {
        static mut _IS_INIT: bool = false;
        static mut _ALGO_INSTANCE: core::mem::MaybeUninit<$type> = core::mem::MaybeUninit::uninit();

        core::arch::global_asm!(".section .PrgData, \"aw\"");

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
                _ => core::panic!("This branch can only be reached if the host library sent an unknown function code.")
            };
            match <$type as $crate::FlashAlgorithm>::new(addr, clock, function) {
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
            match <$type as $crate::FlashAlgorithm>::erase_sector(this, addr) {
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
            match <$type as $crate::FlashAlgorithm>::program_page(this, addr, data_slice) {
                Ok(()) => 0,
                Err(e) => e.get(),
            }
        }
        $crate::erase_chip!($type);
        $crate::verify!($type);

        #[allow(non_upper_case_globals)]
        #[no_mangle]
        #[used]
        #[link_section = "DeviceData"]
        pub static FlashDevice: FlashDeviceDescription = FlashDeviceDescription {
            // The version is never read by probe-rs and can be fixed.
            vers: 0x1,
            // The device name here can be customized but it really has no real use
            // appart from identifying the device the ELF is intended for which we have
            // in our YAML.
            dev_name: $crate::arrayify_string($device_name),
            // The specification does not specify the values that can go here,
            // but this value means internal flash device.
            dev_type: $device_type,
            dev_addr: $flash_address,
            device_size: $flash_size,
            page_size: $page_size,
            _reserved: 0,
            // The empty state of a byte in flash.
            empty: $empty_value,
            // This value can be used to estimate the amount of time the flashing procedure takes worst case.
            program_time_out: $program_time_out,
            // This value can be used to estimate the amount of time the erasing procedure takes worst case.
            erase_time_out: $erase_time_out,
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
            dev_type: DeviceType,
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

        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        #[repr(u16)]
        pub enum DeviceType {
            Unknown = 0,
            Onchip = 1,
            Ext8Bit = 2,
            Ext16Bit = 3,
            Ext32Bit = 4,
            ExtSpi = 5,
        }
    };
}

#[doc(hidden)]
#[macro_export]
#[cfg(not(feature = "erase-chip"))]
macro_rules! erase_chip {
    ($type:ty) => {};
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
            match <$type as $crate::FlashAlgorithm>::erase_all(this) {
                Ok(()) => 0,
                Err(e) => e.get(),
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
#[cfg(not(feature = "verify"))]
macro_rules! verify {
    ($type:ty) => {};
}
#[doc(hidden)]
#[macro_export]
#[cfg(feature = "verify")]
macro_rules! verify {
    ($type:ty) => {
        #[no_mangle]
        #[link_section = ".entry"]
        pub unsafe extern "C" fn Verify(addr: u32, size: u32, data: *const u8) -> u32 {
            if !_IS_INIT {
                return 1;
            }
            let this = &mut *_ALGO_INSTANCE.as_mut_ptr();

            if data.is_null() {
                match <$type as $crate::FlashAlgorithm>::verify(this, addr, size, None) {
                    Ok(()) => 0,
                    Err(e) => e.get(),
                }
            } else {
                let data_slice: &[u8] = unsafe { core::slice::from_raw_parts(data, size as usize) };
                match <$type as $crate::FlashAlgorithm>::verify(this, addr, size, Some(data_slice))
                {
                    Ok(()) => 0,
                    Err(e) => e.get(),
                }
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! count {
    () => (0usize);
    ( $x:tt $($xs:tt)* ) => (1usize + $crate::count!($($xs)*));
}

pub const fn arrayify_string<const N: usize>(msg: &'static str) -> [u8; N] {
    let mut arr = [0u8; N];
    let mut idx = 0;
    let msg_bytes = msg.as_bytes();

    while (idx < msg_bytes.len()) && (idx < N) {
        arr[idx] = msg_bytes[idx];
        idx += 1;
    }

    arr
}
