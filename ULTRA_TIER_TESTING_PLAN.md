# Ultra Tier Testing Plan - All Modes Validation

## Testing Objectives
- Verify Ultra Tier functionality works in Terminal mode
- Verify Ultra Tier functionality works in Chat toggle mode  
- Verify Ultra Tier functionality works in Sandbox mode
- Verify Ultra Tier functionality works in Chat UI mode
- Ensure 100% test pass rate
- Fix any errors or failures encountered

## Test Categories

### 1. Terminal Mode Tests
- Ultra Tier jailbreak application via terminal commands
- Safety filter with dangerous commands
- Tool execution (file operations, web scraping)
- Audit logging completeness
- Performance metrics

### 2. Chat Toggle Mode Tests
- Chat interface with Ultra tier prompts
- Jailbreak toggle functionality
- Response generation with Cerebras API
- Safety filtering in chat context
- User tier detection in chat

### 3. Sandbox Mode Tests
- Sandboxed code execution
- Resource limits enforcement
- Docker-based isolation
- Process monitoring
- Security violation tracking

### 4. Chat UI Mode Tests
- Full UI integration
- Real-time responses
- User interactions
- Error handling
- Performance under load

## Execution Plan

### Phase 1: Compilation Verification
```bash
cargo check -p mr_darkpromth_services
cargo check -p mr_darkpromth_core
```

### Phase 2: Unit Tests
```bash
cargo test -p mr_darkpromth_services --lib
```

### Phase 3: Integration Tests
```bash
cargo test -p mr_darkpromth_services --lib ultra_tier_integration_tests
cargo test -p mr_darkpromth_services --lib ultra_tier_dark_tests
```

### Phase 4: Mode-Specific Tests
```bash
# Terminal mode
cargo test -p mr_darkpromth_services --lib terminal

# Chat toggle mode  
cargo test -p mr_darkpromth_services --lib agent_editor

# Sandbox mode
cargo test -p mr_darkpromth_services --lib sandboxed_execution

# Chat UI mode
cargo test -p mr_darkpromth_services --lib main_integrated
```

### Phase 5: End-to-End Tests
```bash
cargo test -p mr_darkpromth_services --test '*'
```

## Expected Results

### Success Criteria
- ✅ All tests compile without errors
- ✅ 100% test pass rate (37 tests)
- ✅ Cerebras API integration works
- ✅ All modes function correctly
- ✅ No memory leaks
- ✅ Performance within thresholds

### Failure Handling
- If compilation fails: Fix errors immediately
- If tests fail: Identify root cause, fix, re-run
- If Cerebras API fails: Check environment variables, API key
- If mode-specific tests fail: Fix mode-specific issues

## Testing Questions

### Terminal Mode
1. Can Ultra tier users execute jailbreak prompts via terminal?
2. Does safety filter block dangerous commands?
3. Are tools (file ops, web scraping) working?
4. Is audit logging complete?

### Chat Toggle Mode
1. Does chat toggle work for Ultra tier?
2. Are jailbreak prompts applied correctly?
3. Are responses generated via Cerebras API?
4. Is safety filtering applied?

### Sandbox Mode
1. Is code execution properly sandboxed?
2. Are resource limits enforced?
3. Is Docker isolation working?
4. Are suspicious processes monitored?

### Chat UI Mode
1. Does full UI integration work?
2. Are real-time responses functional?
3. Are user interactions smooth?
4. Is error handling robust?

## Validation Checklist

- [ ] All tests compile
- [ ] All tests pass (37/37)
- [ ] Cerebras API integration verified
- [ ] Terminal mode functional
- [ ] Chat toggle mode functional
- [ ] Sandbox mode functional
- [ ] Chat UI mode functional
- [ ] No memory leaks detected
- [ ] Performance within thresholds
- [ ] Security violations tracked correctly

## Next Steps

1. Run compilation check
2. Execute all tests
3. Fix any errors found
4. Verify each mode works
5. Generate final validation report
