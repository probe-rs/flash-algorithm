#![no_std]
#![no_main]

use flash_algorithm::FlashAlgorithm;

struct Algorithm;

flash_algorithm::algorithm!(Algorithm, {
    device_name: "test",
    device_type: DeviceType::Onchip,
    flash_address: 0x0,
    flash_size: 0x0,
    page_size: 0x0,
    empty_value: 0xFF,
    program_time_out: 1000,
    erase_time_out: 2000,
    sectors: [{
        size: 0x0,
        address: 0x0,
    }]
});

impl FlashAlgorithm for Algorithm {
    fn new(
        _address: u32,
        _clock: u32,
        _function: flash_algorithm::Function,
    ) -> Result<Self, flash_algorithm::ErrorCode> {
        todo!()
    }

    fn erase_all(&mut self) -> Result<(), flash_algorithm::ErrorCode> {
        todo!()
    }

    fn erase_sector(&mut self, _address: u32) -> Result<(), flash_algorithm::ErrorCode> {
        todo!()
    }

    fn program_page(
        &mut self,
        _address: u32,
        _data: &[u8],
    ) -> Result<(), flash_algorithm::ErrorCode> {
        todo!()
    }

    fn verify(
        &mut self,
        _address: u32,
        _size: u32,
        _data: Option<&[u8]>,
    ) -> Result<(), flash_algorithm::ErrorCode> {
        todo!()
    }
}
