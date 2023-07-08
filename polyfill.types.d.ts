declare namespace process {
  interface EnvironmentVariables {
    STATION_ID: string;
    ICON_SET: 'msn' | 'windy' | string;
  }

  const env: EnvironmentVariables;
}
