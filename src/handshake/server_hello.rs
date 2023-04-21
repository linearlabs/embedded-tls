use heapless::Vec;

use crate::cipher_suites::CipherSuite;
use crate::crypto_engine::CryptoEngine;
use crate::extensions::extension_data::key_share::KeyShareEntry;
use crate::extensions::server::ServerExtension;
use crate::extensions::ExtensionType;
use crate::handshake::Random;
use crate::parse_buffer::ParseBuffer;
use crate::TlsError;
use p256::ecdh::{EphemeralSecret, SharedSecret};
use p256::PublicKey;
use sha2::Digest;

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ServerHello<'a> {
    random: Random,
    legacy_session_id_echo: &'a [u8],
    cipher_suite: CipherSuite,
    extensions: Vec<ServerExtension<'a>, 16>,
}

impl<'a> ServerHello<'a> {
    // Source: https://www.rfc-editor.org/rfc/rfc8446#section-4.2 table, rows marked with SH
    const ALLOWED_EXTENSIONS: &[ExtensionType] = &[
        ExtensionType::KeyShare,
        ExtensionType::PreSharedKey,
        ExtensionType::SupportedVersions,
        ExtensionType::PostHandshakeAuth,
    ];

    pub fn read<D: Digest>(buf: &'a [u8], digest: &mut D) -> Result<ServerHello<'a>, TlsError> {
        //trace!("server hello hash [{:x?}]", &buf[..]);
        digest.update(buf);
        Self::parse(&mut ParseBuffer::new(buf))
    }

    pub fn parse(buf: &mut ParseBuffer<'a>) -> Result<ServerHello<'a>, TlsError> {
        //let mut buf = ParseBuffer::new(&buf[0..content_length]);
        //let mut buf = ParseBuffer::new(&buf);

        let _version = buf.read_u16().map_err(|_| TlsError::InvalidHandshake)?;

        let mut random = [0; 32];
        buf.fill(&mut random)?;

        let session_id_length = buf
            .read_u8()
            .map_err(|_| TlsError::InvalidSessionIdLength)?;

        //info!("sh 1");

        let session_id = buf
            .slice(session_id_length as usize)
            .map_err(|_| TlsError::InvalidSessionIdLength)?;
        //info!("sh 2");

        let cipher_suite = buf.read_u16().map_err(|_| TlsError::InvalidCipherSuite)?;
        let cipher_suite = CipherSuite::of(cipher_suite).ok_or(TlsError::InvalidCipherSuite)?;

        ////info!("sh 3");
        // skip compression method, it's 0.
        buf.read_u8()?;

        let extensions = ServerExtension::parse_vector(buf, Self::ALLOWED_EXTENSIONS)?;

        // info!("server random {:x?}", random);
        // info!("server session-id {:x?}", session_id.as_slice());
        // info!("server cipher_suite {:x?}", cipher_suite);
        // info!("server extensions {:?}", extensions);

        Ok(Self {
            random,
            legacy_session_id_echo: session_id.as_slice(),
            cipher_suite,
            extensions,
        })
    }

    pub fn key_share(&self) -> Option<&KeyShareEntry> {
        self.extensions.iter().find_map(|e| {
            if let ServerExtension::KeyShare(entry) = e {
                Some(&entry.0)
            } else {
                None
            }
        })
    }

    pub fn calculate_shared_secret(&self, secret: &EphemeralSecret) -> Option<SharedSecret> {
        let server_key_share = self.key_share()?;
        let server_public_key = PublicKey::from_sec1_bytes(server_key_share.opaque).ok()?;
        Some(secret.diffie_hellman(&server_public_key))
    }

    pub fn initialize_crypto_engine(&self, secret: EphemeralSecret) -> Option<CryptoEngine> {
        let server_key_share = self.key_share()?;

        let group = server_key_share.group;

        let server_public_key = PublicKey::from_sec1_bytes(server_key_share.opaque).ok()?;
        let shared = secret.diffie_hellman(&server_public_key);

        Some(CryptoEngine::new(group, shared))
    }
}
