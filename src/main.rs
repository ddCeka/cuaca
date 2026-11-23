use chrono::{DateTime, Duration, TimeZone, Utc};
use colored::{Color, Colorize};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
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
        Self {
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

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
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
        Self {
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

    let toml_string = toml::to_string(&config).expect("Failed to serialize config");
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

fn parse_hex_color(hex: &str) -> Option<Color> {
    let cleaned = hex.trim().strip_prefix('#').unwrap_or(hex.trim());
    if cleaned.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&cleaned[0..2], 16).ok()?;
    let g = u8::from_str_radix(&cleaned[2..4], 16).ok()?;
    let b = u8::from_str_radix(&cleaned[4..6], 16).ok()?;
    Some(Color::TrueColor { r, g, b })
}

fn style_text(text: &str, hex: &str, use_colors: bool, bold: bool) -> String {
    if !use_colors {
        return text.to_string();
    }
    let styled = match parse_hex_color(hex) {
        Some(color) => text.color(color),
        None => text.normal(),
    };
    if bold {
        styled.bold().to_string()
    } else {
        styled.to_string()
    }
}

fn format_weather(weather_data: &Value, config: &Config, now: DateTime<Utc>) -> String {
    let weather = weather_data["weather"][0]["main"]
        .as_str()
        .unwrap_or("Unknown");
    let temp = weather_data["main"]["temp"].as_f64().unwrap_or(0.0);
    let wind_speed = weather_data["wind"]["speed"].as_f64().unwrap_or(0.0);
    let sunrise = weather_data["sys"]["sunrise"].as_i64().unwrap_or(0);
    let sunset = weather_data["sys"]["sunset"].as_i64().unwrap_or(0);

    let (windspeed_unit, temp_unit) = match config.units.as_str() {
        "imperial" => ("mph", "°F"),
        "metric" => ("m/s", "°C"),
        _ => ("m/s", "K"),
    };

    let temp_str = style_text(
        &format!("Temperature: {:.1}{}", temp, temp_unit),
        &config.color.temp,
        config.use_colors,
        false,
    );

    let wind_speed_str = style_text(
        &format!("Wind speed: {:.1} {}", wind_speed, windspeed_unit),
        &config.color.wind,
        config.use_colors,
        false,
    );

    let offset = Duration::hours(config.timeplus) - Duration::hours(config.timeminus);
    let sunrise_datetime = Utc.timestamp_opt(sunrise, 0).unwrap() + offset;
    let sunset_datetime = Utc.timestamp_opt(sunset, 0).unwrap() + offset;

    let time_format = match config.timeformat.as_str() {
        "12" => "%I:%M %p",
        _ => "%H:%M",
    };

    let sunrisestring = sunrise_datetime.format(time_format).to_string();
    let sunsetstring = sunset_datetime.format(time_format).to_string();

    let city_str = if config.showcityname {
        style_text(
            &format!("City: {}", config.city),
            &config.color.city,
            config.use_colors,
            true,
        )
    } else {
        String::new()
    };

    let sunrise_str = style_text(
        &format!("Sunrise: {sunrisestring}"),
        &config.color.sunrise,
        config.use_colors,
        false,
    );

    let sunset_str = style_text(
        &format!("Sunset: {sunsetstring}"),
        &config.color.sunset,
        config.use_colors,
        false,
    );

    let (art, weather_text, weather_hex) = match weather {
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
            "Weather: clear".to_string(),
            config.color.clear.as_str(),
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
            "Weather: cloudy".to_string(),
            config.color.clouds.as_str(),
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
            "Weather: rainy".to_string(),
            config.color.rain.as_str(),
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
            "Weather: snowy".to_string(),
            config.color.snow.as_str(),
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
            "Weather: thundery".to_string(),
            config.color.thunderstorm.as_str(),
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
            format!("Weather: {weather}"),
            config.color.other.as_str(),
        ),
    };

    let weather_str = style_text(&weather_text, weather_hex, config.use_colors, true);

    let date_str = if config.showdate {
        let formatted_date = (now + offset).format("%A, %d %B %Y").to_string();
        let label = style_text("Date: ", &config.color.date_label, config.use_colors, false);
        let value = style_text(
            &formatted_date,
            &config.color.date_value,
            config.use_colors,
            false,
        );
        format!("{label}{value}")
    } else {
        String::new()
    };

    let details = [
        city_str,
        weather_str,
        temp_str,
        wind_speed_str,
        sunrise_str,
        sunset_str,
        date_str,
    ];

    art.iter()
        .zip(details.iter())
        .map(|(prefix, detail)| format!("{prefix}{detail}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn main() {
    let config = read_config();

    let response = Client::new()
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
        println!("{}", format_weather(&weather_data, &config, Utc::now()));
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

    fn temp_config_file(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir()
            .join(format!("cuaca_test_{name}_{nanos}"))
            .join("cuaca.toml")
    }

    #[test]
    fn creates_default_config_when_missing() {
        let path = temp_config_file("missing");
        let config = load_or_init_config(&path);

        assert_eq!(config, Config::default());
        assert!(path.exists());

        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn merges_partial_config_with_defaults() {
        let path = temp_config_file("partial");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "api_key = \"test_key\"\ncity = \"Surabaya\"\n").unwrap();

        let config = load_or_init_config(&path);

        assert_eq!(config.api_key, "test_key");
        assert_eq!(config.city, "Surabaya");
        assert_eq!(config.units, "metric");
        assert_eq!(config.timeformat, "24");
        assert_eq!(config.color, ColorConfig::default());

        let saved = fs::read_to_string(&path).unwrap();
        assert!(saved.contains("[color]"));
        assert!(saved.contains("temp = \"#E08256\""));

        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn formats_metric_weather_with_timezone_offset() {
        let config = Config {
            city: "Surabaya".to_string(),
            units: "metric".to_string(),
            timeplus: 7,
            timeminus: 0,
            showcityname: true,
            showdate: true,
            timeformat: "24".to_string(),
            use_colors: false,
            ..Config::default()
        };

        let sample = json!({
            "weather": [{ "main": "Clear" }],
            "main": { "temp": 30.4 },
            "wind": { "speed": 3.2 },
            "sys": { "sunrise": 1700000000, "sunset": 1700043200 }
        });

        let now = Utc.with_ymd_and_hms(2026, 11, 23, 20, 0, 0).unwrap();
        let output = format_weather(&sample, &config, now);

        assert!(output.contains("City: Surabaya"));
        assert!(output.contains("Weather: clear"));
        assert!(output.contains("Temperature: 30.4°C"));
        assert!(output.contains("Wind speed: 3.2 m/s"));
        assert!(output.contains("Sunrise: 05:13"));
        assert!(output.contains("Sunset: 17:13"));
        assert!(output.contains("Date: Tuesday, 24 November 2026"));
    }

    #[test]
    fn formats_imperial_12h_and_fallback_weather() {
        let config = Config {
            city: "New York".to_string(),
            units: "imperial".to_string(),
            showcityname: false,
            showdate: false,
            timeformat: "12".to_string(),
            use_colors: false,
            ..Config::default()
        };

        let sample = json!({
            "weather": [{ "main": "Mist" }],
            "main": { "temp": 68.0 },
            "wind": { "speed": 10.5 },
            "sys": { "sunrise": 1700000000, "sunset": 1700043200 }
        });

        let now = Utc.with_ymd_and_hms(2026, 11, 23, 12, 0, 0).unwrap();
        let output = format_weather(&sample, &config, now);

        assert!(!output.contains("City:"));
        assert!(!output.contains("Date:"));
        assert!(output.contains("Weather: Mist"));
        assert!(output.contains("Temperature: 68.0°F"));
        assert!(output.contains("Wind speed: 10.5 mph"));
        assert!(output.contains("Sunrise: 10:13 PM"));
        assert!(output.contains("Sunset: 10:13 AM"));
    }

    #[test]
    fn applies_hex_truecolor_and_ignores_invalid_hex() {
        colored::control::set_override(true);

        let mut config = Config {
            use_colors: true,
            ..Config::default()
        };
        config.color.temp = "#E08256".to_string();
        config.color.wind = "not-a-hex".to_string();

        let sample = json!({
            "weather": [{ "main": "Rain" }],
            "main": { "temp": 25.0 },
            "wind": { "speed": 4.0 },
            "sys": { "sunrise": 1700000000, "sunset": 1700043200 }
        });

        let now = Utc.with_ymd_and_hms(2026, 11, 23, 12, 0, 0).unwrap();
        let output = format_weather(&sample, &config, now);

        assert!(output.contains("\x1b[38;2;224;130;86mTemperature: 25.0°C\x1b[0m"));
        assert!(output.contains("Wind speed: 4.0 m/s"));
        assert!(!output.contains("\x1b[38;2;52;191;160mWind speed:"));

        colored::control::unset_override();
    }
}
