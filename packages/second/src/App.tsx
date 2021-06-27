import React, { useEffect, useState } from 'react';
import './App.css';
import WeatherReport from './weather/WeatherReport';
import { ReportDataEl } from './api/dwd-api.types';
import { getReport } from './api/dwd-api';

function App() {
  const [report, setReport] = useState<null | ReportDataEl[]>(null);
  useEffect(() => {
    let timeout = 0;
    const updateReport = async () => {
      try {
        setReport(await getReport());
      } catch (e) {
        console.error(e);
      }
      timeout = setTimeout(() => updateReport(), 5 * 60 * 1000);
    };
    updateReport().catch(console.error);

    return () => clearTimeout(timeout);
  }, [setReport]);
  return <main>{report ? <WeatherReport data={report} /> : null}</main>;
}

export default App;
