import { scaleLinear, scaleTime } from '@visx/scale';
import React, { useMemo } from 'react';
import { CommonWeatherObj } from '../types';
import { LinearGradient } from '@visx/gradient';
import { GridRows } from '@visx/grid';
import { AreaClosed } from '@visx/shape';
import { curveBasis } from '@visx/curve';
import { extent, max, min } from 'd3-array';
import { THEME_COLOR } from '../constants';
// accessors
const getDate = (d: CommonWeatherObj) => new Date(d.timestamp);

interface Props {
  width: number;
  height: number;
  padding?: { top: number; bottom: number };

  data: CommonWeatherObj[];
  getMetric: (obj: CommonWeatherObj) => number;
}

const WeatherGraph: React.FC<Props> = ({ data, getMetric, width, padding = { top: 0.05, bottom: 0.05 }, height }) => {
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
        <AreaClosed<CommonWeatherObj>
          data={data}
          x={d => dateScale(getDate(d)) ?? 0}
          y={d => valueScale(getMetric(d)) ?? 0}
          yScale={valueScale}
          strokeWidth={1}
          stroke="url(#area-gradient)"
          fill="url(#area-gradient)"
          curve={curveBasis}
        />
      </svg>
    </div>
  );
};

export default WeatherGraph;
