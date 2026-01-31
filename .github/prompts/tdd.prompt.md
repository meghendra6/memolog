---
name: tdd
description: "Start test-driven development for a feature. NOT for PoC, analysis, or exploration."
argument-hint: "Describe the feature to implement with TDD"
agent: TDDGuide
tools: ["editFiles", "runTests", "codebase", "textSearch"]
---

# TDD Prompt

<Scope_Warning>
**WHEN TO USE**: Feature development with defined requirements.
**WHEN NOT TO USE**: PoC, prototyping, analysis, exploration, spike work, or research tasks.
For exploratory work, iterate fast without tests first.
</Scope_Warning>

Guide the implementation using Test-Driven Development.

## Feature to Implement

${input:feature:Describe the feature to implement.}

## TDD Workflow

### Phase 1: RED (Write Failing Test)

1. Understand the requirement
2. Write a test that describes the expected behavior
3. Run the test - it MUST fail
4. Verify the failure is for the right reason

### Phase 2: GREEN (Make It Pass)

1. Write the minimal code to make the test pass
2. Run the test - it MUST pass
3. Don't optimize yet, just make it work

### Phase 3: REFACTOR

1. Improve code quality
2. Remove duplication
3. Ensure tests still pass

## Edge Cases to Consider

- Null/undefined inputs
- Empty collections
- Boundary values
- Error conditions
- Concurrent access (if applicable)

## Test Structure

Use the project's existing test framework. Follow this pattern:

```
describe('FeatureName')
  describe('methodName')
    it('should [expected behavior] when [condition]')
      // Arrange - set up test data
      // Act - call the method
      // Assert - verify the result
```

## Output

After each phase, report:

- Tests written
- Tests passing
- Current coverage
- Next step
