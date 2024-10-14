// from standard library
extern crate alloc;
use crate::error::IpcError;
use crate::io::{BufRead, Read, Seek, SeekFrom, Write};
use alloc::boxed::Box;
use alloc::collections::vec_deque::VecDeque;
use alloc::vec::Vec;
use core::cmp;
use core::mem;

// =============================================================================
// Forwarding implementations

impl<R: Read + ?Sized> Read for &mut R {
    type Error = R::Error;
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        (**self).read(buf)
    }
}

impl<W: Write + ?Sized> Write for &mut W {
    type Error = W::Error;
    #[inline]
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        (**self).write(buf)
    }

    #[inline]
    fn flush(&mut self) -> Result<(), Self::Error> {
        (**self).flush()
    }
}

impl<S: Seek + ?Sized> Seek for &mut S {
    type Error = S::Error;
    #[inline]
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        (**self).seek(pos)
    }
}

impl<B: BufRead + ?Sized> BufRead for &mut B {
    type Error = B::Error;
    #[inline]
    fn fill_buf(&mut self) -> Result<&[u8], Self::Error> {
        (**self).fill_buf()
    }

    #[inline]
    fn consume(&mut self, amt: usize) {
        (**self).consume(amt)
    }
}

impl<R: Read + ?Sized> Read for Box<R> {
    type Error = R::Error;
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        (**self).read(buf)
    }
}

impl<W: Write + ?Sized> Write for Box<W> {
    type Error = W::Error;
    #[inline]
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        (**self).write(buf)
    }
    #[inline]
    fn flush(&mut self) -> Result<(), Self::Error> {
        (**self).flush()
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        (**self).write_all(buf)
    }
}

impl<S: Seek + ?Sized> Seek for Box<S> {
    type Error = S::Error;
    #[inline]
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        (**self).seek(pos)
    }

    #[inline]
    fn stream_position(&mut self) -> Result<u64, Self::Error> {
        (**self).stream_position()
    }
}

impl<B: BufRead + ?Sized> BufRead for Box<B> {
    type Error = B::Error;
    #[inline]
    fn fill_buf(&mut self) -> Result<&[u8], Self::Error> {
        (**self).fill_buf()
    }

    #[inline]
    fn consume(&mut self, amt: usize) {
        (**self).consume(amt)
    }
}

// =============================================================================
// In-memory buffer implementations

/// Read is implemented for `&[u8]` by copying from the slice.
///
/// Note that reading updates the slice to point to the yet unread part.
/// The slice will be empty when EOF is reached.
impl Read for &[u8] {
    type Error = IpcError;
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let amt = cmp::min(buf.len(), self.len());
        let (a, b) = self.split_at(amt);

        // First check if the amount of bytes we want to read is small:
        // `copy_from_slice` will generally expand to a call to `memcpy`, and
        // for a single byte the overhead is significant.
        if amt == 1 {
            buf[0] = a[0];
        } else {
            buf[..amt].copy_from_slice(a);
        }

        *self = b;
        Ok(amt)
    }
}

impl BufRead for &[u8] {
    type Error = IpcError;
    #[inline]
    fn fill_buf(&mut self) -> Result<&[u8], Self::Error> {
        Ok(*self)
    }

    #[inline]
    fn consume(&mut self, amt: usize) {
        *self = &self[amt..];
    }
}

/// Write is implemented for `&mut [u8]` by copying into the slice, overwriting
/// its data.
///
/// Note that writing updates the slice to point to the yet unwritten part.
/// The slice will be empty when it has been completely overwritten.
///
/// If the number of bytes to be written exceeds the size of the slice, write operations will
/// return short writes: ultimately, `Ok(0)`; in this situation, `write_all` returns an error of
/// kind `ErrorKind::WriteZero`.
impl Write for &mut [u8] {
    type Error = IpcError;
    #[inline]
    fn write(&mut self, data: &[u8]) -> Result<usize, Self::Error> {
        let amt = cmp::min(data.len(), self.len());
        let (a, b) = mem::take(self).split_at_mut(amt);
        a.copy_from_slice(&data[..amt]);
        *self = b;
        Ok(amt)
    }

    #[inline]
    fn write_all(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        if self.write(data)? == data.len() {
            Ok(())
        } else {
            Err(IpcError::SliceWriteError)
        }
    }

    #[inline]
    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// Write is implemented for `Vec<u8>` by appending to the vector.
/// The vector will grow as needed.
impl Write for Vec<u8> {
    type Error = IpcError;
    #[inline]
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.extend_from_slice(buf);
        Ok(buf.len())
    }
    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        self.extend_from_slice(buf);
        Ok(())
    }

    #[inline]
    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// Read is implemented for `VecDeque<u8>` by consuming bytes from the front of the `VecDeque`.
impl Read for VecDeque<u8> {
    type Error = IpcError;
    /// Fill `buf` with the contents of the "front" slice as returned by
    /// [`as_slices`][`VecDeque::as_slices`]. If the contained byte slices of the `VecDeque` are
    /// discontiguous, multiple calls to `read` will be needed to read the entire content.
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let (ref mut front, _) = self.as_slices();
        let n = Read::read(front, buf)?;
        self.drain(..n);
        Ok(n)
    }
}

/// BufRead is implemented for `VecDeque<u8>` by reading bytes from the front of the `VecDeque`.
impl BufRead for VecDeque<u8> {
    type Error = IpcError;
    /// Returns the contents of the "front" slice as returned by
    /// [`as_slices`][`VecDeque::as_slices`]. If the contained byte slices of the `VecDeque` are
    /// discontiguous, multiple calls to `fill_buf` will be needed to read the entire content.
    #[inline]
    fn fill_buf(&mut self) -> Result<&[u8], Self::Error> {
        let (front, _) = self.as_slices();
        Ok(front)
    }

    #[inline]
    fn consume(&mut self, amt: usize) {
        self.drain(..amt);
    }
}

/// Write is implemented for `VecDeque<u8>` by appending to the `VecDeque`, growing it as needed.
impl Write for VecDeque<u8> {
    type Error = IpcError;
    #[inline]
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.extend(buf);
        Ok(buf.len())
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        self.extend(buf);
        Ok(())
    }

    #[inline]
    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}
