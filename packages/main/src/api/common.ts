/* eslint-disable @typescript-eslint/no-explicit-any */
import { ArrToObj, CommonWeatherData } from '../types';

export function dataArraysToObjectsUnsafe<In extends Record<string, any[]>>(data: In): Array<ArrToObj<In>> {
  const arr: any[] = [];
  for (const [key, values] of Object.entries(data) as any) {
    for (let i = 0; i < values.length; i++) {
      if (!arr[i]) arr[i] = {};
      arr[i][key] = values[i];
    }
  }

  return arr;
}

export function findFirstValidTs(data: CommonWeatherData) {
  const now = Date.now();
  let idx = 0;
  for (let i = 0; i < data.timestamp.length; i++) {
    if (data.timestamp[i] - now > 0) break;
    idx = i;
  }
  return idx;
}
