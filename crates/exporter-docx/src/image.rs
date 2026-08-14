use document_ir::MediaTypeIr;

use crate::{DocxError, DocxLimit, DocxLimits};

const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ImageInfo {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) extension: &'static str,
}

pub(crate) fn validate_image(
    bytes: &[u8],
    media_type: MediaTypeIr,
    limits: &DocxLimits,
) -> Result<ImageInfo, DocxError> {
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > limits.max_image_bytes {
        return Err(DocxError::LimitExceeded(DocxLimit::ImageBytes));
    }
    let info = match media_type {
        MediaTypeIr::Png => {
            if !bytes.starts_with(PNG_SIGNATURE) {
                return Err(DocxError::MediaTypeMismatch);
            }
            validate_png(bytes)?
        }
        MediaTypeIr::Jpeg => {
            if !bytes.starts_with(&[0xff, 0xd8]) {
                return Err(DocxError::MediaTypeMismatch);
            }
            validate_jpeg(bytes)?
        }
    };
    if info.width > limits.max_image_dimension || info.height > limits.max_image_dimension {
        return Err(DocxError::LimitExceeded(DocxLimit::ImageDimension));
    }
    let pixels = u64::from(info.width)
        .checked_mul(u64::from(info.height))
        .ok_or(DocxError::LimitExceeded(DocxLimit::ImagePixels))?;
    if pixels > limits.max_image_pixels {
        return Err(DocxError::LimitExceeded(DocxLimit::ImagePixels));
    }
    Ok(info)
}

fn validate_png(bytes: &[u8]) -> Result<ImageInfo, DocxError> {
    let mut offset = PNG_SIGNATURE.len();
    let mut chunk_index = 0_usize;
    let mut width = 0_u32;
    let mut height = 0_u32;
    let mut saw_idat = false;
    let mut saw_iend = false;
    while offset < bytes.len() {
        let header_end = offset.checked_add(8).ok_or(DocxError::MalformedImage)?;
        if header_end > bytes.len() {
            return Err(DocxError::MalformedImage);
        }
        let length = u32::from_be_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .map_err(|_| DocxError::MalformedImage)?,
        ) as usize;
        let chunk_type = &bytes[offset + 4..offset + 8];
        let data_start = header_end;
        let data_end = data_start
            .checked_add(length)
            .ok_or(DocxError::MalformedImage)?;
        let chunk_end = data_end.checked_add(4).ok_or(DocxError::MalformedImage)?;
        if chunk_end > bytes.len() {
            return Err(DocxError::MalformedImage);
        }
        let expected_crc = u32::from_be_bytes(
            bytes[data_end..chunk_end]
                .try_into()
                .map_err(|_| DocxError::MalformedImage)?,
        );
        if png_crc(&bytes[offset + 4..data_end]) != expected_crc {
            return Err(DocxError::MalformedImage);
        }
        match chunk_type {
            b"IHDR" => {
                if chunk_index != 0 || length != 13 {
                    return Err(DocxError::MalformedImage);
                }
                width = u32::from_be_bytes(
                    bytes[data_start..data_start + 4]
                        .try_into()
                        .map_err(|_| DocxError::MalformedImage)?,
                );
                height = u32::from_be_bytes(
                    bytes[data_start + 4..data_start + 8]
                        .try_into()
                        .map_err(|_| DocxError::MalformedImage)?,
                );
                if width == 0
                    || height == 0
                    || bytes[data_start + 10] != 0
                    || bytes[data_start + 11] != 0
                    || bytes[data_start + 12] > 1
                    || !valid_png_color_depth(bytes[data_start + 8], bytes[data_start + 9])
                {
                    return Err(DocxError::MalformedImage);
                }
            }
            b"PLTE" | b"tRNS" => {
                if chunk_index == 0 || saw_idat || saw_iend {
                    return Err(DocxError::MalformedImage);
                }
            }
            b"IDAT" => {
                if chunk_index == 0 || saw_iend {
                    return Err(DocxError::MalformedImage);
                }
                saw_idat = true;
            }
            b"IEND" => {
                if length != 0 || !saw_idat || saw_iend || chunk_end != bytes.len() {
                    return Err(DocxError::MalformedImage);
                }
                saw_iend = true;
            }
            _ if chunk_type[0] & 0x20 != 0 => {
                return Err(DocxError::ImageMetadataForbidden);
            }
            _ => return Err(DocxError::MalformedImage),
        }
        chunk_index = chunk_index
            .checked_add(1)
            .ok_or(DocxError::MalformedImage)?;
        offset = chunk_end;
    }
    if !saw_iend || width == 0 || height == 0 {
        return Err(DocxError::MalformedImage);
    }
    Ok(ImageInfo {
        width,
        height,
        extension: "png",
    })
}

