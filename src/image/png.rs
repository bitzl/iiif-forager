extern crate nom;

use nom::{
    bytes::complete::{tag, take, take_till},
    multi::many0,
    number::complete::{be_u32, be_u8},
    IResult,
};

use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

const PNG_SIGNATURE: &[u8] = &[137, 80, 78, 71, 13, 10, 26, 10];
const ONE: u32 = 1 as u32;

#[derive(Debug, PartialEq)]
pub struct PNG {
    pub width: u32,
    pub height: u32,
    pub chunks: Vec<Chunk>,
}

impl PNG {
    pub fn load(path: &PathBuf) -> Result<PNG, std::io::Error> {
        let mut data = Vec::new();
        let mut file = File::open(path).unwrap();
        file.read_to_end(&mut data).unwrap();
        match parse_png(&data) {
            Ok((_, png)) => Ok(png),
            Err(e) => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("could not parse png: {}", e),
            )),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Chunk {
    // IHDR
    ImageHeader(ImageHeader, u32),
    // PLTE
    // IEND
    End,
    // tRNS
    // cHRM
    // gAMA
    ImageGamma(u32, u32),
    // iCCP
    // sBit
    // sRGB
    // tEXt - Keyword null Text
    Text(String, String, u32),
    // iTXt
    InternationalText(InternationalText, u32),
    // zTXt
    // bKGD
    // hIST
    // pHYs
    // sPLT
    // tIME
    // All chunks we don't know or support yet
    Other(String, Vec<u8>, u32),
}

#[derive(Debug, PartialEq)]
pub struct ImageHeader {
    pub width: u32,
    pub height: u32,
    pub bit_depth: u8,
    pub colour_type: u8,
    pub compression_method: u8,
    pub filter_method: u8,
    pub interlace_method: u8,
}

#[derive(Debug, PartialEq)]
pub struct InternationalText {
    pub keyword: String,
    pub compression_flag: bool,
    pub compression_method: u8,
    pub language_tag: String,
    pub translated_keyword: String,
    pub text: String,
}

pub fn parse_png(input: &[u8]) -> IResult<&[u8], PNG> {
    let (input, _signature) = tag(PNG_SIGNATURE)(input)?;
    let (input, chunks) = many0(parse_chunk)(input)?;
    let image_header = chunks.iter().find_map(|chunk| match chunk {
        Chunk::ImageHeader(image_header, _crc) => Some(image_header),
        _ => None,
    });
    match image_header {
        Some(header) => Ok((
            input,
            PNG {
                width: header.width,
                height: header.height,
                chunks,
            },
        )),
        None => {
            return Err(nom::Err::Error(nom::error::ParseError::from_error_kind(
                input,
                nom::error::ErrorKind::Eof,
            )));
        }
    }
}

fn parse_chunk(input: &[u8]) -> IResult<&[u8], Chunk> {
    let (input, length) = be_u32(input)?;
    let (input, chunk_type) = take_str(input, 4)?;
    match chunk_type {
        "IHDR" => parse_image_header_chunk(input),
        "tEXt" => parse_text_chunk(input, length),
        "iTXt" => parse_international_text_chunk(input, length),
        "gAMA" => parse_image_gamma_chunk(input),
        "IEND" => parse_end_chunk(input),
        _ => parse_other_chunk(input, chunk_type, length),
    }
}

fn take_str(input: &[u8], length: u32) -> IResult<&[u8], &str> {
    let (input, value) = take(length)(input)?;
    match std::str::from_utf8(&value) {
        Ok(string) => Ok((input, string)),
        Err(e) => Err(nom::Err::Error(nom::error::ParseError::from_error_kind(
            input,
            nom::error::ErrorKind::Char,
        ))),
    }
}

fn parse_text_chunk(input: &[u8], length: u32) -> IResult<&[u8], Chunk> {
    let (input, value) = take(length)(input)?;
    let (input, crc) = be_u32(input)?;

    let (key, value) = match key_value(value) {
        Ok((k, v)) => (k, v),
        Err(e) => {
            return Err(nom::Err::Error(nom::error::ParseError::from_error_kind(
                input,
                nom::error::ErrorKind::Eof,
            )))
        }
    };
    Ok((input, Chunk::Text(key, value, crc)))
}

fn take_str_null_delim<'a>(input: &'a [u8]) -> IResult<&[u8], (&'a str, u32)> {
    let (input, value) = take_till(|b| b == 0)(input)?;
    let bytes_taken = value.len();
    match std::str::from_utf8(&value) {
        Ok(string) => Ok((input, (string, bytes_taken as u32))),
        Err(e) => Err(nom::Err::Error(nom::error::ParseError::from_error_kind(
            input,
            nom::error::ErrorKind::Char,
        ))),
    }
}

