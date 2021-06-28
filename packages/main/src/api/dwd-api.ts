import { ForecastResponse, ForecastResponseData } from './dwd-api.types';
import { CommonWeatherData } from '../types';

export async function getDwdCommonWeather(): Promise<CommonWeatherData> {
  const base = await getForecast();
  return dwdToCommon(base);
}

async function getForecast(): Promise<ForecastResponseData> {
  const res: ForecastResponse = await fetch(
    `https://weatherapi.nerixyz.de/forecast/${import.meta.env.VITE_APP_STATION}`,
  ).then(x => x.json());

  return res.data;
}

function dwdToCommon(dwd: ForecastResponseData): CommonWeatherData {
  return {
    temp: (dwd.temp ?? []).map(x => (x === null ? 0 : x)),
    rain: (dwd.precipitation_1h_significant_weather ?? []).map(x => (x === null ? 0 : x)),
    timestamp: dwd.time_steps,
    icon: getIcons(dwd),
  };
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
function getIcons(forecast: ForecastResponseData): number[] {
  const getSafe = (idx: number, key: keyof ForecastResponseData, base = 0): number => forecast[key]?.[idx] ?? base;

  return forecast.time_steps.map((_, i) => {
    const cloudCover = getSafe(i, 'total_cloud_cover') / 100;
    const sig = getSafe(i, 'significant_weather');

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
  });
}

/**
 * See page 13 on https://www.dwd.de/DE/forschung/wettervorhersage/num_modellierung/01_num_vorhersagemodelle/01c_wetterinterpretation/wetter_interpretation.pdf?__blob=publicationFile&v=6
 * @param {number} total
 * @param {number} light
 * @param {number} mid
 * @param {number} full
 * @returns {number}
 */
function withCloudTotal(total: number, light: number, mid: number, full: number): number {
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
