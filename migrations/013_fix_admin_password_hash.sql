-- Fix admin password hash if placeholder was inserted by earlier migration
UPDATE users
SET password_hash = '$argon2id$v=19$m=19456,t=2,p=1$PaqTPuSDTe9S6OsT+l7+zw$ScE7QbLywnTOe59cF0csKTchlCBKY8DIneQwNAFcbdg'
WHERE email = 'admin@mrdarkpromth.ai'
  AND password_hash LIKE '%example_hash_replace_in_app%';
