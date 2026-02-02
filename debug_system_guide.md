# 🔍 MR.DarkPromth Debug Guide - ระบบ Debug แบบละเอียด

## 📋 ตารางสรุปการ Debug

| ส่วนของระบบ | Command | สิ่งที่ตรวจสอบ | สถานะปกติ |
|-------------|---------|----------------|------------|
| **Infrastructure** | `docker ps` | Containers ที่รันอยู่ | 7+ containers |
| **Network** | `docker network ls` | Network connections | mr_darkpromth_network |
| **Database** | `docker exec -it mr_darkpromth_postgres psql` | Connection และ tables | Connected |
| **Redis** | `docker exec -it mr_darkpromth_redis redis-cli` | Cache status | PONG |
| **API** | `curl http://localhost:8080/health` | API response | JSON status |
| **Frontend** | `curl https://bt-shop-dark.online` | HTML response | 200 OK |
| **SSL** | `certbot certificates` | Certificate validity | Valid |
| **Nginx** | `nginx -t` | Config syntax | OK |

---

## 🚀 1. Infrastructure Debug

### ตรวจสอบ Containers ทั้งหมด
```bash
# ดู containers ทั้งหมด
docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"

# ตรวจสอบ containers ที่หยุดทำงาน
docker ps -a --format "table {{.Names}}\t{{.Status}}\t{{.Exited}}"

# ดู resource usage
docker stats --no-stream
```

### ตรวจสอบ Network
```bash
# ดู network ทั้งหมด
docker network ls

# ตรวจสอบว่า containers อยู่ใน network เดียวกัน
docker network inspect mr_darkpromth_network

# ทดสอบการเชื่อมต่อระหว่าง containers
docker exec mr_darkpromth_api ping mr_darkpromth_postgres
docker exec mr_darkpromth_api ping mr_darkpromth_redis
```

---

## 🗄️ 2. Database Debug

### ตรวจสอบ PostgreSQL
```bash
# เข้าสู่ database
docker exec -it mr_darkpromth_postgres psql -U postgres -d mr_darkpromth

# ตรวจสอบ tables
\dt

# ตรวจสอดู connections
SELECT * FROM pg_stat_activity;

# ตรวจสอบ database size
SELECT pg_size_pretty(pg_database_size('mr_darkpromth'));

# ทดสอบ query
SELECT COUNT(*) FROM users;
```

### ตรวจสอบ Redis
```bash
# เข้าสู่ redis cli
docker exec -it mr_darkpromth_redis redis-cli

# ทดสอบ connection
PING

# ดู keys ทั้งหมด
KEYS *

# ตรวจสอดู memory
INFO memory

# ดู connections
INFO clients
```

---

## 🔌 3. API Debug

### ตรวจสอบ API Endpoints
```bash
# Health check
curl -v http://localhost:8080/health

# API status
curl -v http://localhost:8080/api/status

# API info
curl -v http://localhost:8080/api/info

# Metrics
curl -v http://localhost:8080/metrics

# Test with POST
curl -X POST http://localhost:8080/api/test \
  -H "Content-Type: application/json" \
  -d '{"test": "data"}'
```

### ตรวจสอบ API Logs
```bash
# ดู logs ล่าสุด
docker logs mr_darkpromth_api --tail 50

# ดู logs แบบ real-time
docker logs -f mr_darkpromth_api

# ดู logs พร้อม timestamp
docker logs --timestamps mr_darkpromth_api

# บันทึก logs ลงไฟล์
docker logs mr_darkpromth_api > api_logs.txt 2>&1
```

### ตรวจสอบ API Performance
```bash
# ทดสอบ response time
time curl -s http://localhost:8080/health

# ทดสอบ concurrent connections
ab -n 100 -c 10 http://localhost:8080/health

# ตรวจสอบ port usage
netstat -tlnp | grep 8080
```

---

## 🌐 4. Frontend Debug

### ตรวจสอบ Frontend
```bash
# ตรวจสอบ HTTP response
curl -I https://bt-shop-dark.online

# ตรวจสอบ HTML content
curl -s https://bt-shop-dark.online | head -20

# ตรวจสอบ static assets
curl -I https://bt-shop-dark.online/assets/index-RDiDuoCZ.js

# ตรวจสอดู landing page
curl -I https://bt-shop-dark.online/landing
```

### ตรวจสอบ Frontend Logs
```bash
# ดู nginx logs
docker logs mr_darkpromth_frontend_react --tail 50

# ตรวจสอบ nginx config
docker exec mr_darkpromth_frontend_react nginx -t

# ดู access logs
docker exec mr_darkpromth_frontend_react cat /var/log/nginx/access.log | tail -20
```

---

## 🔐 5. SSL & Security Debug

### ตรวจสอบ SSL Certificate
```bash
# ดู certificate ทั้งหมด
certbot certificates

# ตรวจสอบ certificate details
openssl x509 -in /etc/letsencrypt/live/bt-shop-dark.online/cert.pem -text -noout

# ทดสอบ SSL connection
openssl s_client -connect bt-shop-dark.online:443 -servername bt-shop-dark.online

# ตรวจสอบ expiration date
openssl x509 -in /etc/letsencrypt/live/bt-shop-dark.online/cert.pem -noout -dates
```

### ตรวจสอบ Nginx SSL Config
```bash
# ตรวจสอบ nginx config
nginx -t

# ดู SSL config
grep -n ssl_ /etc/nginx/nginx.conf

# ทดสอบ HTTPS redirect
curl -I http://bt-shop-dark.online
```

