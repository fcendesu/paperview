use std::{fs, path::Path};

#[must_use]
pub fn alt_text(source: &str) -> String {
    source.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[must_use]
pub fn markdown_text(alt: &str, url: &str, title: &str) -> String {
    if title.is_empty() {
        format!("![{alt}]({url})")
    } else {
        format!("![{alt}]({url} \"{title}\")")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageDimensions {
    pub width: u32,
    pub height: u32,
}

impl ImageDimensions {
    #[must_use]
    pub fn label(self) -> String {
        format!("{} x {} px", self.width, self.height)
    }
}

#[must_use]
pub fn dimensions_from_path(path: &Path) -> Option<ImageDimensions> {
    let bytes = fs::read(path).ok()?;
    dimensions_from_bytes(&bytes)
}

#[must_use]
pub fn dimensions_from_bytes(bytes: &[u8]) -> Option<ImageDimensions> {
    png_dimensions(bytes)
        .or_else(|| gif_dimensions(bytes))
        .or_else(|| jpeg_dimensions(bytes))
        .or_else(|| webp_dimensions(bytes))
}

fn png_dimensions(bytes: &[u8]) -> Option<ImageDimensions> {
    const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";

    if bytes.len() < 24 || &bytes[..8] != PNG_SIGNATURE || &bytes[12..16] != b"IHDR" {
        return None;
    }

    Some(ImageDimensions {
        width: u32::from_be_bytes(bytes[16..20].try_into().ok()?),
        height: u32::from_be_bytes(bytes[20..24].try_into().ok()?),
    })
}

fn gif_dimensions(bytes: &[u8]) -> Option<ImageDimensions> {
    if bytes.len() < 10 || (&bytes[..6] != b"GIF87a" && &bytes[..6] != b"GIF89a") {
        return None;
    }

    Some(ImageDimensions {
        width: u16::from_le_bytes(bytes[6..8].try_into().ok()?).into(),
        height: u16::from_le_bytes(bytes[8..10].try_into().ok()?).into(),
    })
}

fn jpeg_dimensions(bytes: &[u8]) -> Option<ImageDimensions> {
    if bytes.len() < 4 || bytes[0] != 0xff || bytes[1] != 0xd8 {
        return None;
    }

    let mut cursor = 2;
    while cursor + 9 < bytes.len() {
        while cursor < bytes.len() && bytes[cursor] != 0xff {
            cursor += 1;
        }
        while cursor < bytes.len() && bytes[cursor] == 0xff {
            cursor += 1;
        }
        if cursor >= bytes.len() {
            return None;
        }

        let marker = bytes[cursor];
        cursor += 1;

        if marker == 0xd9 || marker == 0xda {
            return None;
        }
        if matches!(marker, 0x01 | 0xd0..=0xd7) {
            continue;
        }
        if cursor + 2 > bytes.len() {
            return None;
        }

        let segment_length = usize::from(u16::from_be_bytes(
            bytes[cursor..cursor + 2].try_into().ok()?,
        ));
        if segment_length < 2 || cursor + segment_length > bytes.len() {
            return None;
        }

        if matches!(
            marker,
            0xc0 | 0xc1
                | 0xc2
                | 0xc3
                | 0xc5
                | 0xc6
                | 0xc7
                | 0xc9
                | 0xca
                | 0xcb
                | 0xcd
                | 0xce
                | 0xcf
        ) {
            if segment_length < 7 {
                return None;
            }

            return Some(ImageDimensions {
                height: u16::from_be_bytes(bytes[cursor + 3..cursor + 5].try_into().ok()?).into(),
                width: u16::from_be_bytes(bytes[cursor + 5..cursor + 7].try_into().ok()?).into(),
            });
        }

        cursor += segment_length;
    }

    None
}

fn webp_dimensions(bytes: &[u8]) -> Option<ImageDimensions> {
    if bytes.len() < 30 || &bytes[..4] != b"RIFF" || &bytes[8..12] != b"WEBP" {
        return None;
    }

    match &bytes[12..16] {
        b"VP8 " if bytes.len() >= 30 && &bytes[23..26] == b"\x9d\x01\x2a" => {
            Some(ImageDimensions {
                width: u32::from(u16::from_le_bytes(bytes[26..28].try_into().ok()?) & 0x3fff),
                height: u32::from(u16::from_le_bytes(bytes[28..30].try_into().ok()?) & 0x3fff),
            })
        }
        b"VP8L" if bytes.len() >= 25 && bytes[20] == 0x2f => {
            let b1 = u32::from(bytes[21]);
            let b2 = u32::from(bytes[22]);
            let b3 = u32::from(bytes[23]);
            let b4 = u32::from(bytes[24]);

            Some(ImageDimensions {
                width: 1 + (((b2 & 0x3f) << 8) | b1),
                height: 1 + (((b4 & 0x0f) << 10) | (b3 << 2) | ((b2 & 0xc0) >> 6)),
            })
        }
        b"VP8X" if bytes.len() >= 30 => Some(ImageDimensions {
            width: 1 + u32::from_le_bytes([bytes[24], bytes[25], bytes[26], 0]),
            height: 1 + u32::from_le_bytes([bytes[27], bytes[28], bytes[29], 0]),
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{ImageDimensions, dimensions_from_bytes, dimensions_from_path};

    #[test]
    fn reads_png_dimensions_from_bytes_and_path() {
        let png = [
            0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n', 0, 0, 0, 13, b'I', b'H', b'D', b'R',
            0, 0, 1, 0x40, 0, 0, 0, 0xf0, 8, 2, 0, 0, 0,
        ];
        let dir = temp_dir("image-dimensions");
        let path = dir.join("preview.png");

        fs::create_dir_all(&dir).expect("create temp dir");
        fs::write(&path, png).expect("write png header");

        let dimensions = dimensions_from_bytes(&png).expect("png dimensions");
        assert_eq!(
            dimensions,
            ImageDimensions {
                width: 320,
                height: 240
            }
        );
        assert_eq!(dimensions.label(), "320 x 240 px");
        assert_eq!(dimensions_from_path(&path), Some(dimensions));

        fs::remove_dir_all(dir).expect("remove temp dir");
    }

    #[test]
    fn reads_gif_jpeg_and_webp_dimensions_from_bytes() {
        let gif = [b'G', b'I', b'F', b'8', b'9', b'a', 0x40, 0x01, 0xf0, 0x00];
        let jpeg = [
            0xff, 0xd8, 0xff, 0xe0, 0x00, 0x04, 0x00, 0x00, 0xff, 0xc0, 0x00, 0x11, 0x08, 0x00,
            0xf0, 0x01, 0x40, 0x03, 0x01, 0x11, 0x00, 0x02, 0x11, 0x00, 0x03, 0x11, 0x00,
        ];
        let webp = [
            b'R', b'I', b'F', b'F', 22, 0, 0, 0, b'W', b'E', b'B', b'P', b'V', b'P', b'8', b'X',
            10, 0, 0, 0, 0, 0, 0, 0, 0x3f, 0x01, 0, 0xef, 0, 0,
        ];

        assert_eq!(
            dimensions_from_bytes(&gif),
            Some(ImageDimensions {
                width: 320,
                height: 240
            })
        );
        assert_eq!(
            dimensions_from_bytes(&jpeg),
            Some(ImageDimensions {
                width: 320,
                height: 240
            })
        );
        assert_eq!(
            dimensions_from_bytes(&webp),
            Some(ImageDimensions {
                width: 320,
                height: 240
            })
        );
        assert_eq!(dimensions_from_bytes(b"not an image"), None);
    }

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock after Unix epoch")
            .as_nanos();

        std::env::temp_dir().join(format!("paperview-core-{nanos}-{name}"))
    }
}
