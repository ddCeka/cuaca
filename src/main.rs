use chrono::{DateTime, Duration, TimeZone, Utc};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Serialize, Clone, PartialEq, Debug)]
#[serde(default)]
struct ColorConfig {
    city: String,
    temp: String,
    wind: String,
    sunrise: String,
    sunset: String,
    date_label: String,
    date_value: String,
    clear: String,
    clouds: String,
    rain: String,
    snow: String,
    thunderstorm: String,
    other: String,
}

impl Default for ColorConfig {
    fn default() -> Self {
        ColorConfig {
            city: String::from("#34BFA0"),
            temp: String::from("#E08256"),
            wind: String::from("#34BFA0"),
            sunrise: String::from("#E08256"),
            sunset: String::from("#599CAB"),
            date_label: String::from("#99D1CE"),
            date_value: String::from("#99D1CE"),
            clear: String::from("#D3EBE9"),
            clouds: String::from("#A0B9D6"),
            rain: String::from("#599CAB"),
            snow: String::from("#B8CCE4"),
            thunderstorm: String::from("#D9544D"),
            other: String::from("#9C9FB8"),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Debug)]
#[serde(default)]
struct Config {
    api_key: String,
    city: String,
    units: String,
    timeplus: i64,
    timeminus: i64,
    showcityname: bool,
    showdate: bool,
    timeformat: String,
    use_colors: bool,
    color: ColorConfig,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            api_key: String::new(),
            city: String::new(),
            units: String::from("metric"),
            timeplus: 0,
            timeminus: 0,
            showcityname: false,
            showdate: false,
            timeformat: String::from("24"),
            use_colors: false,
            color: ColorConfig::default(),
        }
    }
}

fn default_config_path() -> PathBuf {
    let mut path = dirs::config_dir().expect("Failed to get config directory");
    path.push("cuaca");
    path.push("cuaca.toml");
    path
}

fn load_or_init_config(config_path: &Path) -> Config {
    if !config_path.exists() {
        let default_config = Config::default();
        let toml_string =
            toml::to_string(&default_config).expect("Failed to serialize default config");
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create config directory");
        }
        let mut file = File::create(config_path).expect("Failed to create config file");
        file.write_all(toml_string.as_bytes())
            .expect("Failed to write default config to file");

        println!(
            "No config detected, config made at {}.",
            config_path.display()
        );
        return default_config;
    }

    let config_content = fs::read_to_string(config_path).expect("Failed to read config file");
    let config: Config = toml::from_str(&config_content).unwrap_or_default();

    let toml_string = toml::to_string(&config).expect("Failed to serialize merged config");
    if toml_string != config_content {
        let mut file = File::create(config_path).expect("Failed to create config file");
        file.write_all(toml_string.as_bytes())
            .expect("Failed to write merged config to file");
    }

    config
}

fn read_config() -> Config {
    load_or_init_config(&default_config_path())
}

fn parse_hex_color(hex: &str) -> Option<(u8, u8, u8)> {
    let hex = hex.trim().strip_prefix('#').unwrap_or(hex.trim());
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some((r, g, b))
}

fn apply_hex_color(text: &str, hex: &str, use_colors: bool, bold: bool) -> String {
    if !use_colors {
        return text.to_string();
    }
    match parse_hex_color(hex) {
        Some((r, g, b)) if bold => format!("\x1b[1;38;2;{r};{g};{b}m{text}\x1b[0m"),
        Some((r, g, b)) => format!("\x1b[38;2;{r};{g};{b}m{text}\x1b[0m"),
        None if bold => format!("\x1b[1m{text}\x1b[0m"),
        None => text.to_string(),
    }
}

