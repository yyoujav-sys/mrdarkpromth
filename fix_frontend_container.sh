#!/bin/bash
# 🛠️ MR.DarkPromth Frontend Container Fix Script

echo "=== 🛠️ MR.DarkPromth Frontend Container Fix ==="
echo "Time: $(date)"
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}🔧 Fixing Frontend Container SPA Routing${NC}"

# Check if frontend container exists
if ! docker ps | grep -q "mr_darkpromth_frontend_react"; then
    echo -e "${RED}❌ Frontend container not found${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Frontend container found${NC}"

# Copy SPA nginx config
echo -e "${BLUE}📝 Copying SPA nginx config...${NC}"
docker cp /tmp/frontend.conf mr_darkpromth_frontend_react:/etc/nginx/conf.d/default.conf

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Config copied successfully${NC}"
else
    echo -e "${RED}❌ Failed to copy config${NC}"
    exit 1
fi

# Restart container
echo -e "${BLUE}🔄 Restarting frontend container...${NC}"
docker restart mr_darkpromth_frontend_react

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Container restarted successfully${NC}"
else
    echo -e "${RED}❌ Failed to restart container${NC}"
    exit 1
fi

# Wait for container to be ready
echo -e "${BLUE}⏳ Waiting for container to be ready...${NC}"
sleep 5

# Test SPA routing
echo -e "${BLUE}🧪 Testing SPA routing...${NC}"

# Test main page
if curl -s -f http://localhost:3000/ > /dev/null; then
    echo -e "${GREEN}✅ Main page: OK${NC}"
else
    echo -e "${RED}❌ Main page: FAILED${NC}"
fi

# Test login page
if curl -s -f http://localhost:3000/login > /dev/null; then
    echo -e "${GREEN}✅ Login page: OK${NC}"
else
    echo -e "${RED}❌ Login page: FAILED${NC}"
fi

# Test register page
if curl -s -f http://localhost:3000/register > /dev/null; then
    echo -e "${GREEN}✅ Register page: OK${NC}"
else
    echo -e "${RED}❌ Register page: FAILED${NC}"
fi

# Test assets
if curl -s -f http://localhost:3000/assets/index-I8JhpMqi.css > /dev/null; then
    echo -e "${GREEN}✅ CSS assets: OK${NC}"
else
    echo -e "${RED}❌ CSS assets: FAILED${NC}"
fi

if curl -s -f http://localhost:3000/assets/index-CXNvw4uB.js > /dev/null; then
    echo -e "${GREEN}✅ JS assets: OK${NC}"
else
    echo -e "${RED}❌ JS assets: FAILED${NC}"
fi

echo ""
echo -e "${GREEN}=== ✅ Frontend Container Fix Complete ===${NC}"
echo -e "${BLUE}📋 All SPA routes should now work properly${NC}"
echo -e "${BLUE}🌐 Test with: https://bt-shop-dark.online/login${NC}"
