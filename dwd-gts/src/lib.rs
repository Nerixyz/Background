use anyhow::bail;
use atoi::atoi;
use tinystr::TinyAsciiStr;

#[derive(Debug)]
pub struct GtsHeader {
    pub seq_no: u16,
    pub product_id: TinyAsciiStr<6>,
    pub source: TinyAsciiStr<4>,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
}

impl GtsHeader {
    // from https://www.eumetnet.eu/wp-content/uploads/2025/05/OPERA_bufr_sw_desc.pdf
    // and https://rom-saf.eumetsat.int/romsaf_bufr.pdf
    // <SOH><CR><CR><LF>nnn<CR><CR><LF>T_1 T_2 A_1 A_2 ii<SP>cccc<SP>YYGGgg<CR><CR><LF>
    pub fn read(header: &[u8; 35]) -> anyhow::Result<Self> {
        if &header[..4] != b"\x01\r\r\n" {
            bail!("invalid start of GTS message");
        }
        let Some(seq_no) = atoi(&header[4..7]) else {
            bail!("invalid seq no");
        };
        if &header[7..10] != b"\r\r\n" {
            bail!("expected <cr><cr><lf>");
        }
        let product_id = TinyAsciiStr::<6>::try_from_utf8(&header[10..16])?;
        if header[16] != b' ' {
            bail!("expected space after product");
        }
        let source = TinyAsciiStr::<4>::try_from_utf8(&header[17..21])?;
        if header[21] != b' ' {
            bail!("expected space after source");
        }
        let Some(day) = atoi(&header[22..24]) else {
            bail!("invalid day");
        };
        let Some(hour) = atoi(&header[24..26]) else {
            bail!("invalid hour");
        };
        let Some(minute) = atoi(&header[26..28]) else {
            bail!("invalid minute");
        };

        Ok(Self {
            seq_no,
            product_id,
            source,
            day,
            hour,
            minute,
        })
    }
}
