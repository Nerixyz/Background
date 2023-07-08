import { ReportDataEl, StationReport } from './dwd-api.types';
// @ts-ignore
import * as windyImages from '../../../shared/assets/weather/icons/windy/**/*.svg';
// @ts-ignore
import * as msnImages from '../../../shared/assets/weather/icons/msn/*.svg';
import { MsnIcon } from 'shared/types';

export async function getReport(): Promise<ReportDataEl[]> {
  console.log(process.env.STATION_ID);
  const res: StationReport = await fetch(`https://weatherapi.nerixyz.de/report/${process.env.STATION_ID}`).then(x =>
    x.json(),
  );

  return res.data.sort((a, b) => a.timestamp - b.timestamp);
}

export function getIcon(el: ReportDataEl): string {
  const hour = new Date(el.timestamp).getHours();
  const isNight = hour < 7 || hour > 20; // TODO: use some kind of sunset data

  if (process.env.ICON_SET === 'msn') {
    return getMsnIcon(el.present_weather ?? 1, isNight);
  }
  return getWindyIcon(el.present_weather ?? 1, isNight);
}

function getWindyIcon(presentWeather: number, isNight: boolean): string {
  const iconId = DWD_TO_ICON_MAP[presentWeather as keyof typeof DWD_TO_ICON_MAP] ?? 1;
  return windyImages[isNight ? 'night' : 'day']?.[iconId];
}

function getMsnIcon(presentWeather: number, isNight: boolean): string {
  const [icoDay, icoNight] = DWD_TO_MSN_MAP[presentWeather as keyof typeof DWD_TO_MSN_MAP] ?? DWD_TO_MSN_MAP[1];
  return msnImages[isNight ? icoNight : icoDay];
}

// https://www.dwd.de/DE/leistungen/opendata/help/schluessel_datenformate/csv/poi_present_weather_zuordnung_pdf.pdf?__blob=publicationFile&v=2
const DWD_TO_ICON_MAP = {
  1: 1,
  2: 1,
  3: 3,
  4: 4,
  5: 17,
  6: 17,
  7: 20,
  8: 7,
  9: 7,
  10: 13,
  11: 13,
  12: 13,
  13: 12,
  14: 10,
  15: 10,
  16: 10,
  18: 20,
  19: 20,
  20: 13,
  21: 13,
  22: 13,
  23: 13,
  24: 13,
  25: 13,
  26: 14,
  27: 14,
  28: 14,
  29: 14,
  30: 14,
  31: 4,
};

const DWD_TO_MSN_MAP: Record<number, [MsnIcon, MsnIcon]> = {
  1: ['SunnyDayV3', 'ClearNightV3'],
  2: ['MostlySunnyDay', 'MostlyClearNight'],
  3: ['D200PartlySunnyV2', 'PartlyCloudyNightV2'],
  4: ['CloudyV3', 'CloudyV3'],
  5: ['FogV2', 'FogV2'],
  6: ['FogV2', 'FogV2'],
  7: ['LightRainV3', 'LightRainV3'],
  8: ['HeavyDrizzle', 'HeavyDrizzle'],
  9: ['ModerateRainV2', 'ModerateRainV2'],
  10: ['FreezingRainV2', 'FreezingRainV2'],
  11: ['FreezingRainV2', 'FreezingRainV2'],
  12: ['RainSnowV2', 'RainSnowV2'],
  13: ['RainSnowV2', 'RainSnowV2'],
  14: ['LightSnowV2', 'LightSnowV2'],
  15: ['Snow', 'Snow'],
  16: ['HeavySnowV2', 'HeavySnowV2'],
  17: ['IcePelletsV2', 'IcePelletsV2'],
  18: ['LightRainShowerDay', 'LightRainShowerNight'],
  19: ['RainShowersDayV2', 'RainShowersNightV2'],
  20: ['D221RainSnowShowersV2', 'N221RainSnowShowersV2'],
  21: ['D321RainSnowShowersV2', 'N321RainSnowShowersV2'],
  22: ['LightSnowShowersDay', 'LightSnowShowersNight'],
  23: ['SnowShowersDayV2', 'N222SnowShowersV2'],
  24: ['IcePelletsV2', 'IcePelletsV2'],
  25: ['IcePelletsV2', 'IcePelletsV2'],
  26: ['ThunderstormsV2', 'ThunderstormsV2'],
  27: ['D240TstormsV2', 'N240TstormsV2'],
  28: ['D340TstormsV2', 'N340TstormsV2'],
  29: ['ThunderstormsV2', 'ThunderstormsV2'],
  30: ['ThunderstormsV2', 'ThunderstormsV2'],
  31: ['WindyV2', 'WindyV2'],
};
