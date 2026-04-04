use crate::{Result, TsgoError};
use corsa_core::fast::compact_format;
use std::io::{Read, Write};

pub(crate) const MSG_REQUEST: u8 = 1;
pub(crate) const MSG_CALL_RESPONSE: u8 = 2;
pub(crate) const MSG_CALL_ERROR: u8 = 3;
pub(crate) const MSG_RESPONSE: u8 = 4;
pub(crate) const MSG_ERROR: u8 = 5;
pub(crate) const MSG_CALL: u8 = 6;

#[derive(Debug)]
pub(crate) struct MsgpackTuple {
    pub kind: u8,
    pub method: Vec<u8>,
    pub payload: Vec<u8>,
}

pub(crate) fn read_tuple<R: Read>(reader: &mut R) -> Result<MsgpackTuple> {
    let mut tag = [0_u8; 1];
    reader.read_exact(&mut tag)?;
    if tag[0] != 0x93 {
        return Err(TsgoError::Protocol(compact_format(format_args!(
            "expected tuple marker, got {:x}",
            tag[0]
        ))));
    }
    let kind = read_int(reader)?;
    let method = read_bin(reader)?;
    let payload = read_bin(reader)?;
    Ok(MsgpackTuple {
        kind,
        method,
        payload,
    })
}

pub(crate) fn write_tuple<W: Write>(
    writer: &mut W,
    kind: u8,
    method: &[u8],
    payload: &[u8],
) -> Result<()> {
    writer.write_all(&[0x93, kind])?;
    write_bin(writer, method)?;
    write_bin(writer, payload)?;
    writer.flush()?;
    Ok(())
}

fn read_int<R: Read>(reader: &mut R) -> Result<u8> {
    let mut buf = [0_u8; 1];
    reader.read_exact(&mut buf)?;
    if buf[0] <= 0x7f {
        return Ok(buf[0]);
    }
    if buf[0] != 0xcc {
        return Err(TsgoError::Protocol(compact_format(format_args!(
            "expected uint8 marker, got {:x}",
            buf[0]
        ))));
    }
    reader.read_exact(&mut buf)?;
    Ok(buf[0])
}

fn read_bin<R: Read>(reader: &mut R) -> Result<Vec<u8>> {
    let mut tag = [0_u8; 1];
    reader.read_exact(&mut tag)?;
    let len = match tag[0] {
        0xc4 => read_len::<1, _>(reader)?,
        0xc5 => read_len::<2, _>(reader)?,
        0xc6 => read_len::<4, _>(reader)?,
        other => {
            return Err(TsgoError::Protocol(compact_format(format_args!(
                "expected bin marker, got {:x}",
                other
            ))));
        }
    };
    let mut buf = vec![0_u8; len];
    reader.read_exact(&mut buf)?;
    Ok(buf)
}

fn read_len<const N: usize, R: Read>(reader: &mut R) -> Result<usize> {
    match N {
        1 => {
            let mut buf = [0_u8; 1];
            reader.read_exact(&mut buf)?;
            Ok(buf[0] as usize)
        }
        2 => {
            let mut buf = [0_u8; 2];
            reader.read_exact(&mut buf)?;
            Ok(u16::from_be_bytes(buf) as usize)
        }
        4 => {
            let mut buf = [0_u8; 4];
            reader.read_exact(&mut buf)?;
            Ok(u32::from_be_bytes(buf) as usize)
        }
        _ => unreachable!(),
    }
}

fn write_bin<W: Write>(writer: &mut W, bytes: &[u8]) -> Result<()> {
    match bytes.len() {
        0..=255 => writer.write_all(&[0xc4, bytes.len() as u8])?,
        256..=65535 => {
            writer.write_all(&[0xc5])?;
            writer.write_all(&(bytes.len() as u16).to_be_bytes())?;
        }
        _ => {
            writer.write_all(&[0xc6])?;
            writer.write_all(&(bytes.len() as u32).to_be_bytes())?;
        }
    }
    writer.write_all(bytes)?;
    Ok(())
}

#[cfg(test)]
#[path = "msgpack_codec_tests.rs"]
mod tests;
