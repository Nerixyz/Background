import React, { FunctionComponent, useMemo } from 'react';
import WeatherGraph from './WeatherGraph';
import { CommonIcon, CommonWeatherObj } from '../types';
import { ParentSize } from '@visx/responsive';
import './Weather.css';
import { stringifyNum, stringifyTemp } from '../utilities';
import { everySecond, map } from 'shared/itertools';
// @ts-ignore
import * as windyImages from '../../../shared/assets/weather/icons/windy/**/*.svg';
// @ts-ignore
import * as msnImages from '../../../shared/assets/weather/icons/msn/*.svg';

interface Props {
  data: CommonWeatherObj[];
  updatedAt: Date;
}

const Weather: FunctionComponent<Props> = props => {
  const anyRain = useMemo(() => !!props.data.find(d => d.rain !== 0), [props.data]);
  return (
    <div id="weather">
      <div className="weather-graph">
        <ParentSize>
          {({ width, height }) => (
            <WeatherGraph getMetric={d => d.temp - 237.15} data={props.data} height={height} width={width} />
          )}
        </ParentSize>
      </div>
      <div className="info-cards">
        {[
          ...map(everySecond(props.data), d => {
            const hour = new Date(d.timestamp).getHours();
            return (
              <div key={d.timestamp} className="info-card">
                <div className="info-hour">{hour}</div>
                <img alt="icon" className="info-icon" src={iconAtHour(d.icon, hour) ?? ''} />
                <div className="info-temp">{stringifyTemp(d.temp)}°C</div>
                {anyRain ? <div className="info-rain">{stringifyNum(d.rain)}mm</div> : null}
              </div>
            );
          }),
        ]}
      </div>
      {anyRain ? (
        <div className="weather-graph">
          <ParentSize>
            {({ width, height }) => (
              <WeatherGraph
                getMetric={d => d.rain}
                data={props.data}
                padding={{ top: 0.05, bottom: 0 }}
                height={height}
                width={width}
              />
            )}
          </ParentSize>
        </div>
      ) : null}
      <div className="weather-updated-at">
        Last Updated:{' '}
        {new Intl.DateTimeFormat(undefined, { timeStyle: 'short', hour12: false }).format(props.updatedAt)}
      </div>
    </div>
  );
};

export default Weather;

function iconAtHour(ico: CommonIcon, hour: number) {
  const isNight = hour < 7 || hour > 20;
  if (process.env.ICON_SET === 'msn') {
    return msnImages[isNight ? ico.msn[1] : ico.msn[0]];
  } else {
    return windyImages[isNight ? 'night' : 'day']?.[ico.windy];
  }
}
