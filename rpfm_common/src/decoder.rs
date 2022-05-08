//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with the `Decoder` trait, to decode bytes to readable data.

This module contains the `Decoder` trait and his implementation for `&[u8]`. This trait allow us
to safely (yes, it covers your `index-out-of-bounds` bugs) decode any type of data contained within
a PackFile/PackedFile.

Note: If you change anything from here, remember to update the `decoder_test.rs` file for it.
!*/

use byteorder::{ByteOrder, LittleEndian};
use encoding::{Encoding, DecoderTrap};
use encoding::all::ISO_8859_1;

use crate::error::{RCommonError, Result};

/// These constants are needed to work with LEB_128 encoded numbers.
pub const LEB128_CONTROL_BIT: u8 = 0b10000000;
pub const LEB128_SIGNED_MAX: u8 = 0b00111111;
pub const LEB128_UNSIGNED_MAX: u8 = 0b01111111;
pub const U32_BITS: u32 = 32;

//---------------------------------------------------------------------------//
//                      `Decoder` Trait Definition
//---------------------------------------------------------------------------//

/// This trait allow us to easely decode all kind of data from a `&[u8]`.
pub trait Decoder {

    /// This function returns an slice after his bounds have been checked, to avoid `index-out-of-range` errors.
    ///
    /// You must provide an slice to read from, the position of the first byte you want to read, and the amount of bytes to read.
    fn decode_bytes_checked(&self, offset: usize, size: usize) -> Result<&Self>;

    /// This function allows us to decode a boolean from a byte. This is simple: 0 is false, 1 is true. It only uses a byte.
    fn decode_bool(&self, offset: usize) -> Result<bool>;

    /// This function allows us to decode an u8 integer from raw data.
    fn decode_integer_u8(&self, offset: usize) -> Result<u8>;

    /// This function allows us to decode an u16 integer from raw data.
    fn decode_integer_u16(&self, offset: usize) -> Result<u16>;

    /// This function allows us to decode an u24 integer from raw data.
    fn decode_integer_u24(&self, offset: usize) -> Result<u32>;

    /// This function allows us to decode an u32 integer from raw data.
    fn decode_integer_u32(&self, offset: usize) -> Result<u32>;

    /// This function allows us to decode an u64 integer from raw data.
    fn decode_integer_u64(&self, offset: usize) -> Result<u64>;

    /// This function allows us to decode an i8 integer from raw data.
    fn decode_integer_i8(&self, offset: usize) -> Result<i8>;

    /// This function allows us to decode an i16 integer from raw data.
    fn decode_integer_i16(&self, offset: usize) -> Result<i16>;

    /// This function allows us to decode an i24 integer from raw data.
    fn decode_integer_i24(&self, offset: usize) -> Result<i32>;

    /// This function allows us to decode an i32 integer from raw data.
    fn decode_integer_i32(&self, offset: usize) -> Result<i32>;

    /// This function allows us to decode an i64 integer from raw data.
    fn decode_integer_i64(&self, offset: usize) -> Result<i64>;

    /// This function allows us to decode a f32 float from raw data.
    fn decode_float_f32(&self, offset: usize) -> Result<f32>;

    /// This function allows us to decode a f64 float from raw data.
    fn decode_float_f64(&self, offset: usize) -> Result<f64>;

    /// This function allows us to decode a u32 encoded colour from raw data.
    fn decode_integer_colour_rgb(&self, offset: usize) -> Result<u32>;

    /// This function allows us to decode an UTF-8 String  from raw data.
    fn decode_string_u8(&self, offset: usize, size: usize) -> Result<String>;

    /// This function allows us to decode an ISO_8859_1 String from raw data.
    fn decode_string_u8_iso_8859_1(&self, offset: usize, size: usize) -> Result<String>;