fn valid_png_color_depth(depth: u8, color_type: u8) -> bool {
    match color_type {
        0 => matches!(depth, 1 | 2 | 4 | 8 | 16),
        2 => matches!(depth, 8 | 16),
        3 => matches!(depth, 1 | 2 | 4 | 8),
        4 | 6 => matches!(depth, 8 | 16),
        _ => false,
    }
}

fn png_crc(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffff_u32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = 0_u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}

fn validate_jpeg(bytes: &[u8]) -> Result<ImageInfo, DocxError> {
    if bytes.len() < 4 || !bytes.ends_with(&[0xff, 0xd9]) {
        return Err(DocxError::MalformedImage);
    }
    let mut offset = 2_usize;
    let mut dimensions = None;
    let mut saw_scan = false;
    while offset + 1 < bytes.len() {
        if bytes[offset] != 0xff {
            return Err(DocxError::MalformedImage);
        }
        while offset < bytes.len() && bytes[offset] == 0xff {
            offset += 1;
        }
        if offset >= bytes.len() {
            return Err(DocxError::MalformedImage);
        }
        let marker = bytes[offset];
        offset += 1;
        match marker {
            0xd9 => {
                if offset != bytes.len() || !saw_scan {
                    return Err(DocxError::MalformedImage);
                }
                break;
            }
            0xd8 | 0x00 | 0x01 | 0xd0..=0xd7 => return Err(DocxError::MalformedImage),
            0xe1..=0xef | 0xfe => return Err(DocxError::ImageMetadataForbidden),
            _ => {
                let length_end = offset.checked_add(2).ok_or(DocxError::MalformedImage)?;
                if length_end > bytes.len() {
                    return Err(DocxError::MalformedImage);
                }
                let segment_length = usize::from(u16::from_be_bytes(
                    bytes[offset..length_end]
                        .try_into()
                        .map_err(|_| DocxError::MalformedImage)?,
                ));
                if segment_length < 2 {
                    return Err(DocxError::MalformedImage);
                }
                let segment_end = offset
                    .checked_add(segment_length)
                    .ok_or(DocxError::MalformedImage)?;
                if segment_end > bytes.len() {
                    return Err(DocxError::MalformedImage);
                }
                if marker == 0xe0
                    && (segment_length != 16
                        || &bytes[offset + 2..offset + 7] != b"JFIF\0"
                        || bytes[offset + 14] != 0
                        || bytes[offset + 15] != 0)
                {
                    return Err(DocxError::ImageMetadataForbidden);
                }
                if matches!(marker, 0xc0..=0xc3 | 0xc5..=0xc7 | 0xc9..=0xcb | 0xcd..=0xcf) {
                    if segment_length < 8 {
                        return Err(DocxError::MalformedImage);
                    }
                    let height = u32::from(u16::from_be_bytes(
                        bytes[offset + 3..offset + 5]
                            .try_into()
                            .map_err(|_| DocxError::MalformedImage)?,
                    ));
                    let width = u32::from(u16::from_be_bytes(
                        bytes[offset + 5..offset + 7]
                            .try_into()
                            .map_err(|_| DocxError::MalformedImage)?,
                    ));
                    if width == 0 || height == 0 || dimensions.replace((width, height)).is_some() {
                        return Err(DocxError::MalformedImage);
                    }
                }
                if marker == 0xda {
                    saw_scan = true;
                    offset = segment_end;
                    while offset + 1 < bytes.len() {
                        if bytes[offset] == 0xff {
                            let next = bytes[offset + 1];
                            if next == 0xd9 {
                                offset += 2;
                                break;
                            }
                            if next != 0x00 && !(0xd0..=0xd7).contains(&next) {
                                return Err(DocxError::MalformedImage);
                            }
                            offset += 2;
                        } else {
                            offset += 1;
                        }
                    }
                    if offset != bytes.len() {
                        return Err(DocxError::MalformedImage);
                    }
                    break;
                }
                offset = segment_end;
            }
        }
    }
    let (width, height) = dimensions.ok_or(DocxError::MalformedImage)?;
    if !saw_scan {
        return Err(DocxError::MalformedImage);
    }
    Ok(ImageInfo {
        width,
        height,
        extension: "jpg",
    })
}
