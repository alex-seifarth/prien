
/// Decoder for UTF-8 byte sequences.
/// # Usage
/// ```
/// use util::utf8::Decoder;
///  let mut decoder = Decoder::new();
///  assert_eq!(decoder.decode(0xc2), Ok( None ));
///  assert_eq!(decoder.decode(0xa2), Ok( Some('\u{00a2}')));
/// ```
pub struct Decoder {
    code: u32,
    remaining: u32,
}

impl Decoder {

    /// Creates a new UTF-8 byte sequence decoder in initial state.
    pub fn new() -> Decoder {
        Decoder{ code: 0, remaining: 0 }
    }

    /// Resets the decoder's internal state - i.e. the decoder can again begin decoding
    /// at the start of a new UTF-8 sequence
    pub fn reset(&mut self) {
        self.remaining = 0
    }

    /// Decodes another byte and returns:
    /// - Ok(None):     if the sequence is not complete, further bytes are expected
    /// - Ok(Some(ch)): if the sequence is completed and a 32 bit long unicode character is returned
    /// - Err(()):      if there is an encoding error encountered
    pub fn decode(&mut self, byte: u8) -> Result< Option<char>, () > {
        if self.remaining == 0 {
            self.decode_ready(byte)
        }
        else {
            self.decode_incomplete(byte)
        }
    }

    fn decode_ready(&mut self, byte: u8) -> Result< Option<char>, () > {
        if 0x00 == (byte & 0x80) {
            Decoder::finalize_char(byte as u32)
        }
        else if 0xc0 == (byte & 0xe0) {
            self.code = (byte & 0x1f) as u32;
            self.remaining = 1;
            Ok( None )
        }
        else if 0xe0 == (byte & 0xf0) {
            self.code = (byte & 0x0f) as u32;
            self.remaining = 2;
            Ok( None )
        }
        else if 0xf0 == (byte & 0xf8) {
            self.code = (byte & 0x07) as u32;
            self.remaining = 3;
            Ok( None )
        }
        else {
            Err(())
        }
    }

    fn decode_incomplete(&mut self, byte: u8) -> Result< Option<char>, () > {
        let new_part = match byte & 0xc0 {
            0x80 => (byte & 0x3f) as u32,
            _ => return Err(())
        };
        self.code = (self.code << 6) | new_part;
        self.remaining -= 1;
        if 0 == self.remaining {
            Decoder::finalize_char(self.code)
        }
        else {
            Ok( None )
        }
    }

    fn finalize_char(code: u32) -> Result< Option<char>, () > {
        match std::char::from_u32(code) {
            Some(c) => Ok( Some( c )),
            None => Err(())
        }
    }
}

#[cfg(test)]
mod test {
    use super::Decoder;

    #[test]
    fn valid_utf8_single() {
        let mut decoder = Decoder::new();

        assert_eq!(decoder.decode(0x41), Ok( Some( 'A' )));
        assert_eq!(decoder.decode(0x39), Ok( Some( '9' )));
        assert_eq!(decoder.decode(0x0a), Ok( Some( '\n' )));
    }

    #[test]
    fn valid_utf8_double() {
        let mut decoder = Decoder::new();

        assert_eq!(decoder.decode(0x42), Ok( Some( 'B' )));
        assert_eq!(decoder.decode(0xc2), Ok( None ));
        assert_eq!(decoder.decode(0xa2), Ok( Some( '\u{00a2}' )));
        assert_eq!(decoder.decode(0xc9), Ok( None ));
        assert_eq!(decoder.decode(0xb8), Ok( Some( '\u{0278}' )));
        assert_eq!(decoder.decode(0x43), Ok( Some( 'C' )));
    }

    #[test]
    fn invalid_utf8_double() {
        let mut decoder = Decoder::new();
        assert_eq!(decoder.decode(0xc2), Ok( None ));
        assert_eq!(decoder.decode(0xc1), Err(()));

        decoder.reset();
        assert_eq!(decoder.decode(0x44), Ok( Some( 'D' )));
    }

    #[test]
    fn valid_utf8_triple() {
        let mut decoder = Decoder::new();
        assert_eq!(decoder.decode( 0xe2), Ok( None ));
        assert_eq!(decoder.decode(0x82), Ok( None ));
        assert_eq!(decoder.decode(0xac), Ok( Some( '\u{20ac}')));
    }


    #[test]
    fn valid_utf8_quadr() {
        let mut decoder = Decoder::new();

        assert_eq!(decoder.decode(0xc9), Ok( None ));
        assert_eq!(decoder.decode( 0xb8), Ok( Some( '\u{0278}' )));
        assert_eq!(decoder.decode(0xf0), Ok( None ));
        assert_eq!(decoder.decode( 0x90), Ok( None ));
        assert_eq!(decoder.decode(0x8d), Ok( None ));
        assert_eq!(decoder.decode(0x88), Ok( Some( '\u{10348}')));
        assert_eq!(decoder.decode(0x45), Ok( Some( 'E' )));
    }

    #[test]
    fn invalid_utf8() {
        let mut decoder = Decoder::new();

        // initial byte wrong
        assert_eq!(decoder.decode(0xf9), Err(()));
        assert_eq!(decoder.decode(0xa2), Err(()));

        // second byte 11-- instead of 10--
        assert_eq!(decoder.decode(0xf0), Ok( None ));
        assert_eq!(decoder.decode(0xc2), Err(()));
        decoder.reset();

        // second byte wrong 0--- instead of 10--
        assert_eq!(decoder.decode(0xf2), Ok( None ));
        assert_eq!(decoder.decode(0x7f), Err(()));
    }
}