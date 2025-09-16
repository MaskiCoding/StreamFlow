use regex::Regex;
use std::sync::OnceLock;

// Regex patterns compiled once and reused
static TWITCH_FULL_REGEX: OnceLock<Regex> = OnceLock::new();
static TWITCH_URL_FULL_REGEX: OnceLock<Regex> = OnceLock::new();
static TWITCH_URL_PARTIAL_REGEX: OnceLock<Regex> = OnceLock::new();
static USERNAME_REGEX: OnceLock<Regex> = OnceLock::new();

const CHANNEL_NAME_PATTERN: &str = r"[a-zA-Z0-9_]{3,25}";

pub struct TwitchValidator;

impl TwitchValidator {
    pub fn is_valid_url(url: &str) -> bool {
        // First try to normalize the URL, then check if it's valid
        let normalized = Self::normalize_url(url);
        let regex = TWITCH_FULL_REGEX.get_or_init(|| {
            Regex::new(&format!(r"^https://www\.twitch\.tv/{}/?$", CHANNEL_NAME_PATTERN))
                .expect("Failed to compile Twitch URL regex")
        });
        
        regex.is_match(&normalized)
    }

    pub fn extract_channel_name(url: &str) -> Option<String> {
        let trimmed = url.trim();
        
        // Handle various input formats:
        // 1. Full URLs: https://twitch.tv/summit1g, https://www.twitch.tv/summit1g
        // 2. Partial URLs: twitch.tv/summit1g, www.twitch.tv/summit1g
        // 3. Just username: summit1g
        // 4. Dashboard URLs: https://www.twitch.tv/dashboard
        // 5. Search result URLs: https://www.twitch.tv/search?term=summit1g
        
        // Try dashboard URL pattern
        if trimmed.contains("/dashboard") {
            // Extract username from dashboard URL
            let dashboard_regex = Regex::new(r"^https?://(www\.)?twitch\.tv/([^/]+)/dashboard").ok()?;
            if let Some(captures) = dashboard_regex.captures(trimmed) {
                return captures.get(2).map(|m| m.as_str().to_owned());
            }
        }
        
        // First try full URL pattern
        let full_regex = TWITCH_URL_FULL_REGEX.get_or_init(|| {
            Regex::new(&format!(r"^https?://(www\.)?twitch\.tv/({})/?$", CHANNEL_NAME_PATTERN))
                .expect("Failed to compile full URL regex")
        });
        
        if let Some(captures) = full_regex.captures(trimmed) {
            return captures.get(2).map(|m| m.as_str().to_owned());
        }
        
        // Try partial URL pattern (without protocol)
        let partial_regex = TWITCH_URL_PARTIAL_REGEX.get_or_init(|| {
            Regex::new(&format!(r"^(www\.)?twitch\.tv/({})/?$", CHANNEL_NAME_PATTERN))
                .expect("Failed to compile partial URL regex")
        });
        
        if let Some(captures) = partial_regex.captures(trimmed) {
            return captures.get(2).map(|m| m.as_str().to_owned());
        }
        
        // Try just username pattern
        let username_regex = USERNAME_REGEX.get_or_init(|| {
            Regex::new(&format!(r"^{}$", CHANNEL_NAME_PATTERN))
                .expect("Failed to compile username regex")
        });
        
        if username_regex.is_match(trimmed) {
            return Some(trimmed.to_owned());
        }
        
        None
    }

