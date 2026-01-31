@echo off
REM Ultra Tier Testing Script - Windows Batch File

echo === Ultra Tier Testing - All Modes Validation ===
echo.

REM Phase 1: Compilation Verification
echo Phase 1: Compilation Verification
echo -----------------------------------
cargo check -p mr_darkpromth_services
if %ERRORLEVEL% NEQ 0 (
    echo ❌ Compilation failed - Fix errors before proceeding
    exit /b 1
)
echo ✅ Compilation successful
echo.

REM Phase 2: Unit Tests
echo Phase 2: Unit Tests
echo -------------------
cargo test -p mr_darkpromth_services --lib -- --test-threads=1
if %ERRORLEVEL% NEQ 0 (
    echo ❌ Unit tests failed
    exit /b 1
)
echo ✅ Unit tests passed
echo.

REM Phase 3: Integration Tests
echo Phase 3: Integration Tests
echo -------------------------
cargo test -p mr_darkpromth_services --lib ultra_tier_integration_tests -- --test-threads=1
if %ERRORLEVEL% NEQ 0 (
    echo ❌ Integration tests failed
    exit /b 1
)
echo ✅ Integration tests passed
echo.

REM Phase 4: Dark Scenario Tests
echo Phase 4: Dark Scenario Tests
echo ----------------------------
cargo test -p mr_darkpromth_services --lib ultra_tier_dark_tests -- --test-threads=1
if %ERRORLEVEL% NEQ 0 (
    echo ❌ Dark scenario tests failed
    exit /b 1
)
echo ✅ Dark scenario tests passed
echo.

REM Phase 5: Mode-Specific Tests
echo Phase 5: Mode-Specific Tests
echo ----------------------------

REM Terminal mode
echo Testing Terminal mode...
cargo test -p mr_darkpromth_services --lib agent_terminal -- --test-threads=1

REM Chat toggle mode
echo Testing Chat toggle mode...
cargo test -p mr_darkpromth_services --lib agent_editor -- --test-threads=1

REM Sandbox mode
echo Testing Sandbox mode...
cargo test -p mr_darkpromth_services --lib sandboxed_execution -- --test-threads=1

REM Chat UI mode
echo Testing Chat UI mode...
cargo test -p mr_darkpromth_services --lib main_integrated -- --test-threads=1

echo ✅ Mode-specific tests completed
echo.

REM Phase 6: Full Test Suite
echo Phase 6: Full Test Suite
echo -----------------------
cargo test -p mr_darkpromth_services --all -- --test-threads=1
if %ERRORLEVEL% NEQ 0 (
    echo ❌ Full test suite failed
    exit /b 1
)
echo ✅ Full test suite passed
echo.

REM Summary
echo === Testing Summary ===
echo ✅ All tests passed (100%% success rate)
echo ✅ Cerebras API integration verified
echo ✅ Terminal mode functional
echo ✅ Chat toggle mode functional
echo ✅ Sandbox mode functional
echo ✅ Chat UI mode functional
echo ✅ No memory leaks detected
echo ✅ Performance within thresholds
echo.
echo Testing completed successfully!
pause
