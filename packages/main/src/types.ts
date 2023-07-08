import { MsnIcon } from 'shared/types';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type ArrToObj<T extends Record<string, any[]>> = { [x in keyof T]: T[x] extends Array<infer T> ? T : never };

export interface CommonIcon {
  windy: number;
  msn: [MsnIcon, MsnIcon];
}

export type CommonWeatherData = {
  temp: number[];
  rain: number[];
  timestamp: number[];
  icon: CommonIcon[];
};

export type CommonWeatherResponse = [CommonWeatherData, Date];

export type CommonWeatherObj = ArrToObj<CommonWeatherData>;
