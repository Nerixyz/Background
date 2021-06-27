// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type ArrToObj<T extends Record<string, any[]>> = { [x in keyof T]: T[x] extends Array<infer T> ? T : never };

export type CommonWeatherData = {
  temp: number[];
  rain: number[];
  timestamp: number[];
  icon: number[];
};

export type CommonWeatherObj = ArrToObj<CommonWeatherData>;
