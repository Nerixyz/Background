import csv
import sys
import json

if len(sys.argv) < 2:
    print("Usage: stations-to-geojson.py <path-to-stationlist-synoptic-germany.csv>")
    exit(1)

features = []
with open(sys.argv[1], encoding="utf8") as csvfile:
    reader = csv.DictReader(csvfile, delimiter=";")
    for line in reader:
        features.append(
            {
                "type": "Feature",
                "geometry": {
                    "type": "Point",
                    "coordinates": [
                        float(line["Geog_Laenge"]),
                        float(line["Geog_Breite"]),
                    ],
                },
                "properties": line,
                "id": len(features),
            }
        )
print(json.dumps({"type": "FeatureCollection", "features": features}))
