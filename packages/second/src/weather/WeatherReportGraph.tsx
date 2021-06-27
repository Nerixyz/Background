import React, { useMemo } from 'react';
import { extent, max, min } from 'd3-array';
import { ReportDataEl } from '../api/dwd-api.types';
import { scaleLinear, scaleTime } from '@visx/scale';
import { LinearGradient } from '@visx/gradient';
import { GridRows } from '@visx/grid';
import { LinePath } from '@visx/shape';
import { curveMonotoneX } from '@visx/curve';
import { THEME_COLOR } from '../constants';

const getDate = (d: ReportDataEl) => new Date(d.timestamp);

interface Props {
  width: number;
  height: number;
  padding?: { top: number; bottom: number };

  data: ReportDataEl[];
  getMetric: (obj: ReportDataEl) => number;
}

const WeatherReportGraph: React.FC<Props> = ({
  data,
  getMetric,
  width,
  padding = { top: 0.05, bottom: 0.05 },
  height,
}) => {
  const dateScale = useMemo(
    () =>
      scaleTime({
        range: [0, width],
        domain: extent(data, getDate) as [Date, Date],
      }),
    [width, data],
  );
  const valueScale = useMemo(
    () =>
      scaleLinear({
        range: [height, 0],
        domain: [
          (min(data, getMetric) || 0) - height * padding.bottom,
          (max(data, getMetric) || 0) + height * padding.top,
        ],
        nice: true,
      }),
    [height, data, getMetric, padding.bottom, padding.top],
  );

  return (
    <div>
      <svg width={width} height={height}>
        <LinearGradient id="area-gradient" from={THEME_COLOR} to={THEME_COLOR} toOpacity={0.8} />
        <GridRows
          scale={valueScale}
          width={width}
          strokeDasharray="1,3"
          stroke={THEME_COLOR}
          strokeOpacity={0}
          pointerEvents="none"
        />
        <LinePath<ReportDataEl>
          data={data}
          x={d => dateScale(getDate(d)) ?? 0}
          y={d => valueScale(getMetric(d)) ?? 0}
          shapeRendering="geometricPrecision"
          strokeWidth={3}
          stroke="url(#area-gradient)"
          curve={curveMonotoneX}
        />
      </svg>
    </div>
  );
};

export default WeatherReportGraph;
