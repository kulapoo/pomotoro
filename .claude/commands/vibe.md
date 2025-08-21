# Claude Code Documentation-Only Planning Workflow (VIBE MODE)

## Workflow Initialization

```
{spec} = $ARGUMENTS
```

## VIBE MODE OUTPUT

**The ONLY output of VIBE MODE is:**
- 📁 `.claude/specs/{spec-name}/` directory with:
  - 📝 Proposed code in `task-*/code.md` files
  - 📝 Test strategies in `task-*/test.md` files
  - 📝 Architecture decisions in `summary.md`
  - 📝 Task breakdown in `tasks.md`

**NEVER:**
- ❌ No files in `/src`, `/domain`, `/infrastructure` or any project directories
- ❌ No `.rs`, `.toml`, `.json` or any actual code files modified
- ❌ No compilation, no cargo commands, no actual implementation

## Core Principles

- **Approach**: Documentation-only planning with Human-in-the-Loop methodology
- **Code Generation**: ALL code written to `.claude/specs/` markdown files ONLY
- **Codebase Modification**: STRICTLY FORBIDDEN - NO actual files are ever modified
- **Control Flow**: Pause-and-proceed mechanism for each task
- **Output**: Proposed implementations in code.md files for user review

## CRITICAL: VIBE MODE TOOL RESTRICTIONS

**ALLOWED TOOLS in VIBE MODE:**
- ✅ Write - ONLY to `.claude/specs/` directory
- ✅ Read - For understanding existing code
- ✅ Grep/Glob/LS - For searching and exploration
- ✅ TodoWrite - For task tracking

**FORBIDDEN TOOLS in VIBE MODE:**
- ❌ Edit - NEVER modify existing files
- ❌ MultiEdit - NEVER modify existing files
- ❌ Write to any path outside `.claude/specs/`
- ❌ Bash commands that modify files
- ❌ Any tool that changes the actual codebase

## CRITICAL: Agent Control Flow Rules

**VIBE MODE CIRCUIT BREAKER:**
- **NEVER** automatically chain agents based on their output
- **IGNORE** any agent suggestions like "now X agent should..."
- **ONLY** user commands (`__proceed`, `__next`) advance workflow
- **ALWAYS** pause after each agent completes
- **ENFORCE** [PAUSED - Awaiting __proceed] state between agents

## Phase 1: Analysis & Planning

### 1.1 Deep Analysis ("ULTRA THINK")
- systems-architect subagent MUST BE USED for Deep Analysis
- **MVP MINDSET**: Focus on smallest viable solution
- **SCOPE REDUCTION**: If spec is complex, extract core functionality only
- **ACHIEVABLE GOALS**: Ensure all tasks can be completed in current session

### 1.2 systems-architect subagent Activation
**CRITICAL PAUSE REQUIREMENT**:
- **MUST** end output with "[PAUSED - Awaiting __proceed]"
- **FORBIDDEN**: Suggesting next agents or implementation steps
- **MANDATORY**: Complete PAUSE before any further action
- **NO AUTOMATIC PROGRESSION**: Wait for explicit user command

**Primary Responsibilities:**
1. **Specification Analysis**
   - Parse functional requirements
   - Identify MINIMAL MVP requirements
   - Focus on ACHIEVABLE scope within current iteration
   - Map only ESSENTIAL dependencies

2. **Architecture Design (MVP-FOCUSED)**
   - Define MINIMAL component boundaries for MVP
   - Establish SIMPLEST data flow patterns that work
   - Identify ONLY necessary design patterns
   - **CRITICAL**: Scope must be SMALL and ACHIEVABLE
   - **AVOID**: Over-engineering or future-proofing beyond immediate needs

3. **Task Decomposition (SMALL SCOPE)**
   - Break down into SMALL, atomic, testable tasks
   - Each task should be completable in <30 minutes
   - Focus on IMMEDIATE deliverables only
   - Define MINIMAL acceptance criteria per task
   - **MAXIMUM**: 5-7 tasks for any spec (prefer 3-5)
   - **CRITICAL**: If spec seems large, extract MVP subset


## Phase 2: Documentation Generation

### 2.1 Summary Generation
**Output Format:**
```markdown
# {spec-name}

## Summary
[Comprehensive solution overview]
[Architecture decisions]
[Implementation strategy]

## Tasks
1. [Task description with clear deliverable]
2. [Task description with clear deliverable]
3. [Task description with clear deliverable]
...
```

