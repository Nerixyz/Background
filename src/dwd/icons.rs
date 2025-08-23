use std::path::PathBuf;

pub struct Msn;

pub trait IconSet {
    fn present_weather_to_path(pw: u16, is_night: bool) -> PathBuf;
    fn significant_weather_to_path(sw: u16, cloud_cover: f32, is_night: bool) -> PathBuf;
}

// https://www.dwd.de/DE/leistungen/opendata/help/schluessel_datenformate/csv/poi_present_weather_zuordnung_pdf.pdf
impl IconSet for super::icons::Msn {
    fn present_weather_to_path(pw: u16, is_night: bool) -> PathBuf {
        use MsnIcon::*;

        let (day, night) = match pw {
            2 => (MostlySunnyDay, MostlyClearNight),
            3 => (D200PartlySunnyV2, PartlyCloudyNightV2),
            4 => (CloudyV3, CloudyV3),
            5 => (FogV2, FogV2),
            6 => (FogV2, FogV2),
            7 => (LightRainV3, LightRainV3),
            8 => (HeavyDrizzle, HeavyDrizzle),
            9 => (ModerateRainV2, ModerateRainV2),
            10 => (FreezingRainV2, FreezingRainV2),
            11 => (FreezingRainV2, FreezingRainV2),
            12 => (RainSnowV2, RainSnowV2),
            13 => (RainSnowV2, RainSnowV2),
            14 => (LightSnowV2, LightSnowV2),
            15 => (Snow, Snow),
            16 => (HeavySnowV2, HeavySnowV2),
            17 => (IcePelletsV2, IcePelletsV2),
            18 => (LightRainShowerDay, LightRainShowerNight),
            19 => (RainShowersDayV2, RainShowersNightV2),
            20 => (D221RainSnowShowersV2, N221RainSnowShowersV2),
            21 => (D321RainSnowShowersV2, N321RainSnowShowersV2),
            22 => (LightSnowShowersDay, LightSnowShowersNight),
            23 => (SnowShowersDayV2, N222SnowShowersV2),
            24 => (IcePelletsV2, IcePelletsV2),
            25 => (IcePelletsV2, IcePelletsV2),
            26 => (ThunderstormsV2, ThunderstormsV2),
            27 => (D240TstormsV2, N240TstormsV2),
            28 => (D340TstormsV2, N340TstormsV2),
            29 => (ThunderstormsV2, ThunderstormsV2),
            30 => (ThunderstormsV2, ThunderstormsV2),
            31 => (WindyV2, WindyV2),
            1 | _ => (SunnyDayV3, ClearNightV3),
        };

        if is_night { night } else { day }.to_path().into()
    }

