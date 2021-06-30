import React, { FunctionComponent } from 'react';
import { ReportDataEl } from '../api/dwd-api.types';
import './WeatherReport.css';
import WeatherMetric from './WeatherMetric';
import { getIcon } from '../api/dwd-api';
const images = import.meta.globEager('/src/assets/weather/**/*.svg');

interface Props {
  data: ReportDataEl[];
  updatedAt: Date;
}

const WeatherReport: FunctionComponent<Props> = props => {
  return (
    <div className="report">
      <div className="icon-wrapper">
        <img alt="icon" className="report-icon" src={images[pathFromData(props.data)]?.default ?? ''} />
      </div>
      <div>
        <WeatherMetric
          title={'Temperature'}
          data={props.data}
          getMetric={d => d.temperature_at_5_cm_above_ground ?? 0}
          unit="°C"
        />
        <WeatherMetric
          title={'Precipitation'}
          data={props.data}
          getMetric={d => d.precipitation_amount_last_hour ?? 0}
          unit="mm"
        />
        <WeatherMetric
          title={'Wind Speed'}
          data={props.data}
          getMetric={d => d['mean_wind_speed_during last_10_min_at_10_meters_above_ground'] ?? 0}
          unit="km/h"
        />
        <WeatherMetric title={'Cloud Cover'} data={props.data} getMetric={d => d.cloud_cover_total ?? 0} unit="%" />
        <WeatherMetric title={'Humidity'} data={props.data} getMetric={d => d.relative_humidity ?? 0} unit="%" />
      </div>
      <div className="report-updated-at">
        Last Updated: {new Intl.DateTimeFormat(undefined, { timeStyle: 'short' }).format(props.updatedAt)}
      </div>
    </div>
  );
};

export default WeatherReport;

function pathFromData(data: ReportDataEl[]): string {
  const el = data[data.length - 1];
  const hour = new Date(el.timestamp).getHours();
  return `/src/assets/weather/icons/${hour < 7 || hour > 20 ? 'night' : 'day'}/${getIcon(el)}.svg`;
}
