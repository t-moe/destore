/// NOR flash wrapper
use embedded_storage::nor_flash::{ErrorType, MultiwriteNorFlash, NorFlash, ReadNorFlash};
use embedded_storage_async::nor_flash::{MultiwriteNorFlash as AsyncMultiwriteNorFlash, NorFlash as AsyncNorFlash, ReadNorFlash as AsyncReadNorFlash};

pub struct BlockingAsync<T> {
    pub wrapped: T,
}

impl<T> BlockingAsync<T> {
    /// Create a new instance of a wrapper for a given peripheral.
    pub fn new(wrapped: T) -> Self {
        Self { wrapped }
    }
}

impl<T> ErrorType for BlockingAsync<T>
where
    T: ErrorType,
{
    type Error = T::Error;
}

impl<T> AsyncNorFlash for BlockingAsync<T>
where
    T: NorFlash,
{
    const WRITE_SIZE: usize = <T as NorFlash>::WRITE_SIZE;
    const ERASE_SIZE: usize = <T as NorFlash>::ERASE_SIZE;

    async fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        self.wrapped.erase(from, to)
    }

    async fn write(&mut self, offset: u32, data: &[u8]) -> Result<(), Self::Error> {
        self.wrapped.write(offset, data)
    }
}

impl<T> AsyncReadNorFlash for BlockingAsync<T>
where
    T: ReadNorFlash,
{
    const READ_SIZE: usize = <T as ReadNorFlash>::READ_SIZE;
    async fn read(&mut self, address: u32, data: &mut [u8]) -> Result<(), Self::Error> {
        self.wrapped.read(address, data)
    }

    fn capacity(&self) -> usize {
        self.wrapped.capacity()
    }
}

impl<T> AsyncMultiwriteNorFlash for BlockingAsync<T> where T: MultiwriteNorFlash {}