    /// This function allows us to decode a 00-Padded UTF-8 String from raw data.
    ///
    /// This type of String has a fixed size and, when the characters end, it's filled with `00` bytes until it reach his size.
    /// We return the decoded String and his full size when encoded (string + zeros).
    fn decode_string_u8_0padded(&self, offset: usize, size: usize) -> Result<(String, usize)>;

    /// This function allows us to decode a 00-Terminated UTF-8 String from raw data.
    ///
    /// This type of String has a 00 byte at his end and variable size. It advances the provided offset while decoding.
    /// We return the decoded String and his size.
    fn decode_string_u8_0terminated(&self, offset: usize) -> Result<(String, usize)>;

    /// This function allows us to decode an UTF-16 String from raw data.
    fn decode_string_u16(&self, offset: usize, size: usize) -> Result<String>;

    /// This function allows us to decode a 00-Padded UTF-16 String from raw data.
    ///
    /// This type of String has a fixed size and, when the characters end, it's filled with `00` bytes until it reach his size.
    /// We return the decoded String and his full size when encoded (string + zeros).
    fn decode_string_u16_0padded(&self, offset: usize, size: usize) -> Result<(String, usize)>;

    /// This function allows us to decode an encoded RGB colour as a String from raw data.
    fn decode_string_colour_rgb(&self, offset: usize) -> Result<String>;

    /// This function allows us to decode a boolean from a byte, moving the provided index to the byte where the next data starts.
    fn decode_packedfile_bool(&self, offset: usize, index: &mut usize) -> Result<bool>;

    /// This function allows us to decode an u8 integer from raw data, moving the provided index to the byte where the next data starts.
    fn decode_packedfile_integer_u8(&self, offset: usize, index: &mut usize) -> Result<u8>;

    /// This function allows us to decode an u16 integer from raw data, moving the provided index to the byte where the next data starts.
    fn decode_packedfile_integer_u16(&self, offset: usize, index: &mut usize) -> Result<u16>;

    /// This function allows us to decode an u24 integer from raw data, moving the provided index to the byte where the next data starts.
    fn decode_packedfile_integer_u24(&self, offset: usize, index: &mut usize) -> Result<u32>;

    /// This function allows us to decode an u32 encoded integer from raw data, moving the provided index to the byte where the next data starts.
    fn decode_packedfile_integer_u32(&self, offset: usize, index: &mut usize) -> Result<u32>;

    /// This function allows us to decode an u64 encoded integer from raw data, moving the provided index to the byte where the next data starts.
    fn decode_packedfile_integer_u64(&self, offset: usize, index: &mut usize) -> Result<u64>;

    /// This function allows us to decode an unsigned leb128 variant-length integer (CA's own twist and flavour) from raw data, moving the provided index to the byte where the next data starts.
    fn decode_packedfile_integer_cauleb128(&self, index: &mut usize) -> Result<u32>;

    /// This function allows us to decode an i8 integer from raw data, moving the provided index to the byte where the next data starts.
    fn decode_packedfile_integer_i8(&self, offset: usize, index: &mut usize) -> Result<i8>;

    /// This function allows us to decode an i16 integer from raw data, moving the provided index to the byte where the next data starts.
    fn decode_packedfile_integer_i16(&self, offset: usize, index: &mut usize) -> Result<i16>;

    /// This function allows us to decode an i24 encoded integer from raw data, moving the provided index to the byte where the next data starts.
    fn decode_packedfile_integer_i24(&self, offset: usize, index: &mut usize) -> Result<i32>;

    /// This function allows us to decode an i32 encoded integer from raw data, moving the provided index to the byte where the next data starts.
    fn decode_packedfile_integer_i32(&self, offset: usize, index: &mut usize) -> Result<i32>;

    /// This function allows us to decode an i64 encoded integer from raw data, moving the provided index to the byte where the next data starts.
    fn decode_packedfile_integer_i64(&self, offset: usize, index: &mut usize) -> Result<i64>;

    /// This function allows us to decode an optional i16 integer from raw data, moving the provided index to the byte where the next data starts.
    fn decode_packedfile_optional_integer_i16(&self, offset: usize, index: &mut usize) -> Result<i16>;