fn format_weather(weather_data: &Value, config: &Config, now_utc: DateTime<Utc>) -> String {
    let weather = weather_data["weather"][0]["main"]
        .as_str()
        .unwrap_or("Unknown");
    let temp = weather_data["main"]["temp"].as_f64().unwrap_or(0.0);
    let wind_speed = weather_data["wind"]["speed"].as_f64().unwrap_or(0.0);
    let sunrise = weather_data["sys"]["sunrise"].as_i64().unwrap_or(0);
    let sunset = weather_data["sys"]["sunset"].as_i64().unwrap_or(0);

    let windspeedunits = match config.units.as_str() {
        "metric" => "m/s",
        "imperial" => "mph",
        _ => "m/s",
    };

    let temp_unit = match config.units.as_str() {
        "metric" => "°C",
        "imperial" => "°F",
        _ => "K",
    };

    let temp_raw = format!("Temperature: {:.1}{}", temp, temp_unit);
    let temp_str = apply_hex_color(&temp_raw, &config.color.temp, config.use_colors, false);

    let wind_raw = format!("Wind speed: {:.1} {}", wind_speed, windspeedunits);
    let wind_speed_str = apply_hex_color(&wind_raw, &config.color.wind, config.use_colors, false);

    let offset = Duration::hours(config.timeplus) - Duration::hours(config.timeminus);
    let sunrise_datetime: DateTime<Utc> = Utc.timestamp_opt(sunrise, 0).unwrap();
    let sunset_datetime: DateTime<Utc> = Utc.timestamp_opt(sunset, 0).unwrap();

    let adjusted_sunrise = sunrise_datetime + offset;
    let adjusted_sunset = sunset_datetime + offset;

    let time_format = match config.timeformat.as_str() {
        "12" => "%I:%M %p",
        _ => "%H:%M",
    };

    let sunrisestring = adjusted_sunrise.format(time_format).to_string();
    let sunsetstring = adjusted_sunset.format(time_format).to_string();

    let city_str = if config.showcityname {
        let city_raw = format!("City: {}", config.city);
        apply_hex_color(&city_raw, &config.color.city, config.use_colors, true)
    } else {
        String::new()
    };

    let sunrise_raw = format!("Sunrise: {sunrisestring}");
    let sunrise_str = apply_hex_color(
        &sunrise_raw,
        &config.color.sunrise,
        config.use_colors,
        false,
    );

    let sunset_raw = format!("Sunset: {sunsetstring}");
    let sunset_str = apply_hex_color(&sunset_raw, &config.color.sunset, config.use_colors, false);

    let (art, weather_str) = match weather {
        "Clear" => (
            [
                "             ",
                r"   \   /     ",
                "    .-.      ",
                " ‒ (   ) ‒   ",
                "    ʻ-ʻ      ",
                r"   /   \     ",
                "             ",
            ],
            apply_hex_color(
                "Weather: clear",
                &config.color.clear,
                config.use_colors,
                true,
            ),
        ),
        "Clouds" => (
            [
                "               ",
                "     .--.      ",
                "  .-(    ).    ",
                " (___.__)__)   ",
                "               ",
                "               ",
                "               ",
            ],
            apply_hex_color(
                "Weather: cloudy",
                &config.color.clouds,
                config.use_colors,
                true,
            ),
        ),
        "Rain" => (
            [
                "               ",
                "     .--.      ",
                "  .-(    ).    ",
                " (___.__)__)   ",
                "  ʻ‚ʻ‚ʻ‚ʻ‚ʻ    ",
                "               ",
                "               ",
            ],
            apply_hex_color(
                "Weather: rainy",
                &config.color.rain,
                config.use_colors,
                true,
            ),
        ),
        "Snow" => (
            [
                "               ",
                "     .--.      ",
                "  .-(    ).    ",
                " (___.__)__)   ",
                "   * * * *     ",
                "  * * * *      ",
                "               ",
            ],
            apply_hex_color(
                "Weather: snowy",
                &config.color.snow,
                config.use_colors,
                true,
            ),
        ),
        "Thunderstorm" => (
            [
                "               ",
                "     .--.      ",
                "  .-(    ).    ",
                " (___.__)__)   ",
                "    /_  /_     ",
                "     /  /      ",
                "               ",
            ],
            apply_hex_color(
                "Weather: thundery",
                &config.color.thunderstorm,
                config.use_colors,
                true,
            ),
        ),
        _ => (
            [
                "               ",
                "     .--.      ",
                "  .-(    ).    ",
                " (___.__)__)   ",
                "               ",
                "               ",
                "               ",
            ],
            apply_hex_color(
                &format!("Weather: {weather}"),
                &config.color.other,
                config.use_colors,
                true,
            ),
        ),
    };

    let date_str = if config.showdate {
        let adjusted_now = now_utc + offset;
        let date = adjusted_now.format("%A, %d %B %Y").to_string();
        let label = apply_hex_color("Date: ", &config.color.date_label, config.use_colors, false);
        let value = apply_hex_color(&date, &config.color.date_value, config.use_colors, false);
        format!("{label}{value}")
    } else {
        String::new()
    };

    let info = [
        city_str,
        weather_str,
        temp_str,
        wind_speed_str,
        sunrise_str,
        sunset_str,
        date_str,
    ];

    art.iter()
        .zip(info.iter())
        .map(|(a, i)| format!("{a}{i}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn main() {
    let config = read_config();

    let client = Client::new();
    let response = client
        .get("https://api.openweathermap.org/data/2.5/weather")
        .query(&[
            ("q", config.city.as_str()),
            ("units", config.units.as_str()),
            ("APPID", config.api_key.as_str()),
        ])
        .send()
        .expect("Failed to send request");

    if response.status().is_success() {
        let weather_data: Value = response.json().expect("Failed to parse JSON");
        let output = format_weather(&weather_data, &config, Utc::now());
        println!("{}", output);
    } else {
        eprintln!(
            "Failed to fetch weather data: Your API key and/or city name are missing from the config file, if they aren't missing, check the spelling of your city here https://openweathermap.org/"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_config_path(test_name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("cuaca_{test_name}_{nanos}.toml"))
    }

    #[test]
    fn creates_default_config_when_missing() {
        let path = temp_config_path("missing");
        let config = load_or_init_config(&path);
        assert_eq!(config, Config::default());
        assert!(path.exists());
        let _ = fs::remove_file(path);
    }

    #[test]
    fn merges_partial_config_with_defaults() {
        let path = temp_config_path("partial");
        fs::write(
            &path,
            r##"api_key = "abc123"
city = "Jakarta"

[color]
temp = "#123456"
"##,
        )
        .unwrap();

        let config = load_or_init_config(&path);
        assert_eq!(config.api_key, "abc123");
        assert_eq!(config.city, "Jakarta");
        assert_eq!(config.units, "metric");
        assert_eq!(config.timeformat, "24");
        assert_eq!(config.color.temp, "#123456");
        assert_eq!(config.color.city, "#34BFA0");

        let persisted = fs::read_to_string(&path).unwrap();
        assert!(persisted.contains(r#"units = "metric""#));
        assert!(persisted.contains(r##"city = "#34BFA0""##));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn formats_metric_weather_with_timezone_offset() {
        let data = json!({
            "weather": [{ "main": "Clear" }],
            "main": { "temp": 28.45 },
            "wind": { "speed": 3.2 },
            "sys": { "sunrise": 0, "sunset": 43200 }
        });
        let config = Config {
            city: String::from("Surabaya"),
            units: String::from("metric"),
            timeplus: 7,
            timeminus: 0,
            showcityname: true,
            showdate: true,
            timeformat: String::from("24"),
            use_colors: false,
            ..Config::default()
        };
        let now = Utc.with_ymd_and_hms(2026, 5, 1, 20, 0, 0).unwrap();

        let output = format_weather(&data, &config, now);
        assert!(output.contains("City: Surabaya"));
        assert!(output.contains("Weather: clear"));
        assert!(output.contains("Temperature: 28.4°C"));
        assert!(output.contains("Wind speed: 3.2 m/s"));
        assert!(output.contains("Sunrise: 07:00"));
        assert!(output.contains("Sunset: 19:00"));
        assert!(output.contains("Date: Saturday, 02 May 2026"));
    }

    #[test]
    fn formats_imperial_12h_and_fallback_weather() {
        let data = json!({
            "weather": [{ "main": "Mist" }],
            "main": { "temp": 72.0 },
            "wind": { "speed": 10.5 },
            "sys": { "sunrise": 0, "sunset": 46800 }
        });
        let config = Config {
            units: String::from("imperial"),
            timeformat: String::from("12"),
            showcityname: false,
            showdate: false,
            ..Config::default()
        };
        let now = Utc.with_ymd_and_hms(2026, 5, 1, 12, 0, 0).unwrap();

        let output = format_weather(&data, &config, now);
        assert!(!output.contains("City:"));
        assert!(output.contains("Weather: Mist"));
        assert!(output.contains("Temperature: 72.0°F"));
        assert!(output.contains("Wind speed: 10.5 mph"));
        assert!(output.contains("Sunrise: 12:00 AM"));
        assert!(output.contains("Sunset: 01:00 PM"));
        assert!(!output.contains("Date:"));
    }

    #[test]
    fn applies_hex_truecolor_and_ignores_invalid_hex() {
        let data = json!({
            "weather": [{ "main": "Rain" }],
            "main": { "temp": 25.0 },
            "wind": { "speed": 5.0 },
            "sys": { "sunrise": 1700000000, "sunset": 1700043200 }
        });

        let mut config = Config {
            use_colors: true,
            ..Config::default()
        };
        config.color.temp = String::from("#E08256");
        config.color.wind = String::from("not-a-hex");

        let now = Utc.with_ymd_and_hms(2025, 1, 15, 12, 0, 0).unwrap();
        let output = format_weather(&data, &config, now);

        assert!(output.contains("\x1b[38;2;224;130;86mTemperature: 25.0°C\x1b[0m"));
        assert!(output.contains("Wind speed: 5.0 m/s"));
        assert!(!output.contains("\x1b[38;2;0;0;0mWind speed:"));
    }
}
