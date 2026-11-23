# Cuaca

![cuaca](assets/cuaca.png)

Cuaca's idea, structure and design is based off [Wttr.in](script/wttr.in) but written in Rust instead of shell script.
The original script had tables that didn't fit on a small screen.

## Installation

### Prerequisites

- **Rust 1.85.0 or newer** (required for Rust 2024 edition)

### From source:

1. Clone the repository:

   ```sh
   git clone https://github.com/ddCeka/cuaca.git
   cd cuaca
   ```

2. Build the project using Cargo:
   ```sh
   cargo build --release
   ```

## Configuration

The application stores its configuration file in your system's standard configuration directory:

- **Linux:** `~/.config/cuaca/cuaca.toml`
- **macOS:** `~/Library/Application Support/cuaca/cuaca.toml`
- **Windows:** `%APPDATA%\cuaca\cuaca.toml`

If the configuration file does not exist, a default one will be created automatically. If new options are added in updates, missing fields are automatically merged with default values and saved back to your `cuaca.toml`.

### Configuration Options

- `api_key`: Your OpenWeatherMap API key.
- `city`: The city for which to fetch weather data (spaces and country codes like `"London, GB"` or `"New York"` are supported).
- `units`: Units for temperature and wind speed (`"metric"` or `"imperial"`).
- `timeplus`: Hours to add to the UTC timezone (find your timezone at https://www.timeanddate.com/).
- `timeminus`: Hours to subtract from the UTC timezone (find your timezone at https://www.timeanddate.com/).
- `showcityname`: Whether to display the city name (`true` or `false`).
- `showdate`: Whether to display the current date adjusted by your timezone offset (`true` or `false`).
- `timeformat`: Time format for sunrise and sunset times (`"24"` or `"12"`).
- `use_colors`: Enables or disables custom ANSI terminal colors (`true` or `false`).

### Color Customization (`[color]`)

When `use_colors = true`, every output field can be customized using `#RRGGBB` hex color codes under the `[color]` section (defaulting to an eye-comfort adapted Gotham dark terminal palette). Invalid hex values fall back to uncolored terminal text.

- `city`: City line color (default: `"#34BFA0"`).
- `temp`: Temperature line color (default: `"#E08256"`).
- `wind`: Wind speed line color (default: `"#34BFA0"`).
- `sunrise`: Sunrise line color (default: `"#E08256"`).
- `sunset`: Sunset line color (default: `"#599CAB"`).
- `date_label`: `"Date: "` prefix color (default: `"#99D1CE"`).
- `date_value`: Formatted date string color (default: `"#99D1CE"`).
- `clear`: Weather label color for clear skies (default: `"#D3EBE9"`).
- `clouds`: Weather label color for cloudy skies (default: `"#A0B9D6"`).
- `rain`: Weather label color for rain (default: `"#599CAB"`).
- `snow`: Weather label color for snow (default: `"#B8CCE4"`).
- `thunderstorm`: Weather label color for thunderstorms (default: `"#D9544D"`).
- `other`: Weather label color for other conditions (default: `"#9C9FB8"`).

Example configuration:

```toml
api_key = "your_api_key"
city = "London, GB"
units = "metric"
timeplus = 0
timeminus = 0
showcityname = true
showdate = true
timeformat = "24"
use_colors = true

[color]
city = "#34BFA0"
temp = "#E08256"
wind = "#34BFA0"
sunrise = "#E08256"
sunset = "#599CAB"
date_label = "#99D1CE"
date_value = "#99D1CE"
clear = "#D3EBE9"
clouds = "#A0B9D6"
rain = "#599CAB"
snow = "#B8CCE4"
thunderstorm = "#D9544D"
other = "#9C9FB8"
```