    fn significant_weather_to_path(sw: u16, cloud_cover: f32, is_night: bool) -> PathBuf {
        use MsnIcon::*;

        let (day, night) = match sw {
            10 => (Haze, HazeNight),
            11 | 12 | 28 | 41 | 43 | 45 | 47 | 49 | 120 | 130..=135 | 247..=249 => (FogV2, FogV2),
            18 | 118 | 127..=129 | 199 => (WindyV2, WindyV2),
            24 => (FreezingRainV2, FreezingRainV2),
            20
            | 21
            | 50..=52
            | 58
            | 60..=62
            | 80
            | 87
            | 150..=153
            | 161
            | 181
            | 250..=252
            | 260
            | 292
            | 293 => with_cloud_total(
                cloud_cover,
                (D210LightRainShowersV2, N210LightRainShowersV2),
                (D310LightRainShowersV2, D310LightRainShowersV2),
                (LightRainV3, LightRainV3),
            ),
            25
            | 53..=55
            | 59
            | 63..=65
            | 81
            | 82
            | 121..=123
            | 140..=146
            | 157
            | 158
            | 160
            | 162
            | 163
            | 180
            | 182..=184
            | 253..=257
            | 261..=267
            | 280
            | 281
            | 284
            | 285
            | 289 => with_cloud_total(
                cloud_cover,
                (RainShowersDayV2, RainShowersNightV2),
                (RainShowersDayV2, RainShowersNightV2),
                match sw {
                    55 | 65 | 82 | 162 | 253 => (ModerateRainV2, ModerateRainV2),
                    _ => (HeavyDrizzle, HeavyDrizzle),
                },
            ),
            56 | 57 | 125 | 147 | 148 => (FreezingRainV2, FreezingRainV2),
            70..=72 | 77 => with_cloud_total(
                cloud_cover,
                (D212LightSnowShowersV2, N212LightSnowShowersV2),
                (LightSnowShowersDay, LightSnowShowersNight),
                (LightSnowV2, LightSnowV2),
            ),
            22
            | 36..=39
            | 73..=79
            | 124
            | 170..=173
            | 177
            | 178
            | 270..=277
            | 279
            | 283
            | 287
            | 291 => with_cloud_total(
                cloud_cover,
                (SnowShowersDayV2, N322SnowShowersV2),
                (SnowShowersDayV2, N322SnowShowersV2),
                if matches!(sw, 37 | 39 | 74 | 75) {
                    (HeavySnowV2, HeavySnowV2)
                } else {
                    (Snow, Snow)
                },
            ),
            23
            | 26
            | 66
            | 67..=69
            | 83..=86
            | 154
            | 156
            | 167
            | 168
            | 185..=187
            | 259
            | 282
            | 286
            | 290 => with_cloud_total(
                cloud_cover,
                (D221RainSnowShowersV2, N221RainSnowShowersV2),
                (D221RainSnowShowersV2, N221RainSnowShowersV2),
                (RainSnowV2, RainSnowV2),
            ),
            88..=90 | 111 | 174..=176 | 278 => (IcePelletsV2, IcePelletsV2),
            17 | 29 | 92..=99 | 190..=196 | 217 => with_cloud_total4(
                cloud_cover,
                (Lightning, Lightning),
                (D240TstormsV2, N240TstormsV2),
                (D340TstormsV2, N340TstormsV2),
                (ThunderstormsV2, ThunderstormsV2),
            ),
            27 | 30..=35 | 189 | 288 => (BlowingHailV2, BlowingHailV2),

            // catch-alls
            _ => match cloud_cover {
                ..1.0 => (SunnyDayV3, ClearNightV3),
                ..30.0 => (MostlySunnyDay, MostlyClearNight),
                ..60.0 => (D200PartlySunnyV2, PartlyCloudyNightV2),
                ..90.0 => (MostlyCloudyDayV2, MostlyCloudyNightV2),
                _ => (CloudyV3, CloudyV3),
            },
        };

        if is_night { night } else { day }.to_path().into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, derive_more::Display)]
#[allow(unused)]
enum MsnIcon {
    BlowingHailV2,
    ClearNightV3,
    CloudyV3,
    D200PartlySunnyV2,
    D210LightRainShowersV2,
    D211LightRainSowShowersV2,
    D212LightSnowShowersV2,
    D221RainSnowShowersV2,
    D240TstormsV2,
    D310LightRainShowersV2,
    D311LightRainSnowShowersV2,
    D321RainSnowShowersV2,
    D340TstormsV2,
    FogV2,
    FreezingRainV2,
    Haze,
    HazeNight,
    HeavyDrizzle,
    HeavySnowV2,
    IcePelletsV2,
    Lightning,
    LightRainShowerDay,
    LightRainShowerNight,
    LightRainV3,
    LightSnowShowersDay,
    LightSnowShowersNight,
    LightSnowV2,
    ModerateRainV2,
    MostlyClearNight,
    MostlyCloudyDayV2,
    MostlyCloudyNightV2,
    MostlySunnyDay,
    N210LightRainShowersV2,
    N211LightRainSnowShowersV2,
    N212LightSnowShowersV2,
    N221RainSnowShowersV2,
    N222SnowShowersV2,
    N240TstormsV2,
    N310LightRainShowersV2,
    N311LightRainSnowShowersV2,
    N321RainSnowShowersV2,
    N322SnowShowersV2,
    N340TstormsV2,
    N422SnowV2,
    PartlyCloudyNightV2,
    RainShowersDayV2,
    RainShowersNightV2,
    RainSnowShowersNightV2,
    RainSnowV2,
    Snow,
    SnowShowersDayV2,
    SunnyDayV3,
    ThunderstormsV2,
    WindyV2,
}

impl MsnIcon {
    pub fn to_path(self) -> String {
        format!("icons/msn/{self}.svg")
    }
}

fn with_cloud_total<T>(total: f32, light: T, mid: T, full: T) -> T {
    if total <= 43.75 {
        light
    } else if total <= 81.25 {
        mid
    } else {
        full
    }
}

fn with_cloud_total4<T>(total: f32, none: T, light: T, mid: T, full: T) -> T {
    if total < 0.1 {
        none
    } else if total <= 0.4375 {
        light
    } else if total <= 0.8125 {
        mid
    } else {
        full
    }
}
