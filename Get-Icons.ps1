Write-Output "Downloading night icons";
1..27 | % -Parallel { iwr "https://www.windy.com/img/icons6/svg/$($_)_night_2.svg" -OutFile (New-Item -Path "packages/shared/assets/weather/icons/night/$($_).svg" -Force) };
Write-Output "Downloading day icons";
1..27 | % -Parallel { iwr "https://www.windy.com/img/icons6/svg/$($_).svg" -OutFile (New-Item -Path "packages/shared/assets/weather/icons/day/$($_).svg" -Force) };
