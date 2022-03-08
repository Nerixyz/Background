import React, { FunctionComponent } from 'react';
import { ReportDataEl } from '../api/dwd-api.types';
import './WeatherReport.css';
import WeatherMetric from './WeatherMetric';
import { getIcon } from '../api/dwd-api';
// @ts-ignore
import * as images from '../../../shared/assets/weather/icons/**/*.svg';

interface Props {
  data: ReportDataEl[];
  updatedAt: Date;
}

const WeatherReport: FunctionComponent<Props> = props => {
  return (
    <div className="report">
      <div className="icon-wrapper">
        <img alt="icon" className="report-icon" src={pathFromData(images,props.data) ?? ''} />
      </div>
      <div>
        <WeatherMetric
          title={'Temperature'}
          data={props.data}
          getMetric={d => d.dry_bulb_temperature_at_2_meter_above_ground ?? 0}
          unit="°C"
        />
        <WeatherMetric
          title={'Precipitation'}
          data={props.data}
          getMetric={d => d.precipitation_amount_last_hour ?? 0}
          unit="mm"
          view={{dataMin: 0}}
        />
        <WeatherMetric
          title={'Wind Speed'}
          data={props.data}
          getMetric={d => d['mean_wind_speed_during last_10_min_at_10_meters_above_ground'] ?? 0}
          unit="km/h"
          view={{dataMin: 0}}
        />
        <WeatherMetric title={'Cloud Cover'} data={props.data} getMetric={d => d.cloud_cover_total ?? 0} unit="%" view={{dataMin: 0, dataMax: 100}} />
        <WeatherMetric title={'Humidity'} data={props.data} getMetric={d => d.relative_humidity ?? 0} unit="%" view={{dataMin: 0, dataMax: 100}}/>
      </div>
      <div className="report-updated-at">
        Last Updated: {new Intl.DateTimeFormat(undefined, { timeStyle: 'short', hour12: false }).format(props.updatedAt)}
      </div>
    </div>
  );
};

export default WeatherReport;

function pathFromData(images: Record<'day' | 'night', Record<number, string>>, data: ReportDataEl[]): string | undefined {
  const el = data[data.length - 1];
  const hour = new Date(el.timestamp).getHours();
  return images[hour < 7 || hour > 20 ? 'night' : 'day']?.[getIcon(el)];
}
