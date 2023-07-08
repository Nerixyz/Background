import React, { useEffect, useState } from 'react';
import './App.css';
import WeatherReport from './weather/WeatherReport';
import { ReportDataEl } from './api/dwd-api.types';
import { getReport } from './api/dwd-api';

function App() {
  const [report, setReport] = useState<null | ReportDataEl[]>(null);
  const [updatedAt, setUpdatedAt] = useState(new Date());
  useEffect(() => {
    let timeout = 0;
    const updateReport = async () => {
      try {
        const data = await getReport();
        setUpdatedAt(new Date(data[data.length - 1].timestamp));
        setReport(data);
      } catch (e) {
        console.error(e);
      }
      timeout = setTimeout(() => updateReport(), 5 * 60 * 1000);
    };
    updateReport().catch(console.error);

    return () => clearTimeout(timeout);
  }, [setReport]);
  return <main>{report ? <WeatherReport data={report} updatedAt={updatedAt} /> : null}</main>;
}

export default App;
