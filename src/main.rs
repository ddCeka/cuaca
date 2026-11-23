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

    let merged_config: Config = toml::from_str(&config_content).unwrap_or_default();
    let toml_string =
        toml::to_string(&merged_config).expect("Failed to serialize merged config");
    let mut file = File::create(&config_path).expect("Failed to create config file");
    file.write_all(toml_string.as_bytes())
        .expect("Failed to write merged config to file");

    merged_config
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

        let sunrise_datetime: DateTime<Utc> = Utc.timestamp_opt(sunrise, 0).unwrap();
        let sunset_datetime: DateTime<Utc> = Utc.timestamp_opt(sunset, 0).unwrap();

        let adjusted_sunrise =
            sunrise_datetime + Duration::hours(config.timeplus) - Duration::hours(config.timeminus);
        let adjusted_sunset =
            sunset_datetime + Duration::hours(config.timeplus) - Duration::hours(config.timeminus);

        let time_format = match config.timeformat.as_str() {
            "12" => "%I:%M %p",
            _ => "%H:%M",
        };

        let sunrisestring = adjusted_sunrise.format(time_format).to_string();
        let sunsetstring = adjusted_sunset.format(time_format).to_string();

        let date = if config.showdate {
            let now = Utc::now();
            now.format("%A, %d %B %Y").to_string()
        } else {
            String::new()
        };

        let date_label = if config.showdate { "Date: " } else { "" };
        let date_value = if config.showdate { date } else { String::new() };

        let output = match weather {
            "Clear" => format!(
                r#"             {}
   \   /     {}
    .-.      {}
 ‒ (   ) ‒   {}
    ʻ-ʻ      {}
   /   \     {}
             {}{}"#,
                if config.use_colors && config.showcityname {
                    format!("City: {}", config.city).bold().green().to_string()
                } else if config.showcityname {
                    format!("City: {}", config.city).to_string()
                } else {
                    String::new()
                },
                if config.use_colors {
                    "Weather: clear".bold().white().to_string()
                } else {
                    "Weather: clear".to_string()
                },
                temp_str,
                wind_speed_str,
                if config.use_colors {
                    format!("Sunrise: {sunrisestring}").yellow().to_string()
                } else {
                    format!("Sunrise: {sunrisestring}")
                },
                if config.use_colors {
                    format!("Sunset: {sunsetstring}").blue().to_string()
                } else {
                    format!("Sunset: {sunsetstring}")
                },
                if config.use_colors {
                    date_label.cyan().to_string()
                } else {
                    date_label.to_string()
                },
                if config.use_colors {
                    date_value.cyan().to_string()
                } else {
                    date_value.to_string()
                }
            ),
            "Clouds" => format!(
                r#"               {}
     .--.      {}
  .-(    ).    {}
 (___.__)__)   {}
               {}
               {}
               {}{}"#,
                if config.use_colors && config.showcityname {
                    format!("City: {}", config.city).bold().green().to_string()
                } else if config.showcityname {
                    format!("City: {}", config.city).to_string()
                } else {
                    String::new()
                },
                if config.use_colors {
                    "Weather: cloudy".bold().white().to_string()
                } else {
                    "Weather: cloudy".to_string()
                },
                temp_str,
                wind_speed_str,
                if config.use_colors {
                    format!("Sunrise: {sunrisestring}").yellow().to_string()
                } else {
                    format!("Sunrise: {sunrisestring}")
                },
                if config.use_colors {
                    format!("Sunset: {sunsetstring}").blue().to_string()
                } else {
                    format!("Sunset: {sunsetstring}")
                },
                if config.use_colors {
                    date_label.cyan().to_string()
                } else {
                    date_label.to_string()
                },
                if config.use_colors {
                    date_value.cyan().to_string()
                } else {
                    date_value.to_string()
                }
            ),
            "Rain" => format!(
                r#"               {}
     .--.      {}
  .-(    ).    {}
 (___.__)__)   {}
  ʻ‚ʻ‚ʻ‚ʻ‚ʻ    {}
               {}
               {}{}"#,
                if config.use_colors && config.showcityname {
                    format!("City: {}", config.city).bold().green().to_string()
                } else if config.showcityname {
                    format!("City: {}", config.city).to_string()
                } else {
                    String::new()
                },
                if config.use_colors {
                    "Weather: rainy".bold().blue().to_string()
                } else {
                    "Weather: rainy".to_string()
                },
                temp_str,
                wind_speed_str,
                if config.use_colors {
                    format!("Sunrise: {sunrisestring}").yellow().to_string()
                } else {
                    format!("Sunrise: {sunrisestring}")
                },
                if config.use_colors {
                    format!("Sunset: {sunsetstring}").blue().to_string()
                } else {
                    format!("Sunset: {sunsetstring}")
                },
                if config.use_colors {
                    date_label.cyan().to_string()
                } else {
                    date_label.to_string()
                },
                if config.use_colors {
                    date_value.cyan().to_string()
                } else {
                    date_value.to_string()
                }
            ),
            "Snow" => format!(
                r#"               {}
     .--.      {}
  .-(    ).    {}
 (___.__)__)   {}
   * * * *     {}
  * * * *      {}
               {}{}"#,
                if config.use_colors && config.showcityname {
                    format!("City: {}", config.city).bold().green().to_string()
                } else if config.showcityname {
                    format!("City: {}", config.city).to_string()
                } else {
                    String::new()
                },
                if config.use_colors {
                    "Weather: snowy".bold().white().to_string()
                } else {
                    "Weather: snowy".to_string()
                },
                temp_str,
                wind_speed_str,
                if config.use_colors {
                    format!("Sunrise: {sunrisestring}").yellow().to_string()
                } else {
                    format!("Sunrise: {sunrisestring}")
                },
                if config.use_colors {
                    format!("Sunset: {sunsetstring}").blue().to_string()
                } else {
                    format!("Sunset: {sunsetstring}")
                },
                if config.use_colors {
                    date_label.cyan().to_string()
                } else {
                    date_label.to_string()
                },
                if config.use_colors {
                    date_value.cyan().to_string()
                } else {
                    date_value.to_string()
                }
            ),
            "Thunderstorm" => format!(
                r#"               {}
     .--.      {}
  .-(    ).    {}
 (___.__)__)   {}
    /_  /_     {}
     /  /      {}
               {}{}"#,
                if config.use_colors && config.showcityname {
                    format!("City: {}", config.city).bold().green().to_string()
                } else if config.showcityname {
                    format!("City: {}", config.city).to_string()
                } else {
                    String::new()
                },
                if config.use_colors {
                    "Weather: thundery".bold().white().to_string()
                } else {
                    "Weather: thundery".to_string()
                },
                temp_str,
                wind_speed_str,
                if config.use_colors {
                    format!("Sunrise: {sunrisestring}").yellow().to_string()
                } else {
                    format!("Sunrise: {sunrisestring}")
                },
                if config.use_colors {
                    format!("Sunset: {sunsetstring}").blue().to_string()
                } else {
                    format!("Sunset: {sunsetstring}")
                },
                if config.use_colors {
                    date_label.cyan().to_string()
                } else {
                    date_label.to_string()
                },
                if config.use_colors {
                    date_value.cyan().to_string()
                } else {
                    date_value.to_string()
                }
            ),
            _ => format!(
                r#"               {}
     .--.      {}
  .-(    ).    {}
 (___.__)__)   {}
               {}
               {}
               {}{}"#,
                if config.use_colors && config.showcityname {
                    format!("City: {}", config.city).bold().green().to_string()
                } else if config.showcityname {
                    format!("City: {}", config.city).to_string()
                } else {
                    String::new()
                },
                if config.use_colors {
                    format!("Weather: {weather}").bold().red().to_string()
                } else {
                    format!("Weather: {weather}")
                },
                temp_str,
                wind_speed_str,
                if config.use_colors {
                    format!("Sunrise: {sunrisestring}").yellow().to_string()
                } else {
                    format!("Sunrise: {sunrisestring}")
                },
                if config.use_colors {
                    format!("Sunset: {sunsetstring}").blue().to_string()
                } else {
                    format!("Sunset: {sunsetstring}")
                },
                if config.use_colors {
                    date_label.black().to_string()
                } else {
                    date_label.to_string()
                },
                if config.use_colors {
                    date_value.magenta().to_string()
                } else {
                    date_value.to_string()
                }
            ),
        };
        println!("{}", output);
    } else {
        eprintln!(
            "Failed to fetch weather data: Your API key and/or city name are missing from the config file, if they aren't missing, check the spelling of your city here https://openweathermap.org/"
        );
    }
}
