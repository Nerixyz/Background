import { ParentSize } from '@visx/responsive';
import React, { FunctionComponent } from 'react';
import { ReportDataEl } from '../api/dwd-api.types';
import WeatherReportGraph, { DataView } from './WeatherReportGraph';
import { lastNotUndefined, toPrettyString } from '../utilities';

interface Props {
  title: string;
  unit?: string;

  data: ReportDataEl[];
  getMetric: (obj: ReportDataEl) => number;
  getMetricMaybe?: (obj: ReportDataEl) => number | null;
  view?: DataView;
}

const WeatherMetric: FunctionComponent<Props> = props => {
  return (
    <div className="report-metric">
      <div className="metric-now">
        <div className="metric-title">{props.title}</div>
        <div className="metric-value">
          {toPrettyString(lastNotUndefined(props.data, props.getMetricMaybe ?? props.getMetric))}
          {props.unit ?? ''}
        </div>
      </div>
      <div className="metric-graph">
        <ParentSize>
          {({ width, height }) => (
            <WeatherReportGraph
              width={width}
              height={height}
              data={props.data}
              getMetric={props.getMetric}
              padding={{ top: 0.01, bottom: 0.01 }}
              view={props.view}
            />
          )}
        </ParentSize>
      </div>
    </div>
  );
};

export default WeatherMetric;
