// Authentication utilities and validators
use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    static ref EMAIL_REGEX: Regex = Regex::new(
        r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$"
    ).unwrap();
}

/// Password validation result
#[derive(Debug, Clone)]
pub struct PasswordValidation {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub score: u8,
}

impl PasswordValidation {
    pub fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            score: 0,
        }
    }

    pub fn add_error(&mut self, error: String) {
        self.is_valid = false;
        self.errors.push(error);
    }

    pub fn add_score(&mut self, points: u8) {
        self.score = self.score.saturating_add(points);
    }
}

impl Default for PasswordValidation {
    fn default() -> Self {
        Self::new()
    }
}

/// Validate a password for security requirements
pub fn validate_password(password: &str) -> PasswordValidation {
    let mut validation = PasswordValidation::new();

    // Length check (minimum 12 characters)
    if password.len() < 12 {
        validation.add_error("Password must be at least 12 characters long".to_string());
    } else {
        validation.add_score(20);
    }

    // Length bonus (more than 16 characters)
    if password.len() >= 16 {
        validation.add_score(10);
    }

    // Uppercase letters check
    if !password.chars().any(|c| c.is_uppercase()) {
        validation.add_error("Password must contain at least one uppercase letter".to_string());
    } else {
        validation.add_score(15);
    }

    // Lowercase letters check
    if !password.chars().any(|c| c.is_lowercase()) {
        validation.add_error("Password must contain at least one lowercase letter".to_string());
    } else {
        validation.add_score(15);
    }

    // Numbers check
    if !password.chars().any(|c| c.is_numeric()) {
        validation.add_error("Password must contain at least one digit".to_string());
    } else {
        validation.add_score(15);
    }

    // Special characters check
    let special_chars = "!@#$%^&*()_+-=[]{}|;:,.<>?";
    if !password.chars().any(|c| special_chars.contains(c)) {
        validation.add_error("Password must contain at least one special character (!@#$%^&*...)".to_string());
    } else {
        validation.add_score(20);
    }

    // No common patterns
    let common_patterns = ["password", "123456", "qwerty", "admin", "letmein", "welcome"];
    for pattern in &common_patterns {
        if password.to_lowercase().contains(pattern) {
            validation.add_error(format!("Password contains common pattern: '{}'", pattern));
        }
    }

    // No repeated characters (more than 3 in a row)
    if password.chars().collect::<Vec<_>>()
        .windows(4)
        .any(|w| w[0] == w[1] && w[1] == w[2] && w[2] == w[3])
    {
        validation.add_error("Password contains too many repeated characters".to_string());
    }

    validation
}

/// Validate email address format
pub fn validate_email(email: &str) -> bool {
    EMAIL_REGEX.is_match(email) && email.len() <= 254
}

/// Validate username
pub fn validate_username(username: &str) -> Result<(), String> {
    if username.is_empty() {
        return Err("Username cannot be empty".to_string());
    }

    if username.len() < 3 {
        return Err("Username must be at least 3 characters long".to_string());
    }

    if username.len() > 32 {
        return Err("Username must not exceed 32 characters".to_string());
    }

    if !username
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err("Username can only contain alphanumeric characters, underscores, and hyphens".to_string());
    }

    if !username.chars().next().unwrap().is_alphabetic() {
        return Err("Username must start with a letter".to_string());
    }

    Ok(())
}

/// Track failed login attempts
pub struct LoginAttemptTracker {
    pub max_attempts: u32,
    pub lockout_duration_secs: u64,
}

impl LoginAttemptTracker {
    pub fn new() -> Self {
        Self {
            max_attempts: 5,
            lockout_duration_secs: 900, // 15 minutes
        }
    }

    pub fn should_lock_account(&self, attempt_count: u32) -> bool {
        attempt_count >= self.max_attempts
    }
}

impl Default for LoginAttemptTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strong_password() {
        let validation = validate_password("SecurePass123!@#");
        assert!(validation.is_valid);
        assert!(validation.errors.is_empty());
        assert!(validation.score > 70);
    }

    #[test]
    fn test_weak_password_too_short() {
        let validation = validate_password("Short1!");
        assert!(!validation.is_valid);
        assert!(validation.errors.iter().any(|e| e.contains("12 characters")));
    }

    #[test]
    fn test_weak_password_no_special() {
        let validation = validate_password("WeakPassword123");
        assert!(!validation.is_valid);
        assert!(validation.errors.iter().any(|e| e.contains("special character")));
    }

    #[test]
    fn test_strong_password_with_special() {
        let validation = validate_password("MyP@ssw0rd!Secure");
        assert!(validation.is_valid);
    }

    #[test]
    fn test_valid_email() {
        assert!(validate_email("user@example.com"));
        assert!(validate_email("test.user+tag@sub.example.co.uk"));
        assert!(!validate_email("invalid"));
        assert!(!validate_email("user@"));
    }

    #[test]
    fn test_valid_username() {
        assert!(validate_username("user123").is_ok());
        assert!(validate_username("valid_user").is_ok());
        assert!(validate_username("abc").is_ok());
        assert!(validate_username("ab").is_err());
        assert!(validate_username("123invalid").is_err());
    }

    #[test]
    fn test_login_attempt_tracker() {
        let tracker = LoginAttemptTracker::new();
        assert!(!tracker.should_lock_account(4));
        assert!(tracker.should_lock_account(5));
    }
}
