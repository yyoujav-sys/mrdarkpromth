use argon2::{Argon2, PasswordHash, PasswordVerifier};

fn main() {
    let hash_str = "\\=19\=19456,t=2,p=1\+l7+zw\";
    let password = "admin123";
    
    let argon2 = Argon2::default();
    let parsed_hash = PasswordHash::new(hash_str).expect("Failed to parse hash");
    
    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
    }
}