fn parse_international_text_chunk(input: &[u8], length: u32) -> IResult<&[u8], Chunk> {
    let mut bytes_total = 0;
    let (input, (keyword, bytes_taken)) = take_str_null_delim(input)?;
    bytes_total = bytes_total + bytes_taken;
    let (input, _delim) = take(ONE)(input)?;
    bytes_total = bytes_total + 1;
    let (input, compression_flag) = be_u8(input)?;
    bytes_total = bytes_total + 1;

    let (input, compression_method) = be_u8(input)?;
    bytes_total = bytes_total + 1;

    let (input, (language_tag, bytes_taken)) = take_str_null_delim(input)?;
    bytes_total = bytes_total + bytes_taken;
    let (input, _delim) = take(ONE)(input)?;
    bytes_total = bytes_total + 1;

    let (input, (translated_keyword, bytes_taken)) = take_str_null_delim(input)?;
    bytes_total = bytes_total + bytes_taken;
    let (input, _delim) = take(ONE)(input)?;
    bytes_total = bytes_total + 1;

    let (input, text) = take_str(input, length - bytes_total)?;
    let (input, crc) = be_u32(input)?;

    let international_text = InternationalText {
        keyword: keyword.to_owned(),
        compression_flag: compression_flag == 255,
        compression_method,
        language_tag: language_tag.to_owned(),
        translated_keyword: translated_keyword.to_owned(),
        text: text.to_owned(),
    };

    Ok((input, Chunk::InternationalText(international_text, crc)))
}

fn parse_image_header_chunk(input: &[u8]) -> IResult<&[u8], Chunk> {
    let (input, width) = be_u32(input)?;
    let (input, height) = be_u32(input)?;
    let (input, bit_depth) = be_u8(input)?;
    let (input, colour_type) = be_u8(input)?;
    let (input, compression_method) = be_u8(input)?;
    let (input, filter_method) = be_u8(input)?;
    let (input, interlace_method) = be_u8(input)?;
    let (input, crc) = be_u32(input)?;
    let image_header = ImageHeader {
        width,
        height,
        bit_depth,
        colour_type,
        compression_method,
        filter_method,
        interlace_method,
    };
    Ok((input, Chunk::ImageHeader(image_header, crc)))
}

fn parse_image_gamma_chunk(input: &[u8]) -> IResult<&[u8], Chunk> {
    let (input, gamma) = be_u32(input)?;
    let (input, crc) = be_u32(input)?;
    Ok((input, Chunk::ImageGamma(gamma, crc)))
}

fn parse_end_chunk(input: &[u8]) -> IResult<&[u8], Chunk> {
    // be_u32(input); // consume checksum if present
    Ok((input, Chunk::End))
}

fn parse_other_chunk<'a>(
    input: &'a [u8],
    chunk_type: &str,
    length: u32,
) -> IResult<&'a [u8], Chunk> {
    let (input, value) = take(length)(input)?;
    let (input, crc) = be_u32(input)?;
    let chunk = Chunk::Other(chunk_type.to_owned(), Vec::from(value), crc);
    Ok((input, chunk))
}

fn key_value(data: &[u8]) -> Result<(String, String), std::str::Utf8Error> {
    let (k, v) = match data.iter().position(|&x| x == 0) {
        Some(position) => (
            std::str::from_utf8(&data[0..position])?,
            std::str::from_utf8(&data[position + 1..data.len()])?,
        ),
        None => ("", std::str::from_utf8(data)?),
    };
    Ok((k.to_owned(), v.to_owned()))
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Read;

    use crate::image::png::parse_png;
    use crate::image::png::Chunk;

    #[test]
    fn it_works() {
        let mut data = Vec::new();
        let mut file = File::open("sample/watergate/simple/MOV_0646000.png").unwrap();
        file.read_to_end(&mut data).unwrap();
        let (_, png) = parse_png(&data).unwrap();
        println!("Got the following chunks:");
        for (i, chunk) in png.chunks.iter().enumerate() {
            match chunk {
                Chunk::ImageHeader(image_header, _crc) => println!(
                    "{}: ImageHeader: {}, {}",
                    i, image_header.width, image_header.height
                ),
                Chunk::ImageGamma(value, _crc) => println!("{}: ImageGamma: {}", i, value),
                Chunk::Text(key, value, _crc) => println!("{}: TextChunk: {} → {}", i, key, value),
                Chunk::InternationalText(text, _crc) => {
                    println!("{}: TextChunk: {} → {}", i, text.keyword, text.text)
                }
                Chunk::End => println!("{}: End", i),
                Chunk::Other(chunk_type, _value, _crc) => {
                    println!("{}: OtherChunk of type {}", i, chunk_type)
                }
            }
        }
    }
}