    /// This function allows us to decode an optional i32 encoded integer from raw data, moving the provided index to the byte where the next data starts.
    fn decode_packedfile_optional_integer_i32(&self, offset: usize, index: &mut usize) -> Result<i32>;

    /// This function allows us to decode an optional i64 encoded integer from raw data, moving the provided index to the byte where the next data starts.
    fn decode_packedfile_optional_integer_i64(&self, offset: usize, index: &mut usize) -> Result<i64>;

    /// This function allows us to decode an f32 encoded float from raw data, moving the provided index to the byte where the next data starts.
    fn decode_packedfile_float_f32(&self, offset: usize, index: &mut usize) -> Result<f32>;

    /// This function allows us to decode an f64 encoded float from raw data, moving the provided index to the byte where the next data starts.
    fn decode_packedfile_float_f64(&self, offset: usize, index: &mut usize) -> Result<f64>;

    /// This function allows us to decode an u32 encoded colour from raw data, moving the provided index to the byte where the next data starts.
    fn decode_packedfile_integer_colour_rgb(&self, offset: usize, index: &mut usize) -> Result<u32>;

    /// This function allows us to decode an UTF-8 encoded String from raw data, moving the provided index to the byte where the next data starts.
    fn decode_packedfile_string_u8(&self, offset: usize, index: &mut usize) -> Result<String>;

    /// This function allows us to decode an UTF-8 0-Terminated String from raw data, moving the provided index to the byte where the next data starts.
    fn decode_packedfile_string_u8_0terminated(&self, offset: usize, index: &mut usize) -> Result<String>;

    /// This function allows us to decode an UTF-16 String from raw data, moving the provided index to the byte where the next data starts.
    fn decode_packedfile_string_u16(&self, offset: usize, index: &mut usize) -> Result<String>;

    /// This function allows us to decode an UTF-8 optional String from raw data, moving the provided index to the byte where the next data starts.
    ///
    /// These Strings's first byte it's a boolean that indicates if the string has something. If false, the string it's just that byte.
    /// If true, there is a normal UTF-8 encoded String after that byte.
    fn decode_packedfile_optional_string_u8(&self, offset: usize, index: &mut usize) -> Result<String>;

    /// This function allows us to decode an UTF-16 optional String from raw data, moving the provided index to the byte where the next data starts.
    ///
    /// These Strings's first byte it's a boolean that indicates if the string has something. If false, the string it's just that byte.
    /// If true, there is a normal UTF-16 encoded String after that byte.
    fn decode_packedfile_optional_string_u16(&self, offset: usize, index: &mut usize) -> Result<String>;

    /// This function allows us to decode an encoded RGB colour as a String from raw data, moving the provided index to the byte where the next data starts.
    fn decode_packedfile_string_colour_rgb(&self, offset: usize, index: &mut usize) -> Result<String>;
}

/// Implementation of trait `Decoder` for `&[u8]`.
impl Decoder for [u8] {

    fn decode_bytes_checked(&self, offset: usize, size: usize) -> Result<&[u8]> {
        if size == 0 { Ok(&[]) }
        else if self.len() >= offset + size {
            if self.get(size - 1).is_some() { Ok(&self[offset..offset + size]) }
            else { Err(RCommonError::DecodingNotMoreBytesToDecode.into()) }
        }
        else { Err(RCommonError::DecodingNotMoreBytesToDecode.into()) }
    }

    //---------------------------------------------------------------------------//
    //                          Normal Decoders
    //---------------------------------------------------------------------------//

    fn decode_bool(&self, offset: usize) -> Result<bool> {
        let value = self.decode_integer_u8(offset)?;
        match value {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(RCommonError::DecodingBoolError(value).into()),
        }
    }

    fn decode_integer_u8(&self, offset: usize) -> Result<u8> {
        self.get(offset).copied().ok_or_else(|| RCommonError::DecodingNoBytesLeftError)
    }

