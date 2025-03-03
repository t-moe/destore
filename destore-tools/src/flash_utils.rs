use embedded_storage_async::nor_flash::{
    ErrorType, NorFlash, NorFlashError, NorFlashErrorKind, ReadNorFlash,
};
use log::debug;
use std::fmt;
use std::fmt::{Display, Formatter};

pub struct FlashVec<'a>(pub &'a mut [u8]);

#[derive(Debug)]
pub struct FlashVecError;
impl NorFlashError for FlashVecError {
    fn kind(&self) -> NorFlashErrorKind {
        NorFlashErrorKind::OutOfBounds
    }
}
impl Display for FlashVecError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "OutOfBounds")
    }
}
impl std::error::Error for FlashVecError {}

impl<'a> ErrorType for FlashVec<'a> {
    type Error = FlashVecError;
}

impl<'a> ReadNorFlash for FlashVec<'a> {
    const READ_SIZE: usize = 4;

    async fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        if offset as usize + bytes.len() > self.0.len() {
            return Err(FlashVecError);
        }
        bytes.copy_from_slice(&self.0[offset as usize..offset as usize + bytes.len()]);
        debug!("read at {} len {}: {:?}", offset, bytes.len(), bytes);
        Ok(())
    }

    fn capacity(&self) -> usize {
        self.0.len()
    }
}

impl<'a> NorFlash for FlashVec<'a> {
    const WRITE_SIZE: usize = 4;

    const ERASE_SIZE: usize = 4096;

    async fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        panic!("tried to erase from {} to {}", from, to);
        /*self.0[from as usize..to as usize].fill(0xFF);
        Ok(())*/
    }

    async fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        panic!("tried to write at {} len {}", offset, bytes.len());
        /*if offset + bytes.len() as u32 > self.0.len() as u32 {
            return Err(FlashVecError);
        }
        self.0[offset as usize..offset as usize + bytes.len()].copy_from_slice(bytes);
        Ok(())*/
    }
}
