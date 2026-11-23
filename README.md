# Cuaca

![cuaca](assets/cuaca.png)

Cuaca's idea, structure and design is based off [Wttr.in](script/wttr.in) but written in Rust instead of shell script.
The original script had tables that didn't fit on small screens.

## Installation

### Prerequisites

- Rust 1.85.0 or newer (required for Rust 2024 edition)

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

The application uses a configuration file located in your system's configuration directory:

- **Linux:** `~/.config/cuaca/cuaca.toml`
- **macOS:** `~/Library/Application Support/cuaca/cuaca.toml`
- **Windows:** `%APPDATA%\cuaca\cuaca.toml`

If the configuration file does not exist (or is missing fields), default values will be created and saved automatically.

### Configuration Options

- `api_key`: Your OpenWeatherMap API key.
- `city`: The city for which to fetch weather data (e.g., `"London, GB"` or `"New York"`).
- `units`: Units for temperature and wind speed (`metric` or `imperial`).
- `timeplus`: Hours to add to the UTC timezone for date, sunrise, and sunset times. You can find your timezone at https://www.timeanddate.com/.
- `timeminus`: Hours to subtract from the UTC timezone for date, sunrise, and sunset times. You can find your timezone at https://www.timeanddate.com/.
- `showcityname`: Whether to display the city name (`true` or `false`).
- `showdate`: Whether to display the current date (`true` or `false`).
- `timeformat`: Time format for sunrise and sunset times (`24` or `12`).
- `use_colors`: Enables or disables text colors (`true` or `false`).

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