    fn decode_integer_u16(&self, offset: usize) -> Result<u16> {
        if self.len() >= offset + 2 { Ok(LittleEndian::read_u16(&self[offset..])) }
        else { Err(RCommonError::DecodingNotEnoughBytesToDecodeForType("u16".to_owned(), 2, offset.checked_sub(self.len()))) }
    }

    fn decode_integer_u24(&self, offset: usize) -> Result<u32> {
        if self.len() >= offset + 3 {
            let mut data = Vec::with_capacity(4);
            data.extend_from_slice(&self[offset..offset + 3]);
            data.push(0);
            Ok(LittleEndian::read_u32(&data)) }
        else { Err(RCommonError::DecodingNotEnoughBytesToDecodeForType("u24".to_owned(), 3, offset.checked_sub(self.len()))) }
    }

    fn decode_integer_u32(&self, offset: usize) -> Result<u32> {
        if self.len() >= offset + 4 { Ok(LittleEndian::read_u32(&self[offset..])) }
        else { Err(RCommonError::DecodingNotEnoughBytesToDecodeForType("u32".to_owned(), 4, offset.checked_sub(self.len()))) }
    }

    fn decode_integer_u64(&self, offset: usize) -> Result<u64> {
        if self.len() >= offset + 8 { Ok(LittleEndian::read_u64(&self[offset..])) }
        else { Err(RCommonError::DecodingNotEnoughBytesToDecodeForType("u64".to_owned(), 8, offset.checked_sub(self.len()))) }
    }

    fn decode_integer_i8(&self, offset: usize) -> Result<i8> {
        self.get(offset).map(|x| *x as i8).ok_or_else(|| RCommonError::DecodingNoBytesLeftError)
    }

    fn decode_integer_i16(&self, offset: usize) -> Result<i16> {
        if self.len() >= offset + 2 { Ok(LittleEndian::read_i16(&self[offset..])) }
        else { Err(RCommonError::DecodingNotEnoughBytesToDecodeForType("i16".to_owned(), 2, offset.checked_sub(self.len()))) }
    }

    fn decode_integer_i24(&self, offset: usize) -> Result<i32> {
        if self.len() >= offset + 3 {
            let mut data = Vec::with_capacity(4);
            data.extend_from_slice(&self[offset..offset + 3]);
            data.push(0);
            Ok(LittleEndian::read_i32(&data)) }
        else { Err(RCommonError::DecodingNotEnoughBytesToDecodeForType("i24".to_owned(), 3, offset.checked_sub(self.len()))) }
    }

    fn decode_integer_i32(&self, offset: usize) -> Result<i32> {
        if self.len() >= offset + 4 { Ok(LittleEndian::read_i32(&self[offset..])) }
        else { Err(RCommonError::DecodingNotEnoughBytesToDecodeForType("i32".to_owned(), 4, offset.checked_sub(self.len()))) }
    }

    fn decode_integer_i64(&self, offset: usize) -> Result<i64> {
        if self.len() >= offset + 8 { Ok(LittleEndian::read_i64(&self[offset..])) }
        else { Err(RCommonError::DecodingNotEnoughBytesToDecodeForType("i64".to_owned(), 8, offset.checked_sub(self.len()))) }
    }

    fn decode_float_f32(&self, offset: usize) -> Result<f32> {
        if self.len() >= offset + 4 { Ok(LittleEndian::read_f32(&self[offset..])) }
        else { Err(RCommonError::DecodingNotEnoughBytesToDecodeForType("f32".to_owned(), 4, offset.checked_sub(self.len()))) }
    }

    fn decode_float_f64(&self, offset: usize) -> Result<f64> {
        if self.len() >= offset + 8 { Ok(LittleEndian::read_f64(&self[offset..])) }
        else { Err(RCommonError::DecodingNotEnoughBytesToDecodeForType("f64".to_owned(), 8, offset.checked_sub(self.len()))) }
    }

