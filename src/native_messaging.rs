use byteorder::{NativeEndian, ReadBytesExt, WriteBytesExt};
use std::io;
use std::io::{Read, Write};

type JSON = serde_json::Value;

/// Number of bytes in one megabyte. Stored as a usize as that's the type it
/// will be compared against later.
const ONE_MEGABYTE_BYTES: &'static usize = &1_048_576;

#[derive(Debug, PartialEq)]
pub enum NativeMessagingError {
    /// Chrome restricts message sizes to a maximum of 1MB.
    MessageTooLarge(usize),
    NoMoreInput,
    UnknownFailure,
}

/// Decodes data from a reader, where said data is decoded according to
/// Chrome's documentation on native messaging.
/// (https://developer.chrome.com/extensions/nativeMessaging#native-messaging-host-protocol)
///
/// 1. A u32 integer specifies how long the following message is.
/// 2. The message is encoded in JSON.
pub fn read_input<R: Read>(mut input: R) -> Result<JSON, NativeMessagingError> {
    match input.read_u32::<NativeEndian>() {
        Ok(len) => {
            // Due to read_exact looking at a vector's length rather than its
            // capacity we need to preallocate here
            let mut buffer = vec![0; len as usize];

            input
                .read_exact(&mut buffer)
                .map_err(|_| NativeMessagingError::UnknownFailure)?;

            let value = serde_json::from_slice(&buffer)
                .map_err(|_| NativeMessagingError::UnknownFailure)?;

            Ok(value)
        }
        Err(err) => match err.kind() {
            io::ErrorKind::UnexpectedEof => Err(NativeMessagingError::NoMoreInput),
            _ => Err(NativeMessagingError::UnknownFailure),
        },
    }
}

/// Outputs JSON data to a writer, where said data is encoded according to
/// Chrome's documentation on native messaging.
/// (https://developer.chrome.com/extensions/nativeMessaging#native-messaging-host-protocol)
pub fn write_output<W: Write>(mut output: W, val: &JSON) -> Result<W, NativeMessagingError> {
    let msg = serde_json::to_vec(val).map_err(|_| NativeMessagingError::UnknownFailure)?;
    let len = msg.len();

    // Web browsers won't accept a message larger than 1MB
    if len > *ONE_MEGABYTE_BYTES {
        return Err(NativeMessagingError::MessageTooLarge(len));
    }

    output
        .write_u32::<NativeEndian>(len as u32)
        .map_err(|_| NativeMessagingError::UnknownFailure)?;
    output
        .write_all(msg.as_slice())
        .map_err(|_| NativeMessagingError::UnknownFailure)?;
    output
        .flush()
        .map_err(|_| NativeMessagingError::UnknownFailure)?;

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Returns a matching pair of a JSON value and its native messaging-encoded
    /// representation.
    fn encoded_pair() -> (JSON, Vec<u8>) {
        let json = json!({ "property": { "subproperty": "value" } });
        let message = vec![
            36, 0, 0, 0, 123, 34, 112, 114, 111, 112, 101, 114, 116, 121, 34, 58, 123, 34, 115,
            117, 98, 112, 114, 111, 112, 101, 114, 116, 121, 34, 58, 34, 118, 97, 108, 117, 101,
            34, 125, 125,
        ];

        (json, message)
    }

    #[test]
    fn test_reader() {
        let (json, msg) = encoded_pair();
        let read = read_input(msg.as_slice()).unwrap();

        assert_eq!(read, json);
    }

    #[test]
    fn test_writer() {
        let (json, msg) = encoded_pair();
        let written = write_output(Vec::new(), &json).unwrap();

        assert_eq!(written, msg);
    }
}
