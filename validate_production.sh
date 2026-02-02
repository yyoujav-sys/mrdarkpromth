#!/bin/bash
# 🔍 MR.DarkPromth Production Readiness Validation Script
# ตรวจสอบความพร้อมสำหรับ Production แบบครอบคลุม

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counters
TESTS_PASSED=0
TESTS_FAILED=0
WARNINGS=0

# Print functions
print_header() {
    echo -e "\n${BLUE}========================================${NC}"
    echo -e "${BLUE} $1${NC}"
    echo -e "${BLUE}========================================${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
    ((TESTS_PASSED++))
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
    ((TESTS_FAILED++))
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
    ((WARNINGS++))
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

# ============================================
# 1. DOCKERFILE VALIDATION
# ============================================
print_header "1. DOCKERFILE VALIDATION"

# Check frontend Dockerfile
if [ -f "frontend/Dockerfile" ]; then
    if grep -q "COPY nginx/frontend.conf" frontend/Dockerfile; then
        print_success "Frontend Dockerfile includes nginx config copy"
    else
        print_error "Frontend Dockerfile missing nginx config copy"
    fi
    
    if grep -q "FROM nginx:stable-alpine" frontend/Dockerfile; then
        print_success "Frontend Dockerfile uses correct nginx base image"
    else
        print_warning "Frontend Dockerfile should use nginx:stable-alpine"
    fi
else
    print_error "Frontend Dockerfile not found"
fi

# Check root Dockerfile
if [ -f "Dockerfile" ]; then
    if grep -q "cargo build --release" Dockerfile; then
        print_success "Root Dockerfile has build step"
    else
        print_error "Root Dockerfile missing build step"
    fi
    
    if grep -q "HEALTHCHECK" Dockerfile; then
        print_success "Root Dockerfile includes healthcheck"
    else
        print_warning "Root Dockerfile missing healthcheck"
    fi
else
    print_error "Root Dockerfile not found"
fi

# ============================================
# 2. NGINX CONFIGURATION VALIDATION
# ============================================
print_header "2. NGINX CONFIGURATION VALIDATION"

if [ -f "nginx/nginx.conf" ]; then
    # Check upstream names
    if grep -q "upstream frontend" nginx/nginx.conf; then
        print_success "Nginx has frontend upstream defined"
    else
        print_error "Nginx missing frontend upstream"
    fi
    
    if grep -q "server mr_darkpromth_frontend:80" nginx/nginx.conf; then
        print_success "Nginx upstream uses correct container name (mr_darkpromth_frontend)"
    else
        print_error "Nginx upstream container name mismatch"
    fi
    
    if grep -q "upstream backend" nginx/nginx.conf; then
        print_success "Nginx has backend upstream defined"
    else
        print_error "Nginx missing backend upstream"
    fi
    
    # Check SPA routing
    if grep -q "try_files.*index.html" nginx/frontend.conf 2>/dev/null; then
        print_success "Frontend nginx config has SPA routing"
    else
        print_error "Frontend nginx config missing SPA routing"
    fi
    
    # Check security headers
    if grep -q "X-Frame-Options" nginx/nginx.conf; then
        print_success "Nginx has security headers"
    else
        print_warning "Nginx missing security headers"
    fi
else
    print_error "nginx.conf not found"
fi

# ============================================
# 3. CORS CONFIGURATION VALIDATION
# ============================================
print_header "3. CORS CONFIGURATION VALIDATION"

if [ -f "mr_darkpromth/api/src/axum_router.rs" ]; then
    if grep -q "bt-shop-dark.online" mr_darkpromth/api/src/axum_router.rs; then
        print_success "CORS includes production domain (bt-shop-dark.online)"
    else
        print_error "CORS missing production domain"
    fi
    
    if grep -q "localhost" mr_darkpromth/api/src/axum_router.rs; then
        print_success "CORS includes localhost for development"
    else
        print_warning "CORS missing localhost"
    fi
else
    print_error "axum_router.rs not found"
fi

# ============================================
# 4. CI/CD PIPELINE VALIDATION
# ============================================
print_header "4. CI/CD PIPELINE VALIDATION"

if [ -f ".github/workflows/ci.yml" ]; then
    if grep -q "Build and push Backend image" .github/workflows/ci.yml; then
        print_success "CI workflow builds backend image"
    else
        print_error "CI workflow missing backend build"
    fi
    
    if grep -q "Build and push Frontend image" .github/workflows/ci.yml; then
        print_success "CI workflow builds frontend image"
    else
        print_error "CI workflow missing frontend build"
    fi
    
    if grep -q "docker/build-push-action@v5" .github/workflows/ci.yml; then
        print_success "CI uses correct Docker build action"
    else
        print_warning "CI should use docker/build-push-action@v5"
    fi
else
    print_error "ci.yml not found"
fi

if [ -f ".github/workflows/deploy.yml" ]; then
    if grep -q "mr-darkpromth-backend" .github/workflows/deploy.yml; then
        print_success "Deploy workflow uses correct backend image name"
    else
        print_error "Deploy workflow backend image name mismatch"
    fi
    
    if grep -q "mr-darkpromth-frontend" .github/workflows/deploy.yml; then
        print_success "Deploy workflow uses correct frontend image name"
    else
        print_error "Deploy workflow frontend image name mismatch"
    fi
else
    print_error "deploy.yml not found"
fi

# ============================================
# 5. DOCKER-COMPOSE VALIDATION
# ============================================
print_header "5. DOCKER-COMPOSE VALIDATION"

if [ -f "docker-compose.yml" ]; then
    if grep -q "container_name: mr_darkpromth_frontend" docker-compose.yml; then
        print_success "docker-compose uses correct frontend container name"
    else
        print_error "docker-compose frontend container name mismatch"
    fi
    
    if grep -q "container_name: mr_darkpromth_api" docker-compose.yml; then
        print_success "docker-compose uses correct API container name"
    else
        print_error "docker-compose API container name mismatch"
    fi
    
    if grep -q "healthcheck" docker-compose.yml; then
        print_success "docker-compose includes healthchecks"
    else
        print_warning "docker-compose missing healthchecks"
    fi
else
    print_error "docker-compose.yml not found"
fi

# ============================================
# 6. FRONTEND CODE VALIDATION
# ============================================
print_header "6. FRONTEND CODE VALIDATION"

if [ -f "frontend/src/App.tsx" ]; then
    if grep -q "BrowserRouter" frontend/src/App.tsx; then
        print_success "Frontend uses BrowserRouter"
    else
        print_error "Frontend missing BrowserRouter"
    fi
    
    if grep -q "Route.*login" frontend/src/App.tsx; then
        print_success "Frontend has login route"
    else
        print_error "Frontend missing login route"
    fi
    
    if grep -q "Route.*register" frontend/src/App.tsx; then
        print_success "Frontend has register route"
    else
        print_error "Frontend missing register route"
    fi
else
    print_error "App.tsx not found"
fi

# Check package.json
if [ -f "frontend/package.json" ]; then
    if grep -q '"build"' frontend/package.json; then
        print_success "Frontend has build script"
    else
        print_error "Frontend missing build script"
    fi
else
    print_error "frontend/package.json not found"
fi

# ============================================
# 7. BACKEND CODE VALIDATION
# ============================================
print_header "7. BACKEND CODE VALIDATION"

if [ -f "mr_darkpromth/api/src/main.rs" ]; then
    if grep -q "8080" mr_darkpromth/api/src/main.rs; then
        print_success "Backend listens on port 8080"
    else
        print_warning "Backend port not explicitly set to 8080"
    fi
else
    print_error "main.rs not found"
fi

# Check handlers
if [ -d "mr_darkpromth/api/src/handlers" ]; then
    HANDLER_COUNT=$(find mr_darkpromth/api/src/handlers -name "*.rs" | wc -l)
    if [ "$HANDLER_COUNT" -gt 0 ]; then
        print_success "Backend has $HANDLER_COUNT handler modules"
    else
        print_error "Backend missing handler modules"
    fi
else
    print_error "handlers directory not found"
fi

# ============================================
# 8. VS CODE EXTENSION VALIDATION
# ============================================
print_header "8. VS CODE EXTENSION VALIDATION"

if [ -f "vscode-extension/package.json" ]; then
    if grep -q '"name": "mr-darkpromth"' vscode-extension/package.json; then
        print_success "VS Code extension has correct name"
    else
        print_error "VS Code extension name mismatch"
    fi
    
    if grep -q '"vscode":' vscode-extension/package.json; then
        print_success "VS Code extension has engine requirement"
    else
        print_warning "VS Code extension missing engine requirement"
    fi
else
    print_warning "vscode-extension/package.json not found"
fi

# ============================================
# 9. DATABASE MIGRATIONS VALIDATION
# ============================================
print_header "9. DATABASE MIGRATIONS VALIDATION"

MIGRATION_COUNT=$(find migrations -name "*.sql" 2>/dev/null | wc -l)
if [ "$MIGRATION_COUNT" -gt 0 ]; then
    print_success "Found $MIGRATION_COUNT database migration files"
else
    print_warning "No database migration files found"
fi

# ============================================
# 10. ENVIRONMENT CONFIGURATION
# ============================================
print_header "10. ENVIRONMENT CONFIGURATION"

if [ -f ".env.example" ]; then
    print_success ".env.example file exists"
    
    if grep -q "DATABASE_URL" .env.example; then
        print_success ".env.example includes DATABASE_URL"
    else
        print_warning ".env.example missing DATABASE_URL"
    fi
else
    print_warning ".env.example not found"
fi

# ============================================
# SUMMARY
# ============================================
print_header "VALIDATION SUMMARY"

echo -e "Tests Passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "Tests Failed: ${RED}$TESTS_FAILED${NC}"
echo -e "Warnings: ${YELLOW}$WARNINGS${NC}"

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "\n${GREEN}🎉 All critical tests passed! System is ready for production.${NC}"
    exit 0
else
    echo -e "\n${RED}⚠️  Some tests failed. Please fix the issues before deploying to production.${NC}"
    exit 1
fi
