import { CommonWeatherData } from '../types';

export let Windy = { version: '0', token: '', uid: '', token2: '' };

import { WindyData, WindyResponse } from './windy-api.types';

export async function getWindyCommonWeather(): Promise<CommonWeatherData> {
  if (!Windy.token) {
    await initWindy();
  }

  const data = await getWindyWeather();
  return windyToCommon(data.data);
}

function proxyFetch(url: string) {
  return fetch(`http://localhost:${import.meta.env.VITE_APP_PROXY_PORT}/proxy`, {
    method: 'POST',
    body: JSON.stringify({
      url,
    }),
  });
}

function matchFirst(str: string, regex: RegExp): string | undefined {
  const possible = str.match(regex);
  if (!possible) return undefined;
  return possible[1];
}

async function initWindy() {
  const baseRes = await proxyFetch(
    `https://www.windy.com/-Rain-thunder-rain?rain,${import.meta.env.VITE_APP_LATITUDE},${
      import.meta.env.VITE_APP_LONGITUDE
    },9=`,
  ).then(x => x.text());
  const token = matchFirst(baseRes, /meta name="token" content="([^"]+)"/);
  const version = matchFirst(baseRes, /window.W={version:"([^"]+)"/);
  if (!token || !version) return;

  const uid = randomUuid();
  const { token: token2 } = await proxyFetch(
    `https://node.windy.com/users/info?token=${token}&token2=pending&uid=${uid}&sc=1&pr=1&v=27.2.0&poc=1`,
  ).then(x => x.json());

  Windy = {
    version,
    token,
    uid,
    token2,
  };
}

export function windyToCommon(data: WindyData): CommonWeatherData {
  return {
    temp: data.temp,
    rain: data.rain,
    timestamp: data.ts,
    icon: data.icon,
  };
}

async function getWindyWeather(): Promise<WindyResponse> {
  return proxyFetch(
    `https://node.windy.com/${btoa(
      `forecast?ecmwf?point/ecmwf/v2.7/${import.meta.env.VITE_APP_LATITUDE}/${
        import.meta.env.VITE_APP_LONGITUDE
      }?source=detail&step=3&token=${Windy.token}&token2=${Windy.token2}&uid=${Windy.uid}&sc=1&pr=0&v=${
        Windy.version
      }&poc=10`,
    )}`,
  )
    .then(x => x.text())
    .then(x => JSON.parse(atob(x)));
}

function randomHex(len: number) {
  return [...Array(len)].map(() => Math.floor(Math.random() * 16).toString(16)).join('');
}

function randomUuid() {
  return `${randomHex(8)}-${randomHex(4)}-${randomHex(4)}-${randomHex(4)}-${randomHex(12)}`;
}