    fn decode_string_u8(&self, offset: usize, size: usize) -> Result<String> {
        if self.len() < offset + size {
            return Err(RCommonError::DecodingNotEnoughBytesToDecodeForType("UTF-8 String".to_owned(), size, offset.checked_sub(self.len())))
        }
        String::from_utf8(self[offset..offset + size].to_vec()).map_err(From::from)
    }

    fn decode_integer_colour_rgb(&self, offset: usize) -> Result<u32> {
        if self.len() >= offset + 4 { Ok(LittleEndian::read_u32(&self[offset..])) }
        else { Err(RCommonError::DecodingNotEnoughBytesToDecodeForType("RGB Colour".to_owned(), 4, offset.checked_sub(self.len()))) }
    }

    fn decode_string_u8_iso_8859_1(&self, offset: usize, size: usize) -> Result<String> {
        if self.len() < offset + size {
            return Err(RCommonError::DecodingNotEnoughBytesToDecodeForType("UTF-8 String from an ISO8859-1 String".to_owned(), size, offset.checked_sub(self.len())))
        }

        ISO_8859_1.decode(&self[offset..offset + size], DecoderTrap::Replace).map_err(|error| RCommonError::DecodeUTF8FromISO8859Error(error.to_string()))
    }

    fn decode_string_u8_0padded(&self, offset: usize, size: usize) -> Result<(String, usize)> {
        if self.len() < offset + size {
            return Err(RCommonError::DecodingNotEnoughBytesToDecodeForType("UTF-8 0-Padded String".to_owned(), size, offset.checked_sub(self.len())))
        }

        let size_no_zeros = self[offset..offset + size].iter().position(|x| *x == 0).map_or(size, |x| x);
        let string_decoded = String::from_utf8(self[offset..offset + size_no_zeros].to_vec())?;
        Ok((string_decoded, size))
    }

    fn decode_string_u8_0terminated(&self, offset: usize) -> Result<(String, usize)> {
        if self.len() < offset {
            return Err(RCommonError::DecodingNotMoreBytesToDecode.into());
        }

        let (ends_in_zero, size) = self[offset..].iter().position(|x| *x == 0).map_or((false, self[offset..].len()), |x| (true, x));
        let string_decoded = String::from_utf8_lossy(&self[offset..offset + size]).to_string();
        Ok((string_decoded, if ends_in_zero { size + 1 } else { size }))
    }

    fn decode_string_u16(&self, offset: usize, size: usize) -> Result<String> {
        if self.len() < offset + size && size % 2 == 0 {
            return Err(RCommonError::DecodingNotEnoughBytesToDecodeForType("UTF-16 String".to_owned(), size, offset.checked_sub(self.len())))
        }

        let u16_characters = self[offset..offset + size]
            .chunks_exact(2)
            .map(|x| u16::from_le_bytes([x[0], x[1]]))
            .collect::<Vec<u16>>();
        String::from_utf16(&u16_characters).map_err(From::from)
    }

    fn decode_string_u16_0padded(&self, offset: usize, size: usize) -> Result<(String, usize)> {
        if self.len() < offset + size {
            return Err(RCommonError::DecodingNotEnoughBytesToDecodeForType("UTF-16 0-Padded String".to_owned(), size, offset.checked_sub(self.len())))
        }

        let size_no_zeros = self[offset..offset + size].iter().step_by(2).position(|x| *x == 0).map_or(size, |x| x * 2);
        let u16_characters = self[offset..offset + size_no_zeros].chunks_exact(2).map(|x| u16::from_le_bytes([x[0], x[1]])).collect::<Vec<u16>>();
        let string_decoded = String::from_utf16(&u16_characters)?;
        Ok((string_decoded, size))
    }

