Write-Output "Downloading night icons";
1..27 | % -Parallel { iwr "https://www.windy.com/img/icons6/svg/$($_)_night_2.svg" -OutFile (New-Item -Path "icons/windy/night/$($_).svg" -Force) };
Write-Output "Downloading day icons";
1..27 | % -Parallel { iwr "https://www.windy.com/img/icons6/svg/$($_).svg" -OutFile (New-Item -Path "icons/windy/day/$($_).svg" -Force) };

Write-Output "Downloading MSN icons";

(
    "BlowingHailV2",
    "ClearNightV3",
    "CloudyV3",
    "D200PartlySunnyV2",
    "D210LightRainShowersV2",
    "D211LightRainSowShowersV2",
    "D212LightSnowShowersV2",
    "D221RainSnowShowersV2",
    "D240TstormsV2",
    "D310LightRainShowersV2",
    "D311LightRainSnowShowersV2",
    "D321RainSnowShowersV2",
    "D340TstormsV2",
    "FogV2",
    "FreezingRainV2",
    "Haze",
    "HeavyDrizzle",
    "HeavySnowV2",
    "IcePelletsV2",
    "LightRainShowerDay",
    "LightRainShowerNight",
    "LightRainV3",
    "LightSnowShowersDay",
    "LightSnowShowersNight",
    "LightSnowV2",
    "ModerateRainV2",
    "MostlyClearNight",
    "MostlyCloudyDayV2",
    "MostlyCloudyNightV2",
    "MostlySunnyDay",
    "N210LightRainShowersV2",
    "N211LightRainSnowShowersV2",
    "N212LightSnowShowersV2",
    "N221RainSnowShowersV2",
    "N222SnowShowersV2",
    "N240TstormsV2",
    "N310LightRainShowersV2",
    "N311LightRainSnowShowersV2",
    "N321RainSnowShowersV2",
    "N322SnowShowersV2",
    "N340TstormsV2",
    "N422SnowV2",
    "PartlyCloudyNightV2",
    "RainShowersDayV2",
    "RainShowersNightV2",
    "RainSnowShowersNightV2",
    "RainSnowV2",
    "Snow",
    "SnowShowersDayV2",
    "SunnyDayV3",
    "ThunderstormsV2",
    "WindyV2"
) | % -Parallel { iwr "https://assets.msn.com/weathermapdata/1/static/weather/Icons/taskbar_v10/Condition_Card/$($_).svg" -OutFile (New-Item -Path "icons/msn/$($_).svg" -Force) };

Copy-Item -Recurse -Force .\extra-icons\* icons
