#![no_std]
#![no_main]
#![macro_use]

#[cfg(not(test))]
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
    /// * `size` - Specifies the size of the data buffer.
    /// * `data` - The data to be written to the page.
    fn program_page(&mut self, address: u32, size: u32, data: *const u8) -> Result<(), ErrorCode>;
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
    ($type:ty) => {
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
            match <$type as FlashAlgorithm>::program_page(this, addr, size, data) {
                Ok(()) => 0,
                Err(e) => e.get(),
            }
        }
    };
}
