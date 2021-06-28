# Background

This is my background 🙂. It's mainly for Windows with [Wallpaper Engine](https://store.steampowered.com/app/431960/Wallpaper_Engine/).

It uses the [DWD open-data server](https://opendata.dwd.de/) to get weather data.

# Images

### MainMonitor

![Main Monitor](https://i.imgur.com/HdG5gf1.png)

### Second Monitor (yeah I'm still using a 4:3 one)

![Second Monitor](https://i.imgur.com/lWTAfjY.png)

# Setup

- To get the icons (I'm using the icons from [Windy.com](https://windy.com)),
  run `Get-Icons.ps1` in [PowerShell](https://github.com/PowerShell/powershell/releases) (you might need PowerShell 7).
- Rename `.env.example` to `.env` and edit the station-id.

  Stations can be found on [`weatherapi.nerixyz.de/stations`](https://weatherapi.nerixyz.de/stations) or [dwd.de](https://www.dwd.de/DE/leistungen/met_verfahren_mosmix/mosmix_stationskatalog.cfg?view=nasPublication&nn=16102).

- Run `npm i` to install the dependencies (you need npm v7).
- Run `npm run build-unsafe` to build both backgrounds (unsafe because it's not type-checked).

### Setting up a local webserver

I'm using [nginx](https://nginx.org) to host the SPA. This tutorial is for Windows :)

- Download [nginx](https://nginx.org/en/download.html).
- Unpack it _somewhere_.
- Download [nssm](https://nssm.cc/download).
- Unpack the .exe for your architecture (you only need the exe).
- Open a shell and run `nssm install nginx`.
- In the GUI, set the `path` to the `nginx.exe` (the directory should be automatically set)
- Navigate to the `IO` tab and set `Input (stdin)` to `start nginx`
- Click `Install Service`

#### Configuring Nginx

- In the nginx folder, open `conf/nginx.conf`
- Delete the default server block and create two new ones, each pointing to the dist folder.
- An example:

`MAIN_PORT` and `SECOND_PORT` should be different and have a high number (e.g. 54879 and 54880).
`MAIN_DIST_PATH` and `SECOND_DIST_PATH` should be with _forward slashes_ (e.g. `C:/something/dist`) and contain the path to the respective dist folder.

```conf
error_log logs/error.log warn;
http {
    include       mime.types;
    default_type  application/octet-stream;

    sendfile        on;
    keepalive_timeout  65;
    server {
        listen       MAIN_PORT;
        server_name  localhost;

        root MAIN_DIST_PATH;
    }
    server {
        listen       SECOND_PORT;
        server_name  localhost;

        root SECOND_DIST_PATH;
    }
}
```

- Open the TaskManager
- Go to the `Services` tab
- Start the `nginx` service (right click)

### Setting up the Wallpaper

Wallpaper Engine supports websites as wallpapers through CEF, so we'll use that.

- Open the Wallpaper Engine GUI.
- For each monitor:
  - At the bottom click `Open Wallpaper`
  - Select `Open from URL`
  - Enter the URL `http://localhost:MAIN_OR_SECOND_PORT` (use the port you specified in the `nginx.conf`)
  - Select the Wallpaper
