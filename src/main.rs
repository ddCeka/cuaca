use chrono::{DateTime, Duration, TimeZone, Utc};
use colored::Colorize;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::fs::File;
use std::io::Write;

#[derive(Deserialize, Serialize)]
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
        }
    }
}

fn read_config() -> Config {
    let mut config_path = dirs::config_dir().expect("Failed to get config directory");
    config_path.push("cuaca");
    config_path.push("cuaca.toml");

    if !config_path.exists() {
        let default_config = Config::default();
        let toml_string =
            toml::to_string(&default_config).expect("Failed to serialize default config");
        fs::create_dir_all(config_path.parent().unwrap())
            .expect("Failed to create config directory");
        let mut file = File::create(&config_path).expect("Failed to create config file");
        file.write_all(toml_string.as_bytes())
            .expect("Failed to write default config to file");

        let config_location = config_path.display().to_string();
        println!("No config detected, config made at {}.", config_location);
        return default_config;
    }

    let config_content = fs::read_to_string(&config_path).expect("Failed to read config file");

    if let Ok(config) = toml::from_str::<Config>(&config_content) {
        return config;
    }

    let config = Config::default();
    let toml_string = toml::to_string(&config).expect("Failed to serialize merged config");
    let mut file = File::create(&config_path).expect("Failed to create config file");
    file.write_all(toml_string.as_bytes())
        .expect("Failed to write merged config to file");

    config
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

        let temp_str = if config.use_colors {
            format!("Temperature: {:.1}{}", temp, temp_unit)
                .red()
                .to_string()
        } else {
            format!("Temperature: {:.1}{}", temp, temp_unit)
        };

        let wind_speed_str = if config.use_colors {
            format!("Wind speed: {:.1} {}", wind_speed, windspeedunits)
                .green()
                .to_string()
        } else {
            format!("Wind speed: {:.1} {}", wind_speed, windspeedunits)
        };

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

        let date = if config.showdate {
            let now = Utc::now() + offset;
            now.format("%A, %d %B %Y").to_string()
        } else {
            String::new()
        };

        let date_label = if config.showdate { "Date: " } else { "" };
        let date_value = if config.showdate { date } else { String::new() };

        let city_str = if config.use_colors && config.showcityname {
            format!("City: {}", config.city).bold().green().to_string()
        } else if config.showcityname {
            format!("City: {}", config.city)
        } else {
            String::new()
        };

        let sunrise_str = if config.use_colors {
            format!("Sunrise: {sunrisestring}").yellow().to_string()
        } else {
            format!("Sunrise: {sunrisestring}")
        };

        let sunset_str = if config.use_colors {
            format!("Sunset: {sunsetstring}").blue().to_string()
        } else {
            format!("Sunset: {sunsetstring}")
        };

        let (art, weather_label, date_str) = match weather {
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
                if config.use_colors {
                    "Weather: clear".bold().white().to_string()
                } else {
                    "Weather: clear".to_string()
                },
                if config.use_colors {
                    format!("{}{}", date_label.cyan(), date_value.cyan())
                } else {
                    format!("{date_label}{date_value}")
                },
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
                if config.use_colors {
                    "Weather: cloudy".bold().white().to_string()
                } else {
                    "Weather: cloudy".to_string()
                },
                if config.use_colors {
                    format!("{}{}", date_label.cyan(), date_value.cyan())
                } else {
                    format!("{date_label}{date_value}")
                },
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
                if config.use_colors {
                    "Weather: rainy".bold().blue().to_string()
                } else {
                    "Weather: rainy".to_string()
                },
                if config.use_colors {
                    format!("{}{}", date_label.cyan(), date_value.cyan())
                } else {
                    format!("{date_label}{date_value}")
                },
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
                if config.use_colors {
                    "Weather: snowy".bold().white().to_string()
                } else {
                    "Weather: snowy".to_string()
                },
                if config.use_colors {
                    format!("{}{}", date_label.cyan(), date_value.cyan())
                } else {
                    format!("{date_label}{date_value}")
                },
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
                if config.use_colors {
                    "Weather: thundery".bold().white().to_string()
                } else {
                    "Weather: thundery".to_string()
                },
                if config.use_colors {
                    format!("{}{}", date_label.cyan(), date_value.cyan())
                } else {
                    format!("{date_label}{date_value}")
                },
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
                if config.use_colors {
                    format!("Weather: {weather}").bold().red().to_string()
                } else {
                    format!("Weather: {weather}")
                },
                if config.use_colors {
                    format!("{}{}", date_label.black(), date_value.magenta())
                } else {
                    format!("{date_label}{date_value}")
                },
            ),
        };

        let details = [
            city_str,
            weather_label,
            temp_str,
            wind_speed_str,
            sunrise_str,
            sunset_str,
            date_str,
        ];

        for (art_line, detail) in art.iter().zip(details.iter()) {
            println!("{art_line}{detail}");
        }
    } else {
        eprintln!(
            "Failed to fetch weather data: Your API key and/or city name are missing from the config file, if they aren't missing, check the spelling of your city here https://openweathermap.org/"
        );
    }
}
