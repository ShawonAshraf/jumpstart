use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Serialize, Clone)]
pub enum Side {
    Left,
    Right,
}

impl<'de> Deserialize<'de> for Side {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "left" => Ok(Side::Left),
            "right" => Ok(Side::Right),
            _ => Err(serde::de::Error::custom(format!("Invalid side: {}", s))),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Application {
    pub name: String,
    pub display: u32,
    pub side: Side,
    pub executable: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub applications: Vec<Application>,
}

pub fn load_config() -> Result<Config, String> {
    let yaml_content = std::fs::read_to_string("config.yml")
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    serde_yaml::from_str(&yaml_content).map_err(|e| format!("Failed to parse config: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_side_deserialization() {
        let yaml_left = "left";
        let side_left: Side = serde_yaml::from_str(yaml_left).unwrap();
        assert!(matches!(side_left, Side::Left));

        let yaml_right = "right";
        let side_right: Side = serde_yaml::from_str(yaml_right).unwrap();
        assert!(matches!(side_right, Side::Right));

        let yaml_invalid = "invalid";
        let result: Result<Side, _> = serde_yaml::from_str(yaml_invalid);
        assert!(result.is_err());
    }

    #[test]
    fn test_config_loading() {
        // Create a temporary config file for testing
        let test_config = r#"
applications:
  - name: "Test App"
    display: 1
    side: "left"
    executable: "test.exe"
"#;

        let temp_file_path = "test_config.yml";
        fs::write(temp_file_path, test_config).unwrap();

        // Temporarily rename the original config file if it exists
        let original_config_exists = std::path::Path::new("config.yml").exists();
        if original_config_exists {
            fs::rename("config.yml", "config.yml.bak")
                .expect("Failed to backup original config file");
        }

        // Rename our test file to config.yml
        fs::rename(temp_file_path, "config.yml").unwrap();

        let config = load_config().unwrap();
        assert_eq!(config.applications.len(), 1);
        assert_eq!(config.applications[0].name, "Test App");
        assert_eq!(config.applications[0].display, 1);
        assert!(matches!(config.applications[0].side, Side::Left));
        assert_eq!(config.applications[0].executable, "test.exe");

        // Clean up
        if let Err(e) = fs::remove_file("config.yml") {
            eprintln!("Warning: Failed to remove test config file: {}", e);
        }

        // Restore original config if it existed
        if original_config_exists && let Err(e) = fs::rename("config.yml.bak", "config.yml") {
            eprintln!("Warning: Failed to restore original config file: {}", e);
        }
    }

    #[test]
    fn test_config_loading_invalid_file() {
        // Temporarily rename the original config file if it exists
        let original_config_exists = std::path::Path::new("config.yml").exists();
        if original_config_exists {
            fs::rename("config.yml", "config.yml.bak")
                .expect("Failed to backup original config file");
        }

        let result = load_config();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to read config file"));

        // Restore original config if it existed
        if original_config_exists {
            fs::rename("config.yml.bak", "config.yml").unwrap();
        }
    }
}
