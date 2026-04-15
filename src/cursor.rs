//! A `Cursor` wraps an in-memory buffer and provides `Read`/`Write` access
//! with a tracked position, mirroring [`std::io::Cursor`] for `no_std`
//! environments.

use core::cmp;

use crate::{Error, ErrorKind};
use crate::{Read, Result, Write};

/// A `Cursor` wraps an in-memory buffer and provides it with a [`Read`]
/// and/or [`Write`] implementation.
///
/// This is a `no_std` equivalent of [`std::io::Cursor`].
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Cursor<T> {
    inner: T,
    pos: u64,
}

impl<T> Cursor<T> {
    /// Creates a new cursor wrapping the provided underlying in-memory buffer.
    ///
    /// The initial position is `0` — reads and writes will start at the
    /// beginning of the buffer.
    pub fn new(inner: T) -> Self {
        Cursor { inner, pos: 0 }
    }

    /// Consumes the cursor, returning the underlying value.
    pub fn into_inner(self) -> T {
        self.inner
    }

    /// Returns a reference to the underlying value.
    pub fn get_ref(&self) -> &T {
        &self.inner
    }

    /// Returns a mutable reference to the underlying value.
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Returns the current position of the cursor.
    pub fn position(&self) -> u64 {
        self.pos
    }

    /// Sets the position of the cursor.
    pub fn set_position(&mut self, pos: u64) {
        self.pos = pos;
    }
}

// ---------------------------------------------------------------------------
// Read
// ---------------------------------------------------------------------------

impl<T: AsRef<[u8]>> Read for Cursor<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let slice = self.inner.as_ref();
        let start = cmp::min(self.pos, slice.len() as u64) as usize;
        let remaining = &slice[start..];
        let amt = cmp::min(buf.len(), remaining.len());
        buf[..amt].copy_from_slice(&remaining[..amt]);
        self.pos += amt as u64;
        Ok(amt)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        let slice = self.inner.as_ref();
        let start = cmp::min(self.pos, slice.len() as u64) as usize;
        let remaining = &slice[start..];
        if buf.len() > remaining.len() {
            return Err(Error::from(ErrorKind::UnexpectedEof));
        }
        buf.copy_from_slice(&remaining[..buf.len()]);
        self.pos += buf.len() as u64;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Write — &mut [u8]
// ---------------------------------------------------------------------------

fn slice_write(pos: &mut u64, slice: &mut [u8], buf: &[u8]) -> Result<usize> {
    let start = cmp::min(*pos, slice.len() as u64) as usize;
    let amt = cmp::min(buf.len(), slice.len() - start);
    slice[start..start + amt].copy_from_slice(&buf[..amt]);
    *pos += amt as u64;
    Ok(amt)
}

impl Write for Cursor<&mut [u8]> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        slice_write(&mut self.pos, self.inner, buf)
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Write — Vec<u8> and &mut Vec<u8>
// ---------------------------------------------------------------------------

#[cfg(feature = "alloc")]
fn vec_write(pos: &mut u64, vec: &mut alloc::vec::Vec<u8>, buf: &[u8]) -> Result<usize> {
    let start = *pos as usize;
    // If position is past the end, fill gap with zeros.
    if start > vec.len() {
        vec.resize(start, 0);
    }
    let overlap = if start < vec.len() {
        let end = cmp::min(start + buf.len(), vec.len());
        let n = end - start;
        vec[start..end].copy_from_slice(&buf[..n]);
        n
    } else {
        0
    };
    if overlap < buf.len() {
        vec.extend_from_slice(&buf[overlap..]);
    }
    *pos += buf.len() as u64;
    Ok(buf.len())
}

#[cfg(feature = "alloc")]
impl Write for Cursor<alloc::vec::Vec<u8>> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        vec_write(&mut self.pos, &mut self.inner, buf)
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

#[cfg(feature = "alloc")]
impl Write for Cursor<&mut alloc::vec::Vec<u8>> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        vec_write(&mut self.pos, self.inner, buf)
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

    use super::Cursor;
    use crate::{Read, Write};

    #[test]
    fn cursor_read_basic() {
        let data = vec![1u8, 2, 3, 4, 5];
        let mut cursor = Cursor::new(data.as_slice());
        let mut buf = [0u8; 3];
        cursor.read_exact(&mut buf).unwrap();
        assert_eq!(buf, [1, 2, 3]);
        assert_eq!(cursor.position(), 3);
    }

    #[test]
    fn cursor_read_eof() {
        let data = [1u8, 2];
        let mut cursor = Cursor::new(&data[..]);
        let mut buf = [0u8; 3];
        let err = cursor.read_exact(&mut buf).unwrap_err();
        assert_eq!(err.kind(), crate::ErrorKind::UnexpectedEof);
    }

    #[test]
    fn cursor_position_and_set_position() {
        let data = [10u8, 20, 30, 40];
        let mut cursor = Cursor::new(&data[..]);
        assert_eq!(cursor.position(), 0);
        let mut buf = [0u8; 2];
        cursor.read_exact(&mut buf).unwrap();
        assert_eq!(cursor.position(), 2);
        cursor.set_position(0);
        cursor.read_exact(&mut buf).unwrap();
        assert_eq!(buf, [10, 20]);
    }

    #[test]
    fn cursor_into_inner() {
        let data = vec![1u8, 2, 3];
        let cursor = Cursor::new(data.clone());
        assert_eq!(cursor.into_inner(), data);
    }

    #[test]
    fn cursor_write_vec() {
        let mut cursor = Cursor::new(Vec::new());
        cursor.write_all(&[1, 2, 3]).unwrap();
        cursor.write_all(&[4, 5]).unwrap();
        assert_eq!(cursor.into_inner(), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn cursor_write_vec_at_position() {
        let mut cursor = Cursor::new(vec![0u8; 5]);
        cursor.set_position(2);
        cursor.write_all(&[10, 20]).unwrap();
        assert_eq!(cursor.position(), 4);
        assert_eq!(cursor.get_ref(), &vec![0, 0, 10, 20, 0]);
    }

    #[test]
    fn cursor_write_vec_extends() {
        let mut cursor = Cursor::new(vec![1u8, 2]);
        cursor.set_position(1);
        cursor.write_all(&[10, 20, 30]).unwrap();
        assert_eq!(cursor.into_inner(), vec![1, 10, 20, 30]);
    }

    #[test]
    fn cursor_write_mut_slice() {
        let mut buf = [0u8; 5];
        let mut cursor = Cursor::new(&mut buf[..]);
        cursor.write_all(&[1, 2, 3]).unwrap();
        assert_eq!(cursor.position(), 3);
        drop(cursor);
        assert_eq!(buf, [1, 2, 3, 0, 0]);
    }

    #[test]
    fn cursor_write_ref_vec() {
        let mut v = Vec::new();
        let mut cursor = Cursor::new(&mut v);
        cursor.write_all(&[1, 2, 3]).unwrap();
        drop(cursor);
        assert_eq!(v, vec![1, 2, 3]);
    }
}
