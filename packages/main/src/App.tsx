import React, { useEffect, useState } from 'react';
import './App.css';
import Weather from './weather/Weather';
import Clock from './Clock';
import { getDwdCommonWeather } from './api/dwd-api';
import { dataArraysToObjectsUnsafe, findFirstValidTs } from './api/common';
import { CommonWeatherData, CommonWeatherObj } from './types';

function App() {
  const [weatherData, setWeatherData] = useState<CommonWeatherObj[]>([]);
  useEffect(() => {
    const updateData = async () => {
      try {
        const data = await getDwdCommonWeather();
        const firstTs = findFirstValidTs(data);
        setWeatherData(dataArraysToObjectsUnsafe<CommonWeatherData>(data).slice(firstTs, firstTs + 24));
      } catch (e) {
        console.error(e);
      }
    };
    updateData().catch(console.error);
    const interval = setInterval(updateData, 10 * 60 * 1000);

    return () => clearInterval(interval);
  }, [setWeatherData]);
  return (
    <main>
      <Clock />
      <div className={[weatherData ? '' : 'hidden', 'fade'].join(' ')}>
        {weatherData ? <Weather data={weatherData} /> : undefined}
      </div>
    </main>
  );
}

export default App;