### 2.2 File Structure Creation

**CRITICAL: Documentation-First Approach**
- **ALL code MUST be written to `.claude/specs/{spec-name}/` FIRST**
- **NEVER modify actual codebase files until Phase 4**
- **Generate implementations in `code.md` files ONLY**

**Generated Structure after __proceed prompt:**

```
.claude/specs/{spec-name}/
├── summary.md          # Overall solution and approach
├── tasks.md            # Master task list with IDs
├── task-1/
│   ├── task.md        # Detailed requirements & acceptance criteria
│   ├── code.md        # Implementation (WRITE CODE HERE, NOT IN CODEBASE)
│   └── revision.md    # Iteration history (initially empty)
├── task-2/
│   ├── task.md
│   ├── code.md        # Implementation (WRITE CODE HERE, NOT IN CODEBASE)
│   └── revision.md
└── ...
```

## Phase 3: Human-Controlled Development Loop (NO AUTO-EXECUTION)

### 3.0 PAUSE ENFORCEMENT
**MANDATORY**: After ANY agent completes:
1. Output: "[PAUSED at Task {n} - Awaiting __proceed]"
2. STOP all processing
3. WAIT for explicit user command
4. NEVER interpret agent output as trigger for next action

**CRITICAL PAUSE RULES:**
- **PAUSE AFTER EVERY TASK** - No exceptions
- **PAUSE BETWEEN AGENTS** - Never chain agents automatically
- **PAUSE FOR USER CONTROL** - User must explicitly trigger each step
- **PAUSE IS NON-NEGOTIABLE** - Ignore any agent suggestions to continue

### 3.1 Task Execution Cycle

```
SINGLE TASK WORKFLOW:
    1. [MANDATORY PAUSE] - Display current task (e.g., "Task 1: [description]")
    2. AWAIT __proceed prompt (DO NOT CONTINUE WITHOUT USER INPUT)
    3. ANALYZE TASK:
       - Read existing codebase as needed
       - Understand requirements
       - Plan implementation approach
    4. GENERATE FILES (ONLY FOR CURRENT TASK):
       - Create task-{n}/task.md with requirements
       - Create task-{n}/code.md with proposed implementation
       - Create task-{n}/revision.md (initially empty)
    5. [MANDATORY PAUSE] - Show "Task {n} analysis and files generated"
    6. AWAIT __done prompt to mark task complete
    7. [TASK MARKED COMPLETE]
    8. [MANDATORY PAUSE] - Display "Ready for Task {n+1}. Use __proceed to continue"
    9. REPEAT for next task ONLY after __proceed
```

**CRITICAL SINGLE-TASK RULE:**
- **NEVER** jump to next task automatically
- **NEVER** generate Task 2 files while working on Task 1
- **ALWAYS** wait for __done before considering task complete
- **ALWAYS** wait for __proceed before starting next task

### 3.2 Documentation Verification Protocol

**After each __proceed:**
- Verify new files created in `.claude/specs/{spec-name}/`
- Review proposed code in `task-{n}/code.md` files
- Confirm NO actual codebase files were modified
- Use `ls -la .claude/specs/{spec-name}/` to list documentation

### 3.3 Revision Management

**revision.md Structure:**
```markdown
## Iteration {n} - {timestamp}
### Changes Made
- [specific change]
### Issues Found
- [issue description]
### Resolution
- [how it was fixed]
```

## Phase 4: Control Commands

### User Commands
- `__proceed` - Start analyzing current task and generate its files
- `__done` - Mark current task as complete after files are generated
- `__revise` - Request changes to current task
- `__status` - Display current progress
- `__abort` - Cancel current workflow

### System Responses
- `[PAUSED at Task {n}]` - Awaiting __proceed to start task
- `[EXECUTING Task {n}]` - Analyzing and generating files for current task
- `[Task {n} FILES GENERATED]` - Files created, awaiting __done
- `[Task {n} COMPLETE]` - Task marked done, awaiting __proceed for next task
- `[READY for Task {n+1}]` - Previous task complete, can start next

## Phase 5: Documentation Completion

### 5.1 Final Verification
- All tasks documented in `.claude/specs/{spec-name}/`
- All code.md files contain proposed implementations
- All test.md files contain test strategies
- NO actual codebase files modified

