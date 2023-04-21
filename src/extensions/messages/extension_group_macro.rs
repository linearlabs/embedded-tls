#[doc(hidden)]
#[macro_export]
macro_rules! extension_group {
    (pub enum $name:ident<'a> {
        $($extension:ident($extension_data:ty)),+
    }) => {
        pub enum $name<'a> {
            $($extension($extension_data)),+
        }

        impl<'a> $name<'a> {
            pub fn extension_type(&self) -> crate::extensions::ExtensionType {
                match self {
                    $(Self::$extension(_) => crate::extensions::ExtensionType::$extension),+
                }
            }

            pub fn encode(&self, buf: &mut crate::buffer::CryptoBuffer) -> Result<(), crate::TlsError> {
                self.extension_type().encode(buf)?;

                buf.with_u16_length(|buf| match self {
                    $(Self::$extension(ext_data) => ext_data.encode(buf)),+
                })
            }

            pub fn parse(buf: &mut crate::parse_buffer::ParseBuffer<'a>) -> Result<Self, crate::TlsError> {
                match crate::extensions::ExtensionType::parse(buf).map_err(|err| {
                    match err {
                        crate::parse_buffer::ParseError::InvalidData => crate::TlsError::UnknownExtensionType,
                        _ => crate::TlsError::DecodeError,
                    }
                })? {
                    $(crate::extensions::ExtensionType::$extension => Ok(Self::$extension(<$extension_data>::parse(buf).map_err(|_| crate::TlsError::DecodeError)?)),)+
                    #[allow(unreachable_patterns)]
                    other => {
                        warn!("Read unexpected ExtensionType: {:?}", other);
                        // Section 4.2.  Extensions
                        // If an implementation receives an extension
                        // which it recognizes and which is not specified for the message in
                        // which it appears, it MUST abort the handshake with an
                        // "illegal_parameter" alert.
                        return Err(crate::TlsError::AbortHandshake(
                            crate::alert::AlertLevel::Fatal,
                            crate::alert::AlertDescription::IllegalParameter,
                        ));
                    }
                }
            }
        }
    };
}