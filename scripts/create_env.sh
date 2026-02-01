#!/bin/bash
# สร้าง .env บน VPS - รันบน VPS

cd /opt/mrdarkpromth

# ใช้ printf แทน heredoc
printf '%s\n' \
'# MR.DarkPromth Production Environment' \
'' \
'CEREBRAS_API_KEYS=csk-fev4rctnpryerfxf55p2cwv2nyc4d8tjkvhxjpky4fycp99r,csk-284fp3mdj54tmmdtxxvd9cfrpt39re6v6xreett94mtkppk8,csk-y49knc3fd9cpm9wjrwmhhxj65rvhhndpfhdefexp8dryy6n8,csk-h6xv9vnvy8dre8hmxwfyvjn8vwvry963yrkyt938h9ptxxnn,csk-v6t43y6y5hc6jjc2h445j48txd2d6dtrvj63fdx9xwjd5t28,csk-8rfetm6mt2k5ne8cyc3xe9fhkwenj86x5npw3dyrt9f25prx,csk-ddn62jpy9x6vr4mmden6ee9hfnhfxhcwxcmp535mem54t8dt,csk-t5yx9xt59yv3n49jne332wmf8kcfyykxnn2y2f2my59f29dt,csk-ht3699ypttd6hc5m4yndx54crp2em6h25jmn4dpwrnc899xj,csk-nj6xtf2yddndxwvcynke9kty6r6kyhtf6xftm2wjevj395cv,csk-pm3cehrk49vf8vpen82mhtwjwknh92yhxd6v2xntd8wvh6c5,csk-ht653ydxeme858rrnkw4pewyr4nfkwkxmp93664j4x2tm8c3,csk-fmk2hffxc3whf9698k6ppwrxhktwy9r6npd546j6rj83xhrp,csk-38knh3nefj99yjh2wc49pt6wcdcykpjwry3m6mvwrvyy383n,csk-vhph4fh2j23ycjv3efmr6vx6dcwf9vhx223yp96yrxcetcrf,csk-j2tndm225dw5c9fnk2r32f9kjce3jxjtwtrct4xmmw8pphc2,csk-mnhxcrwttr99mkrn43rhdwj26xy3fxyn8wenmcnxvk96jp45,csk-h4kj9kkw2xchjkjeh4mn9jnvn4p5t4xjev4tp98rmcyhex92,csk-p89c5hc3dj9jv8yyypytfv3ntjd25t6fenhw2cx98c4ec54t,csk-vkjhhfd33cd6kx592p48nmv9pp8c8rv5ryew4fdt3f33dk4k' \
'CEREBRAS_MODEL_DEFAULT=llama-3.3-70b' \
'OPENROUTER_API_KEYS=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,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,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' \
'' \
'SERVER_HOST=0.0.0.0' \
'SERVER_PORT=8080' \
'CORS_ALLOWED_ORIGINS=https://bt-shop-dark.online,https://www.bt-shop-dark.online' \
'' \
'JWT_SECRET=Fb3xK9mP2nQ7vL4wR8sT6yU1aB5cD0eH3jK8lM2nP4qR7sT9vW2xY5zA8bC1dE' \
'JWT_EXPIRES_IN=24h' \
'' \
'DATABASE_URL=postgres://postgres:postgres@postgres:5432/mr_darkpromth' \
'REDIS_URL=redis://redis:6379' \
'' \
'RUST_LOG=info' \
'' \
'GITHUB_CLIENT_ID=Ov23liZnNS4nh4RZg6gE' \
'GITHUB_CLIENT_SECRET=13b74d235a20803bd7c43bc585916185f8fa08f4' \
'GITHUB_REDIRECT_URI=https://bt-shop-dark.online/login/github' \
'' \
'SMTP_HOST=smtp.gmail.com' \
'SMTP_PORT=587' \
'SMTP_USER=mrdarkpromth2@gmail.com' \
'SMTP_PASSWORD=ldqeszoljdkjdric' \
'SMTP_FROM=noreply@bt-shop-dark.online' \
'SMTP_TLS=true' \
'' \
'SSL_CERT_PATH=/etc/nginx/certs/cert.pem' \
'SSL_KEY_PATH=/etc/nginx/certs/key.pem' \
'' \
'ENVIRONMENT=production' \
'DEBUG=false' \
'LOG_LEVEL=info' > .env

echo ".env created successfully!"
cat .env | head -20
