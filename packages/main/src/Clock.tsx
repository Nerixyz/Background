import React, { FunctionComponent, useEffect, useState } from 'react';

const Clock: FunctionComponent = () => {
  const [time, setTime] = useState('');
  useEffect(() => {
    const updateClock = () => {
      const now = new Date();
      const text = new Intl.DateTimeFormat('de-DE', { hour: '2-digit', minute: '2-digit' }).format(now);
      setTime(text);

      setTimeout(() => updateClock(), 60 * 1000 - (Number(now) % (60 * 1000)));
    };
    updateClock();
  }, []);

  return <h1>{time}</h1>;
};

export default Clock;
