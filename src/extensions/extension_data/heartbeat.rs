// RFC 6520

use crate::{
    buffer::CryptoBuffer,
    parse_buffer::{
        ParseBuffer,
        ParseError,
    },
    TlsError,
};

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Heartbeat {
    PeerAllowedToSend = 1,
    PeerNotAllowedToSend = 2,
}

impl Heartbeat {
    pub fn parse(buf: &mut ParseBuffer) -> Result<Self, ParseError> {
        match buf.read_u8()? {
            v if v == Self::PeerAllowedToSend as u8 => Ok(Self::PeerAllowedToSend),
            v if v == Self::PeerAllowedToSend as u8 => Ok(Self::PeerAllowedToSend),
            other => {
                warn!("Read unknown Heartbeat: {}", other);
                Err(ParseError::InvalidData)
            }
        }
    }

    pub fn encode(&self, buf: &mut CryptoBuffer) -> Result<(), TlsError> {
        buf.push(*self as u8)
    }
}
