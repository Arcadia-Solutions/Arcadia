use anyhow::{Result, bail};

pub fn format_title_group_name(series_name: Option<&str>, title_group_name: &str) -> String {
    match series_name {
        Some(series) => format!("{series} - {title_group_name}"),
        None => title_group_name.to_owned(),
    }
}

/// Encodes one byte into 2 ascii-encoded hex digits.
#[inline(always)]
pub fn hex_encode(char: u8) -> [u8; 2] {
    let char_1 = char >> 4;
    let char_2 = char & 0x0F;
    [
        match char_1 {
            0x0..=0x9 => char_1 + b'0',
            0xA..=0xF => char_1 - 0xA + b'A',
            _ => unreachable!(),
        },
        match char_2 {
            0x0..=0x9 => char_2 + b'0',
            0xA..=0xF => char_2 - 0xA + b'A',
            _ => unreachable!(),
        },
    ]
}

/// Decodes two ascii-encoded hex digits into one byte.
#[inline(always)]
pub fn hex_decode(chars: [u8; 2]) -> Result<u8> {
    Ok(match chars[0] {
        digit @ b'0'..=b'9' => (digit - b'0') << 4,
        lower @ b'a'..=b'f' => (lower - b'a' + 0xA) << 4,
        upper @ b'A'..=b'F' => (upper - b'A' + 0xA) << 4,
        _ => bail!("Invalid URL encoding."),
    } + match chars[1] {
        digit @ b'0'..=b'9' => digit - b'0',
        lower @ b'a'..=b'f' => lower - b'a' + 0xA,
        upper @ b'A'..=b'F' => upper - b'A' + 0xA,
        _ => bail!("Invalid URL encoding."),
    })
}