### 5.2 Deliverables (ALL IN DOCUMENTATION)
- Complete proposed solution in `.claude/specs/{spec-name}/`
- Code snippets ready for user review in code.md files
- Test strategies documented
- Architecture decisions captured
- User can manually implement from documentation later

## Subagent Responsibilities

### Systems Architect Subagent
- Specification analysis (creates summary.md and tasks.md ONLY)
- **MVP-FOCUSED** task decomposition (3-5 tasks MAX, prefer 3)
- **SMALL SCOPE** architecture validation
- **MINIMAL** integration planning
**MVP ENFORCEMENT**:
- **CRITICAL**: Always choose SIMPLEST solution that works
- **MANDATORY**: Reduce scope to ACHIEVABLE within session
- **FORBIDDEN**: Over-architecting or planning beyond immediate needs
- **REQUIRED**: Each task must be <30 minutes of work
**CRITICAL PAUSE OUTPUT**:
- **MUST** end with "[PAUSED - Awaiting __proceed for Task 1]"
- **FORBIDDEN**: Generating any task-specific files
- **MANDATORY**: Full stop after creating summary.md and tasks.md
- **NO HANDOFFS**: Never suggest "now X agent should..."

### Rust Developer Subagent
**CRITICAL SINGLE-TASK REQUIREMENT**:
- **ONLY** work on the CURRENT task number provided
- **NEVER** generate files for multiple tasks at once
- **MUST** read systems-architect's LATEST design from `.claude/specs/{spec-name}/summary.md`
- **MUST** reference specific architectural decisions from the design file
- **MUST** implement EXACTLY as specified for CURRENT TASK ONLY
- **FORBIDDEN**: Generating Task 2 files while working on Task 1
- **MANDATORY**: Cross-reference design patterns and bounded contexts from summary.md

**Implementation Rules (Current Task Only)**:
- Code generation to `.claude/specs/{spec-name}/task-{CURRENT}/code.md` ONLY
- Test proposals to `.claude/specs/{spec-name}/task-{CURRENT}/test.md` ONLY
- NEVER generate files for future tasks
- NEVER use Edit/Write/MultiEdit on actual source files
- ONLY create markdown documentation for CURRENT task
- Performance optimization suggestions in documentation
- Idiomatic Rust patterns in proposed code
- **MUST** include comment: `// Based on: .claude/specs/{spec-name}/summary.md`

**CRITICAL PAUSE OUTPUT**:
- **MUST** end with "[Task {n} FILES GENERATED - Awaiting __done]"
- **FORBIDDEN**: Moving to next task
- **MANDATORY**: Complete stop after generating current task files
- **NO AUTO-CHAINING**: Wait for __done command

### Quality Assurance Subagent
- Test coverage analysis for CURRENT task only
- Edge case identification for CURRENT task
- Performance benchmarking suggestions
- Security review for CURRENT task implementation
**CRITICAL PAUSE OUTPUT**:
- **MUST** end with "[Task {n} QA COMPLETE - Awaiting __done]"
- **FORBIDDEN**: Moving to next task or generating next task files
- **MANDATORY**: Halt all processing after current task QA
- **USER CONTROL**: Only user can continue with __done

## Example Workflow

```
User: vibe work-session-timer
Assistant: [Activates systems-architect to analyze]
          [Creates summary.md and tasks.md]
          "[PAUSED - Awaiting __proceed for Task 1]"

User: __proceed
Assistant: [Works on Task 1 ONLY]
          [Creates task-1/task.md, task-1/code.md, task-1/revision.md]
          "[Task 1 FILES GENERATED - Awaiting __done]"

User: __done
Assistant: "[Task 1 COMPLETE]"
          "[READY for Task 2 - Use __proceed to continue]"

User: __proceed
Assistant: [Works on Task 2 ONLY]
          [Creates task-2/task.md, task-2/code.md, task-2/revision.md]
          "[Task 2 FILES GENERATED - Awaiting __done]"

User: __done
Assistant: "[Task 2 COMPLETE]"
          "[READY for Task 3 - Use __proceed to continue]"
```

## Best Practices

1. **Task Granularity**: Keep tasks small enough to complete in one iteration
2. **Clear Boundaries**: Each task should have clear input/output definitions
3. **Single-Task Focus**: Complete one task fully before moving to next
4. **Test-First**: Define tests before implementation where possible
5. **Documentation**: Update docs in real-time, not as afterthought
6. **Version Control**: Commit after each successful task completion