import React, { FunctionComponent, useMemo } from 'react';
import WeatherGraph from './WeatherGraph';
import { CommonWeatherObj } from '../types';
import { ParentSize } from '@visx/responsive';
import './Weather.css';
import { stringifyNum, stringifyTemp } from '../utilities';
import { everySecond, map } from '../itertools';
const images = import.meta.globEager('/src/assets/weather/**/*.svg');

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
            const path = `/src/assets/weather/icons/${hour < 7 || hour > 20 ? 'night' : 'day'}/${d.icon}.svg`;
            return (
              <div key={d.timestamp} className="info-card">
                <div className="info-hour">{hour}</div>
                <img alt="icon" className="info-icon" src={images[path]?.default ?? ''} />
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
        Last Updated: {new Intl.DateTimeFormat(undefined, {timeStyle: 'short'}).format(props.updatedAt)}
      </div>
    </div>
  );
};

export default Weather;