    fn decode_string_colour_rgb(&self, offset: usize) -> Result<String> {
        if self.len() < offset + 4 {
            return Err(RCommonError::DecodingNotEnoughBytesToDecodeForType("RGB Colour".to_owned(), 4, offset.checked_sub(self.len())))
        }
        // Padding to 8 zeros so we don't lose the first one, then remove the last two zeros (alpha?).
        // REMEMBER, FORMAT ENCODED IS BBGGRR00.
        let value = format!("{:06X?}", LittleEndian::read_u32(&self[offset..]));
        Ok(value)
    }

    //---------------------------------------------------------------------------//
    //                              Indexed Decoders
    //---------------------------------------------------------------------------//

    fn decode_packedfile_bool(&self, offset: usize, index: &mut usize) -> Result<bool> {
        let result = self.decode_bool(offset);
        if result.is_ok() { *index += 1; }
        result
    }

    fn decode_packedfile_integer_u8(&self, offset: usize, index: &mut usize) -> Result<u8> {
        let result = self.decode_integer_u8(offset);
        if result.is_ok() { *index += 1; }
        result
    }

    fn decode_packedfile_integer_u16(&self, offset: usize, index: &mut usize) -> Result<u16> {
        let result = self.decode_integer_u16(offset);
        if result.is_ok() { *index += 2; }
        result
    }

    fn decode_packedfile_integer_u24(&self, offset: usize, index: &mut usize) -> Result<u32> {
        let result = self.decode_integer_u24(offset);
        if result.is_ok() { *index += 3; }
        result
    }

    fn decode_packedfile_integer_u32(&self, offset: usize, index: &mut usize) -> Result<u32> {
        let result = self.decode_integer_u32(offset);
        if result.is_ok() { *index += 4; }
        result
    }

    fn decode_packedfile_integer_u64(&self, offset: usize, index: &mut usize) -> Result<u64> {
        let result = self.decode_integer_u64(offset);
        if result.is_ok() { *index += 8; }
        result
    }

    fn decode_packedfile_integer_cauleb128(&self, index: &mut usize) -> Result<u32> {
        let mut size: u32 = 0;
        let mut byte = self.get(*index).ok_or_else(|| RCommonError::DecodingNotMoreBytesToDecode)?;

        while(byte & 0x80) != 0 {
            size = (size << 7) | (byte & 0x7f) as u32;
            *index += 1;

            // Check the new byte is even valid before continuing.
            byte = self.get(*index).ok_or_else(|| RCommonError::DecodingNotMoreBytesToDecode)?;
        }

        size = (size << 7) | (byte & 0x7f) as u32;
        *index += 1;

        Ok(size)
    }

    fn decode_packedfile_integer_i8(&self, offset: usize, index: &mut usize) -> Result<i8> {
        let result = self.decode_integer_i8(offset);
        if result.is_ok() { *index += 1; }
        result
    }

    fn decode_packedfile_integer_i16(&self, offset: usize, index: &mut usize) -> Result<i16> {
        let result = self.decode_integer_i16(offset);
        if result.is_ok() { *index += 2; }
        result
    }

    fn decode_packedfile_integer_i24(&self, offset: usize, index: &mut usize) -> Result<i32> {
        let result = self.decode_integer_i24(offset);
        if result.is_ok() { *index += 3; }
        result
    }

    fn decode_packedfile_integer_i32(&self, offset: usize, index: &mut usize) -> Result<i32> {
        let result = self.decode_integer_i32(offset);
        if result.is_ok() { *index += 4; }
        result
    }

    fn decode_packedfile_integer_i64(&self, offset: usize, index: &mut usize) -> Result<i64> {
        let result = self.decode_integer_i64(offset);
        if result.is_ok() { *index += 8; }
        result
    }

    fn decode_packedfile_optional_integer_i16(&self, offset: usize, index: &mut usize) -> Result<i16> {
        let result = self.decode_bool(offset);
        if result.is_ok() { *index += 1; }

        let result = self.decode_integer_i16(offset);
        if result.is_ok() { *index += 2; }
        result

    }

