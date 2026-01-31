@echo off
echo Starting MR.DarkPromth Backend...

REM Check if Rust is installed
where cargo >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo ERROR: Rust/Cargo is not installed!
    echo Please install Rust from https://rustup.rs/
    pause
    exit /b 1
)

REM Check if PostgreSQL is running on port 5432
netstat -ano | findstr ":5432" >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo WARNING: PostgreSQL is not running on port 5432
    echo Please start PostgreSQL or run with Docker
)

REM Check if Redis is running on port 6379
netstat -ano | findstr ":6379" >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo WARNING: Redis is not running on port 6379
    echo Please start Redis or run with Docker
)

REM Use local environment file
if exist .env.local (
    copy .env.local .env >nul
    echo Using local development configuration
)

echo Building and starting backend...
cargo run

pause
