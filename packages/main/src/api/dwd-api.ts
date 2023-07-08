import { ForecastResponse, ForecastResponseData } from './dwd-api.types';
import { CommonIcon, CommonWeatherResponse } from '../types';
import { MsnIcon } from 'shared/types';

export async function getDwdCommonWeather(): Promise<CommonWeatherResponse> {
  const base = await getForecast();
  return dwdToCommon(base);
}

async function getForecast(): Promise<ForecastResponse> {
  return await fetch(`https://weatherapi.nerixyz.de/forecast/${process.env.STATION_ID}`).then(x => x.json());
}

function dwdToCommon({ data: dwd, issue_time }: ForecastResponse): CommonWeatherResponse {
  return [
    {
      temp: (dwd.temp ?? []).map(x => (x === null ? 0 : x)),
      rain: (dwd.precipitation_1h_significant_weather ?? []).map(x => (x === null ? 0 : x)),
      timestamp: dwd.time_steps,
      icon: getIcons(dwd),
    },
    new Date(issue_time),
  ];
}

/**
 *
 * Icon meaning
 *
 * 1          sunny
 * 2,  3,  4  clouds
 * 18, 19, 20 c-rain (2)
 * 5,  6,  7  c-rain (3)
 * 8,  9,  10 c-snow
 * 11, 12, 13 snow + rain
 * 23, 21, 14 rain + L
 * 24, 26, 15 snow + L
 * 25, 27, 16 rain + snow + L
 * 17         Fog
 * 22         sun + fog
 * 40         wind
 *
 * @param {ForecastResponseData} forecast
 * @returns {number[]}
 */
function getIcons(forecast: ForecastResponseData): CommonIcon[] {
  const getSafe = (idx: number, key: keyof ForecastResponseData, base = 0): number => forecast[key]?.[idx] ?? base;

  return forecast.time_steps.map((_, i) => {
    const cloudCover = getSafe(i, 'total_cloud_cover') / 100;
    const sig = getSafe(i, 'significant_weather');

    return {
      windy: getWindyIcon(cloudCover, sig),
      msn: getMsnIcon(cloudCover, sig),
    };
  });
}

function getWindyIcon(cloudCover: number, sig: number): number {
  if (sig <= 1) {
    return 1;
  } else if (inRange(sig, 1, /* 3 */ 44)) {
    switch (sig) {
      case 1:
        return 2;
      case 2:
        return 3;
      // actually case 3 but just to support ww < 45
      default:
        return 3;
    }
  } else if (inRange(sig, 45, 50)) {
    return withCloudTotal(cloudCover, 22, 17, 17);
  } else if (inRange(sig, 51, 54) || inRange(sig, 61, 63) || sig === 80) {
    return withCloudTotal(cloudCover, 18, 19, 20);
  } else if (sig === 55 || inRange(sig, 64, 65) || inRange(sig, 81, 84)) {
    return withCloudTotal(cloudCover, 5, 6, 7);
  } else if (inRange(sig, 56, 60) || inRange(sig, 66, 70) || inRange(sig, 85, 90)) {
    return withCloudTotal(cloudCover, 11, 12, 13);
  } else if (inRange(sig, 71, 79)) {
    return withCloudTotal(sig, 8, 9, 10);
  } else if (inRange(sig, 90, 100)) {
    // No hail icon or only lightning
    return withCloudTotal(cloudCover, 23, 21, 14);
  } else {
    return 4;
  }
}

function getMsnIcon(cloudCover: number, sig: number): [MsnIcon, MsnIcon] {
  type Ic = [MsnIcon, MsnIcon];
  if (sig < 1) {
    return ['SunnyDayV3', 'ClearNightV3'];
  }
  if (inRange(sig, 1, /* 3 */ 44)) {
    switch (sig) {
      case 1:
        return ['MostlySunnyDay', 'MostlyClearNight'];
      case 2:
        return ['D200PartlySunnyV2', 'PartlyCloudyNightV2'];
      // actually case 3 but just to support ww < 45 (3 < ww < 45 isn't used)
      default:
        return ['MostlyCloudyDayV2', 'MostlyCloudyNightV2'];
    }
  }
  if (inRange(sig, 45, 50)) {
    return both('FogV2');
  }
  if (inRange(sig, 51, 52) || inRange(sig, 61, 62) || sig == 80) {
    return withCloudTotal<Ic>(
      cloudCover,
      ['D210LightRainShowersV2', 'N210LightRainShowersV2'],
      ['D310LightRainShowersV2', 'N310LightRainShowersV2'],
      both('LightRainV3'),
    );
  }
  if (inRange(sig, 53, 55) || inRange(sig, 63, 65) || inRange(sig, 81, 82)) {
    return withCloudTotal<Ic>(
      cloudCover,
      ['RainShowersDayV2', 'RainShowersNightV2'],
      ['RainShowersDayV2', 'RainShowersNightV2'],
      sig === 55 || sig === 65 || sig === 82 ? both('ModerateRainV2') : both('HeavyDrizzle'),
    );
  }
  if (inRange(sig, 56, 57)) {
    return both('FreezingRainV2');
  }
  if (inRange(sig, 70, 72) || sig === 77) {
    return withCloudTotal<Ic>(
      cloudCover,
      ['D212LightSnowShowersV2', 'N212LightSnowShowersV2'],
      ['LightSnowShowersDay', 'LightRainShowerNight'],
      both('LightSnowV2'),
    );
  }
  if (inRange(sig, 73, 75)) {
    return withCloudTotal<Ic>(
      cloudCover,
      ['SnowShowersDayV2', 'N322SnowShowersV2'],
      ['SnowShowersDayV2', 'N322SnowShowersV2'],
      sig < 75 ? both('Snow') : both('HeavySnowV2'),
    );
  }
  if (inRange(sig, 66, 67) || inRange(sig, 85, 86)) {
    return withCloudTotal<Ic>(
      cloudCover,
      ['D221RainSnowShowersV2', 'N221RainSnowShowersV2'],
      ['D221RainSnowShowersV2', 'N221RainSnowShowersV2'],
      both('RainSnowV2'),
    );
  }
  if (inRange(sig, 90, 100)) {
    return withCloudTotal<Ic>(
      cloudCover,
      ['D240TstormsV2', 'N240TstormsV2'],
      ['D340TstormsV2', 'N340TstormsV2'],
      both('ThunderstormsV2'),
    );
  }

  return both('CloudyV3');
}

function both<T>(val: T): [T, T] {
  return [val, val];
}

/**
 * See page 13 on https://www.dwd.de/DE/forschung/wettervorhersage/num_modellierung/01_num_vorhersagemodelle/01c_wetterinterpretation/wetter_interpretation.pdf?__blob=publicationFile&v=6
 */
function withCloudTotal<T>(total: number, light: T, mid: T, full: T): T {
  if (total <= 0.4375) {
    return light;
  } else if (total <= 0.8125) {
    return mid;
  } else {
    return full;
  }
}

function inRange(value: number, start: number, end: number): boolean {
  return value >= start && value <= end;
}
