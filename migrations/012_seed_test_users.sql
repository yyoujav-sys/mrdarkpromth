-- Seed the testultra user for development and testing

INSERT INTO users (id, username, email, tier, password_hash, api_key)
VALUES ('259d30cc-b6d9-4ffe-a9ce-88cd3224ceb9', 'testultra', 'testultra@example.com', 'ultra', '$argon2id$v=19$m=4096,t=3,p=1$ieeR5CEUibIZvMSwTYBOmQ$WjjZfhcjNNfyqom3B6rDBuCSPrXQxujl8U+NHz29GwQ', gen_random_uuid())
ON CONFLICT (id) DO NOTHING;
