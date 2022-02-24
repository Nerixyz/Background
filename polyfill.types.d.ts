declare namespace process {
  interface EnvironmentVariables {
    STATION_ID: string,
  }

  const env: EnvironmentVariables;
}
