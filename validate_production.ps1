# 🔍 MR.DarkPromth Production Readiness Validation (PowerShell)
# ตรวจสอบความพร้อมสำหรับ Production

$ErrorActionPreference = "Continue"

$script:TESTS_PASSED = 0
$script:TESTS_FAILED = 0
$script:WARNINGS = 0

function Print-Header($text) {
    Write-Host "`n========================================" -ForegroundColor Blue
    Write-Host " $text" -ForegroundColor Blue
    Write-Host "========================================" -ForegroundColor Blue
}

function Print-Success($text) {
    Write-Host "✅ $text" -ForegroundColor Green
    $script:TESTS_PASSED++
}

function Print-Error($text) {
    Write-Host "❌ $text" -ForegroundColor Red
    $script:TESTS_FAILED++
}

function Print-Warning($text) {
    Write-Host "⚠️  $text" -ForegroundColor Yellow
    $script:WARNINGS++
}

# 1. DOCKERFILE VALIDATION
Print-Header "1. DOCKERFILE VALIDATION"

if (Test-Path "frontend/Dockerfile") {
    $content = Get-Content "frontend/Dockerfile" -Raw
    if ($content -match "COPY nginx/frontend.conf") {
        Print-Success "Frontend Dockerfile includes nginx config copy"
    } else {
        Print-Error "Frontend Dockerfile missing nginx config copy"
    }
} else {
    Print-Error "Frontend Dockerfile not found"
}

if (Test-Path "Dockerfile") {
    $content = Get-Content "Dockerfile" -Raw
    if ($content -match "cargo build --release") {
        Print-Success "Root Dockerfile has build step"
    } else {
        Print-Error "Root Dockerfile missing build step"
    }
} else {
    Print-Error "Root Dockerfile not found"
}

# 2. NGINX CONFIGURATION
Print-Header "2. NGINX CONFIGURATION VALIDATION"

if (Test-Path "nginx/nginx.conf") {
    $content = Get-Content "nginx/nginx.conf" -Raw
    if ($content -match "upstream frontend") {
        Print-Success "Nginx has frontend upstream defined"
    } else {
        Print-Error "Nginx missing frontend upstream"
    }
    
    if ($content -match "server mr_darkpromth_frontend:80") {
        Print-Success "Nginx upstream uses correct container name"
    } else {
        Print-Error "Nginx upstream container name mismatch"
    }
} else {
    Print-Error "nginx.conf not found"
}

if (Test-Path "nginx/frontend.conf") {
    $content = Get-Content "nginx/frontend.conf" -Raw
    if ($content -match "try_files.*index.html") {
        Print-Success "Frontend nginx config has SPA routing"
    } else {
        Print-Error "Frontend nginx config missing SPA routing"
    }
} else {
    Print-Error "frontend.conf not found"
}

# 3. CORS CONFIGURATION
Print-Header "3. CORS CONFIGURATION VALIDATION"

if (Test-Path "mr_darkpromth/api/src/axum_router.rs") {
    $content = Get-Content "mr_darkpromth/api/src/axum_router.rs" -Raw
    if ($content -match "bt-shop-dark.online") {
        Print-Success "CORS includes production domain"
    } else {
        Print-Error "CORS missing production domain"
    }
} else {
    Print-Error "axum_router.rs not found"
}

# 4. CI/CD PIPELINE
Print-Header "4. CI/CD PIPELINE VALIDATION"

if (Test-Path ".github/workflows/ci.yml") {
    $content = Get-Content ".github/workflows/ci.yml" -Raw
    if ($content -match "Build and push Backend image") {
        Print-Success "CI workflow builds backend image"
    } else {
        Print-Error "CI workflow missing backend build"
    }
    
    if ($content -match "Build and push Frontend image") {
        Print-Success "CI workflow builds frontend image"
    } else {
        Print-Error "CI workflow missing frontend build"
    }
} else {
    Print-Error "ci.yml not found"
}

if (Test-Path ".github/workflows/deploy.yml") {
    $content = Get-Content ".github/workflows/deploy.yml" -Raw
    if ($content -match "mr-darkpromth-backend") {
        Print-Success "Deploy workflow uses correct backend image name"
    } else {
        Print-Error "Deploy workflow backend image name mismatch"
    }
} else {
    Print-Error "deploy.yml not found"
}

# 5. DOCKER-COMPOSE
Print-Header "5. DOCKER-COMPOSE VALIDATION"

if (Test-Path "docker-compose.yml") {
    $content = Get-Content "docker-compose.yml" -Raw
    if ($content -match "container_name: mr_darkpromth_frontend") {
        Print-Success "docker-compose uses correct frontend container name"
    } else {
        Print-Error "docker-compose frontend container name mismatch"
    }
} else {
    Print-Error "docker-compose.yml not found"
}

# 6. FRONTEND CODE
Print-Header "6. FRONTEND CODE VALIDATION"

if (Test-Path "frontend/src/App.tsx") {
    $content = Get-Content "frontend/src/App.tsx" -Raw
    if ($content -match "BrowserRouter") {
        Print-Success "Frontend uses BrowserRouter"
    } else {
        Print-Error "Frontend missing BrowserRouter"
    }
    
    if ($content -match 'Route.*login') {
        Print-Success "Frontend has login route"
    } else {
        Print-Error "Frontend missing login route"
    }
} else {
    Print-Error "App.tsx not found"
}

# SUMMARY
Print-Header "VALIDATION SUMMARY"

Write-Host "Tests Passed: $TESTS_PASSED" -ForegroundColor Green
Write-Host "Tests Failed: $TESTS_FAILED" -ForegroundColor Red
Write-Host "Warnings: $WARNINGS" -ForegroundColor Yellow

if ($TESTS_FAILED -eq 0) {
    Write-Host "`n🎉 All critical tests passed! System is ready for production." -ForegroundColor Green
    exit 0
} else {
    Write-Host "`n⚠️  Some tests failed. Please fix before deploying." -ForegroundColor Red
    exit 1
}
