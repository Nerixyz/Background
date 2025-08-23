use std::{
    io::{self, Read},
    sync::RwLock,
};

use anyhow::bail;

use crate::dwd::{Cache, RadarReading, ZONE, get_etag, needs_fetch};

const STATIC_HEADER_LEN: usize = 91;
const DATE_0_OFFSET: usize = 2;
const DATE_1_OFFSET: usize = 13;
const PR_OFFSET: usize = 46 + 3; // skip ' E-'
const INT_OFFSET: usize = 54;
const VV_OFFSET: usize = 71;

const URL: &str = "https://opendata.dwd.de/weather/radar/composite/rv/DE1200_RV_LATEST.tar.bz2";

pub fn get(cache: &RwLock<Cache>, target: (usize, usize)) -> anyhow::Result<bool> {
    if !needs_fetch(URL, cache.read().unwrap().radar_etag.as_deref()) {
        return Ok(false);
    }

    let mut res = ureq::get(URL).call()?;
    if !res.status().is_success() {
        bail!("Failed to get radar - got status {:?}", res.status());
    }
    let etag = get_etag(&res);

    let reader = bzip2::read::BzDecoder::new(res.body_mut().as_reader());

    let mut ar = tar::Archive::new(reader);
    let mut values = Vec::new();
    for e in ar.entries()?.filter_map(Result::ok) {
        values.push(read_rv(e, target)?);
    }
    values.sort_unstable_by_key(|v| v.timestamp);
    let mut cache = cache.write().unwrap();
    cache.radar_etag = etag;
    cache.radar = values;

    Ok(true)
}

// https://www.dwd.de/DE/leistungen/radarprodukte/formatbeschreibung_rv.pdf?__blob=publicationFile&v=3
fn read_rv(
    mut reader: impl Read,
    (target_x, target_y): (usize, usize),
) -> anyhow::Result<RadarReading> {
    let mut header = [0; STATIC_HEADER_LEN];
    reader.read_exact(&mut header)?;
    let len = std::str::from_utf8(&header[STATIC_HEADER_LEN - 3..])?
        .trim()
        .parse::<u64>()?;
    // discard string
    io::copy(&mut reader.by_ref().take(len), &mut io::sink())?;
    let mut header_end = [0; 1];
    reader.read_exact(&mut header_end)?;
    if header_end[0] != 0x3 {
        bail!("Invalid header");
    }

    let (Some(day), Some(hour), Some(minute), Some(month), Some(year)) = (
        atoi::atoi::<i8>(&header[DATE_0_OFFSET..DATE_0_OFFSET + 2]),
        atoi::atoi::<i8>(&header[DATE_0_OFFSET + 2..DATE_0_OFFSET + 4]),
        atoi::atoi::<i8>(&header[DATE_0_OFFSET + 4..DATE_0_OFFSET + 6]),
        atoi::atoi::<i8>(&header[DATE_1_OFFSET..DATE_1_OFFSET + 2]),
        atoi::atoi::<i16>(&header[DATE_1_OFFSET + 2..DATE_1_OFFSET + 4]),
    ) else {
        bail!("Invalid date")
    };
    let (Some(precision), Some(interval), Some(offset_min)) = (
        atoi::atoi::<u8>(&header[PR_OFFSET..PR_OFFSET + 2]),
        atoi::atoi::<u8>(&header[INT_OFFSET..INT_OFFSET + 4].trim_ascii_start()),
        atoi::atoi::<u8>(&header[VV_OFFSET..VV_OFFSET + 4].trim_ascii_start()),
    ) else {
        bail!("invalid other numbers")
    };
    let dt = jiff::civil::datetime(year + 2000, month, day, hour, minute, 0, 0);
    let timestamp =
        jiff::tz::Offset::constant(0).to_timestamp(dt)? + jiff::Span::new().minutes(offset_min);
    let local_ts = timestamp.to_zoned(ZONE.clone());

    // the data is encoded from south to north and west to east
    let mut buf = vec![0u16; 1200 * 1100];
    reader.read_exact(bytemuck::cast_slice_mut::<_, [u8; 2]>(&mut buf).as_flattened_mut())?;
    fn xy_to_idx(x: usize, y: usize) -> usize {
        (1200 - y - 1) * 1100 + x
    }
    // let mut img = image::RgbaImage::new(1100, 1200);
    // for y in 0..1200 {
    //     for x in 0..1100 {
    //         let idx = xy_to_idx(x, y);
    //         let v = buf[idx];
    //         img.put_pixel(
    //             x as u32,
    //             y as u32,
    //             if v > 0 {
    //                 image::Rgba([255, 255, 255, 255])
    //             } else {
    //                 image::Rgba([0, 0, 0, 0])
    //             },
    //         );
    //     }
    // }
    // img.save(&format!("x-radar-{}.png", timestamp.as_second()))?;

    // read our target value
    // average on 3x3km with bias for the target square
    let mut sum = 0u64;
    let target_idx = xy_to_idx(target_x, target_y);
    for y in target_y - 1..target_y + 2 {
        for x in target_x - 1..target_x + 2 {
            let idx = xy_to_idx(x, y);
            if idx != target_idx {
                sum += buf[idx] as u64;
            }
        }
    }
    sum += 8 * buf[target_idx] as u64;
    let val = sum as f32 / 16.0;

    let p_factor = match precision {
        0 => 1.0,
        1 => 0.1,
        2 => 0.01,
        x => 10.0f32.powi(-(x as i32)),
    };
    let value = val * p_factor * (60 / interval) as f32;

    Ok(RadarReading {
        timestamp,
        local_ts,
        value,
    })
}
