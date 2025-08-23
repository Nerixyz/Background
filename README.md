# Background

This is my background 🙂. It really only shows the weather. I didn't want to use some web view, so I used Skia. But now it only works on Windows 11 (unless someone adds support for others).

It uses the [DWD open-data server](https://opendata.dwd.de/) to get weather data (so it only works in Germany right now).

# Images

TBD

# Setup

- To get the icons (I'm using the icons from [msn.com](https://msn.com)),
  run `Get-Icons.ps1` in [PowerShell](https://github.com/PowerShell/powershell/releases) (you might need PowerShell 7).
- Create a `bg.png` which contains your desired background - cropped to the monitor resolution.
- Create a `config.toml`:

  ```toml
  station = 1234

  # Used for rain forecast
  latitude = 52.1234
  longitude = 9.1234

  # During development for faster startup
  cache_file = "cache.bin"

  # Open on the monitor at this global position
  monitor_at_pos = [0, 0]

  # Stations for weather reports
  # Multiple can be specified (stations at the start take priority)
  synop_stations = ["1234"]
  ```

  Stations can be found on [dwd.de](https://www.dwd.de/DE/leistungen/met_verfahren_mosmix/mosmix_stationskatalog.cfg?view=nasPublication&nn=16102) (use the `ID`).
