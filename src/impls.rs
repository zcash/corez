//! `Read` and `Write` implementations for core types in `no_std` environments.

use crate::{Error, ErrorKind, Read, Result, Write};

// ---------------------------------------------------------------------------
// Forwarding impls — &mut R / &mut W
// ---------------------------------------------------------------------------

impl<R: Read + ?Sized> Read for &mut R {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        (**self).read(buf)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        (**self).read_exact(buf)
    }
}

impl<W: Write + ?Sized> Write for &mut W {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        (**self).write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        (**self).flush()
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        (**self).write_all(buf)
    }

    fn write_fmt(&mut self, fmt: core::fmt::Arguments<'_>) -> Result<()> {
        (**self).write_fmt(fmt)
    }
}

// ---------------------------------------------------------------------------
// Forwarding impls — Box<R> / Box<W>
// ---------------------------------------------------------------------------

#[cfg(feature = "alloc")]
impl<R: Read + ?Sized> Read for alloc::boxed::Box<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        (**self).read(buf)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        (**self).read_exact(buf)
    }
}

#[cfg(feature = "alloc")]
impl<W: Write + ?Sized> Write for alloc::boxed::Box<W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        (**self).write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        (**self).flush()
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        (**self).write_all(buf)
    }

    fn write_fmt(&mut self, fmt: core::fmt::Arguments<'_>) -> Result<()> {
        (**self).write_fmt(fmt)
    }
}

// ---------------------------------------------------------------------------
// In-memory buffer — Read for &[u8]
// ---------------------------------------------------------------------------

impl Read for &[u8] {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let amt = core::cmp::min(buf.len(), self.len());
        let (to_copy, rest) = self.split_at(amt);
        // Optimize single-byte case to avoid slice copy overhead.
        if amt == 1 {
            buf[0] = to_copy[0];
        } else {
            buf[..amt].copy_from_slice(to_copy);
        }
        *self = rest;
        Ok(amt)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        if buf.len() > self.len() {
            return Err(Error::from(ErrorKind::UnexpectedEof));
        }
        let (to_copy, rest) = self.split_at(buf.len());
        buf.copy_from_slice(to_copy);
        *self = rest;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// In-memory buffer — Write for &mut [u8]
// ---------------------------------------------------------------------------

impl Write for &mut [u8] {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let amt = core::cmp::min(buf.len(), self.len());
        let (dest, rest) = core::mem::take(self).split_at_mut(amt);
        dest.copy_from_slice(&buf[..amt]);
        *self = rest;
        Ok(amt)
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        if buf.len() > self.len() {
            return Err(Error::from(ErrorKind::WriteZero));
        }
        let (dest, rest) = core::mem::take(self).split_at_mut(buf.len());
        dest.copy_from_slice(buf);
        *self = rest;
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// In-memory buffer — Write for Vec<u8>
// ---------------------------------------------------------------------------

#[cfg(feature = "alloc")]
impl Write for alloc::vec::Vec<u8> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        self.extend_from_slice(buf);
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    extern crate alloc;
    use alloc::vec;
    use alloc::vec::Vec;

    use crate::{Read, Write};

    #[test]
    fn read_slice_basic() {
        let data = [1u8, 2, 3, 4, 5];
        let mut reader: &[u8] = &data;
        let mut buf = [0u8; 3];
        assert_eq!(reader.read(&mut buf).unwrap(), 3);
        assert_eq!(buf, [1, 2, 3]);
        assert_eq!(reader.read(&mut buf).unwrap(), 2);
        assert_eq!(buf[..2], [4, 5]);
    }

    #[test]
    fn read_slice_exact() {
        let data = [10u8, 20, 30];
        let mut reader: &[u8] = &data;
        let mut buf = [0u8; 3];
        reader.read_exact(&mut buf).unwrap();
        assert_eq!(buf, [10, 20, 30]);
    }

    #[test]
    fn read_slice_exact_eof() {
        let data = [1u8, 2];
        let mut reader: &[u8] = &data;
        let mut buf = [0u8; 3];
        let err = reader.read_exact(&mut buf).unwrap_err();
        assert_eq!(err.kind(), crate::ErrorKind::UnexpectedEof);
    }

    #[test]
    fn read_slice_empty() {
        let mut reader: &[u8] = &[];
        let mut buf = [0u8; 1];
        assert_eq!(reader.read(&mut buf).unwrap(), 0);
    }

    #[test]
    fn write_mut_slice_basic() {
        let mut buf = [0u8; 5];
        let mut writer: &mut [u8] = &mut buf;
        assert_eq!(writer.write(&[1, 2, 3]).unwrap(), 3);
        assert_eq!(writer.write(&[4, 5, 6]).unwrap(), 2);
        assert_eq!(buf, [1, 2, 3, 4, 5]);
    }

    #[test]
    fn write_vec_basic() {
        let mut buf = Vec::new();
        buf.write_all(&[1, 2, 3]).unwrap();
        buf.write_all(&[4, 5]).unwrap();
        assert_eq!(buf, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn read_write_forwarding() {
        let data = [1u8, 2, 3];
        let mut reader: &[u8] = &data;
        let reader_ref: &mut &[u8] = &mut reader;
        let mut buf = [0u8; 3];
        reader_ref.read_exact(&mut buf).unwrap();
        assert_eq!(buf, [1, 2, 3]);
    }
}