---

## 📊 6. Monitoring Debug

### ตรวจสอบ Grafana
```bash
# ทดสอบ Grafana API
curl -s http://localhost:3001/api/health

# ตรวจสอบ Grafana logs
docker logs mr_darkpromth_grafana --tail 20

# ดู Grafana config
docker exec mr_darkpromth_grafana cat /etc/grafana/grafana.ini | grep -E "(server|database)"
```

### ตรวจสอบ Prometheus
```bash
# ทดสอบ Prometheus targets
curl -s http://localhost:9090/api/v1/targets

# ตรวจสอดู metrics
curl -s http://localhost:9090/api/v1/query?query=up

# ดู Prometheus logs
docker logs prometheus --tail 20
```

---

## 🔧 7. Advanced Debug Commands

### System Resource Debug
```bash
# ดู CPU usage
top -p $(docker inspect -f '{{.State.Pid}}' mr_darkpromth_api)

# ดู memory usage
docker stats --no-stream | grep mr_darkpromth

# ดู disk usage
df -h
docker system df

# ดู network connections
ss -tulpn | grep :8080
```

### Container Deep Dive
```bash
# ตรวจสอบ container details
docker inspect mr_darkpromth_api | jq '.[0].NetworkSettings.Networks'

# เข้าสู่ container shell
docker exec -it mr_darkpromth_api /bin/bash

# ดู environment variables
docker exec mr_darkpromth_api env | grep -E "(DATABASE|REDIS)"

# ตรวจสอบ processes ใน container
docker exec mr_darkpromth_api ps aux
```

---

## 🚨 8. Common Issues & Solutions

### Issue 1: API Not Responding
```bash
# ตรวจสอบ
docker ps | grep api
curl -v http://localhost:8080/health
docker logs mr_darkpromth_api

# แก้ไข
docker restart mr_darkpromth_api
```

### Issue 2: Database Connection Failed
```bash
# ตรวจสอบ
docker exec mr_darkpromth_api ping mr_darkpromth_postgres
docker logs mr_darkpromth_postgres

# แก้ไข
docker restart mr_darkpromth_postgres
```

### Issue 3: SSL Certificate Expired
```bash
# ตรวจสอบ
certbot certificates
openssl x509 -noout -dates -in /etc/letsencrypt/live/bt-shop-dark.online/cert.pem

# แก้ไข
certbot renew --force-renewal
systemctl reload nginx
```

### Issue 4: Frontend Not Loading
```bash
# ตรวจสอบ
curl -I https://bt-shop-dark.online
docker logs mr_darkpromth_frontend_react

# แก้ไข
docker restart mr_darkpromth_frontend_react
```

---

## 📝 9. Debug Script อัตโนมัติ

สร้าง script `/opt/mrdarkpromth/debug_system.sh`:
```bash
#!/bin/bash
echo "=== 🔍 MR.DarkPromth System Debug ==="
echo ""

echo "📦 Containers Status:"
docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
echo ""

echo "🌐 Network Test:"
docker exec mr_darkpromth_api ping -c 1 mr_darkpromth_postgres 2>/dev/null && echo "✅ API → PostgreSQL OK" || echo "❌ API → PostgreSQL FAIL"
docker exec mr_darkpromth_api ping -c 1 mr_darkpromth_redis 2>/dev/null && echo "✅ API → Redis OK" || echo "❌ API → Redis FAIL"
echo ""

echo "🔌 API Test:"
curl -s http://localhost:8080/health | jq -r '.status' 2>/dev/null && echo "✅ API Health OK" || echo "❌ API Health FAIL"
echo ""

echo "🌐 Frontend Test:"
curl -s -o /dev/null -w "%{http_code}" https://bt-shop-dark.online | grep "200" && echo "✅ Frontend OK" || echo "❌ Frontend FAIL"
echo ""

echo "🔐 SSL Status:"
certbot certificates 2>/dev/null | grep "bt-shop-dark.online" && echo "✅ SSL OK" || echo "❌ SSL FAIL"
echo ""

echo "📊 Resource Usage:"
docker stats --no-stream --format "table {{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}" | grep mr_darkpromth
echo ""

echo "=== 🔍 Debug Complete ==="
```

---

## 🎯 10. Quick Debug Checklist

- [ ] Containers ทั้งหมดรันอยู่?
- [ ] Network connection เชื่อมต่อได้?
- [ ] Database ตอบสนอง?
- [ ] Redis ทำงาน?
- [ ] API endpoints ตอบสนอง?
- [ ] Frontend โหลดได้?
- [ ] SSL certificate ยังไม่หมด?
- [ ] Nginx config ถูกต้อง?
- [ ] Logs ไม่มี error?
- [ ] Resource usage ปกติ?

---

## 💡 Pro Tips

1. **ใช้ `docker-compose logs`** สำหรับดู logs หลาย container
2. **ตรวจสอบ timestamps** ใน logs เพื่อดู timeline ของปัญหา
3. **ใช้ `jq`** สำหรับ format JSON output
4. **สร้าง aliases** สำหรับ commands ที่ใช้บ่อย
5. **ตั้งค่า log rotation** เพื่อป้องกัน disk เต็ม

**🔍 ใช้คู่มือนี้เพื่อ debug ระบบอย่างมีประสิทธิภาพ!**
