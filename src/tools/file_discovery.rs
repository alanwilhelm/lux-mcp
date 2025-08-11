use std::path::Path;
use glob::glob;
use tracing::info;

/// Auto-discover relevant files based on the message content
pub fn discover_files(message: &str, base_path: Option<&str>) -> Vec<String> {
    let mut discovered = Vec::new();
    let base = base_path.unwrap_or(".");
    
    // Keywords that suggest what files to look for
    let keywords: Vec<(&str, &[&str])> = vec![
        ("package", &["package.json", "Cargo.toml", "pyproject.toml"]),
        ("config", &["*.config.js", "*.config.ts", ".env", "config/*"]),
        ("test", &["*.test.js", "*.spec.ts", "test/*", "tests/*"]),
        ("api", &["*api*.js", "*route*.js", "api/*", "routes/*"]),
        ("database", &["*schema*", "*model*", "migrations/*"]),
        ("auth", &["*auth*", "*login*", "*session*"]),
    ];
    
    // Check message for keywords and find matching files
    for (keyword, patterns) in keywords {
        if message.to_lowercase().contains(keyword) {
            for pattern in patterns {
                let full_pattern = format!("{}/{}", base, pattern);
                if let Ok(paths) = glob(&full_pattern) {
                    for path in paths.flatten() {
                        if let Some(path_str) = path.to_str() {
                            if Path::new(path_str).is_file() {
                                discovered.push(path_str.to_string());
                                info!("Auto-discovered file: {}", path_str);
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Limit to first 5 files to avoid token explosion
    discovered.truncate(5);
    discovered
}