    pub fn normalize_url(url: &str) -> String {
        match Self::extract_channel_name(url) {
            Some(channel) => format!("https://www.twitch.tv/{}", channel),
            None => url.trim().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_urls() {
        // Full URLs with protocol
        assert!(TwitchValidator::is_valid_url("https://www.twitch.tv/summit1g"));
        assert!(TwitchValidator::is_valid_url("https://twitch.tv/summit1g"));
        assert!(TwitchValidator::is_valid_url("http://www.twitch.tv/summit1g"));
        assert!(TwitchValidator::is_valid_url("https://www.twitch.tv/summit1g/"));
        
        // Dashboard URLs should now work
        assert!(TwitchValidator::is_valid_url("https://www.twitch.tv/summit1g/dashboard"));
        assert!(TwitchValidator::is_valid_url("https://twitch.tv/shroud/dashboard"));
        
        // Partial URLs without protocol
        assert!(TwitchValidator::is_valid_url("www.twitch.tv/summit1g"));
        assert!(TwitchValidator::is_valid_url("twitch.tv/summit1g"));
        assert!(TwitchValidator::is_valid_url("twitch.tv/summit1g/"));
        
        // Just username
        assert!(TwitchValidator::is_valid_url("summit1g"));
        assert!(TwitchValidator::is_valid_url("shroud"));
        assert!(TwitchValidator::is_valid_url("pokimane"));
        
        // 3-letter usernames (should now work)
        assert!(TwitchValidator::is_valid_url("xqc"));
        assert!(TwitchValidator::is_valid_url("https://twitch.tv/xqc"));
        assert!(TwitchValidator::is_valid_url("twitch.tv/xqc"));
        
        // 25-character username (maximum allowed)
        assert!(TwitchValidator::is_valid_url("abcdefghijklmnopqrstuvwxy"));
        assert!(TwitchValidator::is_valid_url("twitch.tv/abcdefghijklmnopqrstuvwxy"));
    }

    #[test]
    fn test_invalid_urls() {
        // Too short (less than 3 characters)
        assert!(!TwitchValidator::is_valid_url("ab"));
        assert!(!TwitchValidator::is_valid_url("https://twitch.tv/ab"));
        
        // Too long (more than 25 characters)
        assert!(!TwitchValidator::is_valid_url("abcdefghijklmnopqrstuvwxyz"));
        assert!(!TwitchValidator::is_valid_url("twitch.tv/abcdefghijklmnopqrstuvwxyz"));
        
        // Invalid characters
        assert!(!TwitchValidator::is_valid_url("user-name"));
        assert!(!TwitchValidator::is_valid_url("user.name"));
        assert!(!TwitchValidator::is_valid_url("user@name"));
        assert!(!TwitchValidator::is_valid_url("twitch.tv/user-name"));
        
        // Wrong domain
        assert!(!TwitchValidator::is_valid_url("https://youtube.com/summit1g"));
        assert!(!TwitchValidator::is_valid_url("https://twitch.com/summit1g"));
        
        // Empty/whitespace
        assert!(!TwitchValidator::is_valid_url(""));
        assert!(!TwitchValidator::is_valid_url("   "));
        
        // Special Twitch pages (not user channels)
        assert!(!TwitchValidator::is_valid_url("https://www.twitch.tv/directory"));
        assert!(!TwitchValidator::is_valid_url("twitch.tv/settings"));
        
        // Numbers only or starting with numbers (while valid, might want to test)
        assert!(TwitchValidator::is_valid_url("123456"));
        assert!(TwitchValidator::is_valid_url("1summit1g"));
    }

    #[test]
    fn test_channel_extraction() {
        // Full URLs
        assert_eq!(TwitchValidator::extract_channel_name("https://www.twitch.tv/summit1g"), Some("summit1g".to_string()));
        assert_eq!(TwitchValidator::extract_channel_name("https://twitch.tv/shroud"), Some("shroud".to_string()));
        assert_eq!(TwitchValidator::extract_channel_name("http://www.twitch.tv/pokimane/"), Some("pokimane".to_string()));
        
        // Partial URLs
        assert_eq!(TwitchValidator::extract_channel_name("www.twitch.tv/xqc"), Some("xqc".to_string()));
        assert_eq!(TwitchValidator::extract_channel_name("twitch.tv/ninja"), Some("ninja".to_string()));
        assert_eq!(TwitchValidator::extract_channel_name("twitch.tv/tfue/"), Some("tfue".to_string()));
        
        // Just usernames
        assert_eq!(TwitchValidator::extract_channel_name("summit1g"), Some("summit1g".to_string()));
        assert_eq!(TwitchValidator::extract_channel_name("shroud"), Some("shroud".to_string()));
        
        // Invalid cases
        assert_eq!(TwitchValidator::extract_channel_name(""), None);
        assert_eq!(TwitchValidator::extract_channel_name("https://youtube.com/summit1g"), None);
        assert_eq!(TwitchValidator::extract_channel_name("ab"), None); // Too short
        assert_eq!(TwitchValidator::extract_channel_name("user-name"), None); // Invalid character
    }

    #[test]
    fn test_url_normalization() {
        // Various inputs should all normalize to same format
        let inputs = [
            "summit1g",
            "twitch.tv/summit1g",
            "www.twitch.tv/summit1g",
            "https://twitch.tv/summit1g",
            "https://www.twitch.tv/summit1g",
            "https://www.twitch.tv/summit1g/",
        ];
        
        let expected = "https://www.twitch.tv/summit1g";
        
        for input in &inputs {
            assert_eq!(TwitchValidator::normalize_url(input), expected);
        }
        
        // Invalid inputs should return the original trimmed string
        assert_eq!(TwitchValidator::normalize_url("invalid-url"), "invalid-url");
        assert_eq!(TwitchValidator::normalize_url("  spaces  "), "spaces");
    }
}
