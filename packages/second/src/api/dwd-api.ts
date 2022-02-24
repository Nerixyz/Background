import { ReportDataEl, StationReport } from './dwd-api.types';

export async function getReport(): Promise<ReportDataEl[]> {
  console.log(process.env.STATION_ID);
  const res: StationReport = await fetch(
    `https://weatherapi.nerixyz.de/report/${ process.env.STATION_ID }`,
  ).then(x => x.json());

  return res.data.sort((a, b) => a.timestamp - b.timestamp);
}

export function getIcon(el: ReportDataEl) {
  return DWD_TO_ICON_MAP[(el.present_weather ?? 1) as keyof typeof DWD_TO_ICON_MAP] ?? 1;
}

// https://www.dwd.de/DE/leistungen/opendata/help/schluessel_datenformate/csv/poi_present_weather_zuordnung_pdf.pdf?__blob=publicationFile&v=2
const DWD_TO_ICON_MAP = {
  1: 1,
  2: 1,
  3: 3,
  4: 4,
  5: 17,
  6: 17,
  7: 20,
  8: 7,
  9: 7,
  10: 13,
  11: 13,
  12: 13,
  13: 12,
  14: 10,
  15: 10,
  16: 10,
  18: 20,
  19: 20,
  20: 13,
  21: 13,
  22: 13,
  23: 13,
  24: 13,
  25: 13,
  26: 14,
  27: 14,
  28: 14,
  29: 14,
  30: 14,
  31: 4,
};
