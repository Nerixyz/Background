export interface WindyData {
  mm: number[];
  day: string[];
  hour: number[];
  ts: number[];
  origTs: number[];
  isDay: number[];
  moonPhase: number[];
  origDate: string[];
  icon: number[];
  icon2: number[];
  weathercode: string[];
  snowPrecip: number[];
  convPrecip: number[];
  rain: number[];
  snow: number[];
  temp: number[];
  dewPoint: number[];
  wind: number[];
  windDir: number[];
  rh: number[];
  gust: number[];
  pressure: number[];
  cbase: number[];
}
export interface WindyHeader {
  model: string;
  refTime: string;
  update: string;
  updateTs: number;
  elevation: number;
  origElevation: number;
  origLat: number;
  origLon: number;
  step: number;
  utcOffset: number;
  tzName: string;
  sunset: number;
  sunrise: number;
  daysAvail: number;
  cache: string;
  iconGlobalembedFog: string;
}

export interface WindyCelestial {
  TZname: string;
  TZoffset: number;
  TZoffsetFormatted: string;
  TZoffsetMin: number;
  TZabbrev: string;
  TZtype: string;
  nowObserved: string;
  night: string;
  sunset: string;
  sunrise: string;
  dusk: string;
  sunsetTs: number;
  sunriseTs: number;
  duskTs: number;
  isDay: boolean;
  atSea: number;
}

export interface WindySummary {
  icon: number;
  date: string;
  index: number;
  timestamp: number;
  weekday: string;
  day: number;
  tempMax: number;
  tempMin: number;
  wind: number;
  windDir: number;
  segments: number;
  icon2: number;
}

export interface WindyResponse {
  data: WindyData;
  header: WindyHeader;
  celestial: WindyCelestial;
  summary: Record<string, WindySummary>;
}
