import React, { useMemo } from 'react';
import { extent, max, min } from 'd3-array';
import { ReportDataEl } from '../api/dwd-api.types';
import { scaleLinear } from '@visx/scale';
import { LinearGradient } from '@visx/gradient';
import { GridColumns, GridRows } from '@visx/grid';
import { LinePath } from '@visx/shape';
import { curveBasis } from '@visx/curve';
import { THEME_COLOR } from '../constants';
import { everySecond, map } from 'shared/itertools';

const getDate = (d: ReportDataEl) => new Date(d.timestamp);

export interface DataView {
  dataMin?: number;
  dataMax?: number;
}

interface Props {
  width: number;
  height: number;
  padding?: { top: number; bottom: number };
  view?: DataView;

  data: ReportDataEl[];
  getMetric: (obj: ReportDataEl) => number;
}

const WeatherReportGraph: React.FC<Props> = ({
  data,
  getMetric,
  width,
  padding = { top: 0.05, bottom: 0.05 },
  view = {},
  height,
}) => {
  const dateScale = useMemo(
    () =>
      scaleLinear({
        range: [width * 0.01, width * 0.98],
        domain: extent(data, getDate) as [Date, Date],
      }),
    [width, data],
  );
  const valueScale = useMemo(
    () =>
      scaleLinear({
        range: [height, 0],
        domain: [
          (view.dataMin ?? (min(data, getMetric) || 0)) - height * padding.bottom,
          (view.dataMax ?? (max(data, getMetric) || 0)) + height * padding.top,
        ],
        nice: true,
      }),
    [height, view.dataMin, view.dataMax, data, getMetric, padding.bottom, padding.top],
  );

  return (
    <div>
      <svg width={width} height={height}>
        <LinearGradient id="area-gradient" from={THEME_COLOR} to={THEME_COLOR} toOpacity={0.8} />
        <GridRows
          scale={valueScale}
          width={width}
          stroke={THEME_COLOR}
          strokeDasharray="1,5"
          strokeOpacity={0.75}
          pointerEvents="none"
          numTicks={4}
        />
        <GridColumns
          scale={dateScale}
          height={height}
          stroke={THEME_COLOR}
          strokeOpacity={0.2}
          pointerEvents="none"
          tickValues={[...everySecond(map(data, getDate))]}
        />
        <LinePath<ReportDataEl>
          data={data}
          x={d => dateScale(getDate(d)) ?? 0}
          y={d => valueScale(getMetric(d)) ?? 0}
          shapeRendering="geometricPrecision"
          strokeWidth={3}
          stroke="url(#area-gradient)"
          curve={curveBasis}
        />
      </svg>
    </div>
  );
};

export default WeatherReportGraph;
