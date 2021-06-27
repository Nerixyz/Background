export type StationUnit =
  | 'past_weather_1'
  | 'global_radiation_last_hour'
  | 'dry_bulb_temperature_at_2_meter_above_ground'
  | 'depth_of_new_snow'
  | 'maximum_wind_speed_last_hour'
  | 'precipitation_last_12_hours'
  | 'global_radiation_past_24_hours'
  | 'minimum_temperature_last_12_hours_5_cm_above_ground'
  | 'mean_wind_direction_during_last_10'
  | 'minimum_of_temperature_at_5_cm_above_ground_for_previous_day'
  | 'past_weather_2'
  | 'precipitation_amount_last_3_hours'
  | 'minimum_temperature_last_12_hours_2_meters_above_ground'
  | 'daily_mean_of_temperature_previous_day'
  | 'maximum_wind_speed_as_10_minutes_mean_during_last_hour'
  | 'maximum_temperature_last_12_hours_2_meters_above_ground'
  | 'maximum_wind_speed_for_previous_day'
  | 'horizontal_visibility'
  | 'minimum_of_temperature_for_previous_day'
  | 'precipitation_amount_last_24_hours'
  | 'diffuse_solar_radiation_last_hour'
  | 'direct_solar_radiation_last_hour'
  | 'present_weather'
  | 'total_time_of_sunshine_past_day'
  | 'maximum_of_temperature_for_previous_day'
  | 'total_time_of_sunshine_during_last_hour'
  | 'maximum_of_10_minutes_mean_of_wind_speed_for_previous_day'
  | 'direct_solar_radiation_last_24_hours'
  | 'precipitation_amount_last_6_hours'
  | 'total_snow_depth'
  | 'mean_wind_direction_during_last_10 min_at_10_meters_above_ground'
  | 'mean_wind_speed_during last_10_min_at_10_meters_above_ground'
  | 'cloud_cover_total'
  | 'precipitation_amount_last_hour'
  | 'evaporation'
  | 'height_of_base_of_lowest_cloud_above_station'
  | 'dew_point_temperature_at_2_meter_above_ground'
  | 'maximum_wind_speed_during_last_6_hours'
  | 'pressure_reduced_to_mean_sea_level'
  | 'temperature_at_5_cm_above_ground'
  | 'sea'
  | 'relative_humidity';

type Units = { [P in StationUnit]: string }; // see table below

export interface StationReport {
  units: Units;
  data: ReportDataEl[];
}

export type ReportDataEl = {
  [P in StationUnit]?: number;
} & { timestamp: number };