    fn decode_packedfile_optional_integer_i32(&self, offset: usize, index: &mut usize) -> Result<i32> {
        let result = self.decode_bool(offset);
        if result.is_ok() { *index += 1; }

        let result = self.decode_integer_i32(offset);
        if result.is_ok() { *index += 4; }
        result

    }

    fn decode_packedfile_optional_integer_i64(&self, offset: usize, index: &mut usize) -> Result<i64> {
        let result = self.decode_bool(offset);
        if result.is_ok() { *index += 1; }

        let result = self.decode_integer_i64(offset);
        if result.is_ok() { *index += 8; }
        result
    }

    fn decode_packedfile_float_f32(&self, offset: usize, index: &mut usize) -> Result<f32> {
        let result = self.decode_float_f32(offset);
        if result.is_ok() { *index += 4; }
        result
    }

    fn decode_packedfile_float_f64(&self, offset: usize, index: &mut usize) -> Result<f64> {
        let result = self.decode_float_f64(offset);
        if result.is_ok() { *index += 8; }
        result
    }

    fn decode_packedfile_integer_colour_rgb(&self, offset: usize, index: &mut usize) -> Result<u32> {
        let result = self.decode_integer_colour_rgb(offset);
        if result.is_ok() { *index += 4; }
        result
    }

    fn decode_packedfile_string_u8(&self, offset: usize, index: &mut usize) -> Result<String> {
        if let Ok(size) = self.decode_packedfile_integer_u16(offset, index) {
            let result = self.decode_string_u8(offset + 2, size as usize);
            if result.is_err() { *index -= 2; } else { *index += size as usize; }
            result
        }
        else {
            return Err(RCommonError::DecodingStringSizeError("UTF-8 String".to_owned(), offset.checked_sub(self.len()), 2))
        }
    }

    fn decode_packedfile_string_u8_0terminated(&self, offset: usize, index: &mut usize) -> Result<String> {
        let (string, size) = self.decode_string_u8_0terminated(offset)?;
        *index += size;
        Ok(string)
    }

    fn decode_packedfile_string_u16(&self, offset: usize, index: &mut usize) -> Result<String> {
        if let Ok(size) = self.decode_packedfile_integer_u16(offset, index) {

            // We wrap this to avoid overflow, as the limit of this is 65,535. We do this because u16 Strings
            // counts pairs of bytes (u16), not single bytes.
            let size = size.wrapping_mul(2) as usize;
            let result = self.decode_string_u16(offset + 2, size);
            if result.is_err() { *index -= 2; } else { *index += size; }
            result
        }
        else {
            return Err(RCommonError::DecodingStringSizeError("UTF-16 String".to_owned(), offset.checked_sub(self.len()), 2))
        }
    }

    fn decode_packedfile_optional_string_u8(&self, offset: usize, index: &mut usize) -> Result<String> {
        let is = self.decode_packedfile_bool(offset, index)
            .map_err(|_| RCommonError::DecodingOptionalStringBoolError("UTF-8 Optional String".to_owned()))?;

        if !is {
            return Ok(String::new())
        }

        let result = self.decode_packedfile_string_u8(offset + 1, index);
        if result.is_err() { *index -= 1 };
        result
    }

    fn decode_packedfile_optional_string_u16(&self, offset: usize, index: &mut usize) -> Result<String> {
        let is = self.decode_packedfile_bool(offset, index)
            .map_err(|_| RCommonError::DecodingOptionalStringBoolError("UTF-16 Optional String".to_owned()))?;

        if !is {
            return Ok(String::new())
        }

        let result = self.decode_packedfile_string_u16(offset + 1, index);
        if result.is_err() { *index -= 1 };
        result
    }

    fn decode_packedfile_string_colour_rgb(&self, offset: usize, index: &mut usize) -> Result<String> {
        let result = self.decode_string_colour_rgb(offset);
        if result.is_ok() { *index += 4; }
        result
    }
}
