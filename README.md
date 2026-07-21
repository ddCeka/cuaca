# Cuaca

Cuaca is a command-line application that fetches and displays weather information from the OpenWeatherMap API.
Cuaca's idea, structure and design is based off [Wttr.in](script/wttr.in) but writen in Rust instead of shell script.
The original script have tables that didn't fit on small screen

![cuaca](assets/cuaca.png)

## Installation

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

The application uses a configuration file located at `~/.config/cuaca/cuaca.toml`. If the configuration file does not exist, a default one will be created.

### Configuration Options

- `api_key`: Your OpenWeatherMap API key.
- `city`: The city for which to fetch weather data.
- `units`: Units for temperature and wind speed (`metric` or `imperial`).
- `timeplus`: Hours to add to the UTC timezone, you can find your timezone here https://www.timeanddate.com/.
- `timeminus`: Hours to subtract from the UTC timezone, you can find your timezone here https://www.timeanddate.com/.
- `showcityname`: Whether to display the city name (`true` or `false`).
- `showdate`: Whether to display the current date (`true` or `false`).
- `timeformat`: Time format for sunrise and sunset times (`24` or `12`).
- `use_colors`: Enables and disabled text colors (`true` or `false`).

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
use_colors = false
```
