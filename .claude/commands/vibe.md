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
- Comprehensive analysis of {spec} requirements
- Identify technical constraints and dependencies
- Map out system architecture implications
- Define success criteria

### 1.2 systems-architect subagent Activation
**CRITICAL PAUSE REQUIREMENT**:
- **MUST** end output with "[PAUSED - Awaiting __proceed]"
- **FORBIDDEN**: Suggesting next agents or implementation steps
- **MANDATORY**: Complete PAUSE before any further action
- **NO AUTOMATIC PROGRESSION**: Wait for explicit user command

**Primary Responsibilities:**
1. **Specification Analysis**
   - Parse functional requirements
   - Identify non-functional requirements
   - Map dependencies and integrations

2. **Architecture Design**
   - Define component boundaries
   - Establish data flow patterns
   - Identify design patterns needed

3. **Task Decomposition**
   - Break down into atomic, testable tasks
   - Establish task dependencies
   - Define acceptance criteria per task


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
FOR each task IN tasks:
    1. [MANDATORY PAUSE] - Display task details and current task item
    2. AWAIT __proceed prompt (DO NOT CONTINUE WITHOUT USER INPUT)
    3. EXECUTE:
       - Generate implementation in code.md
       - Document changes
    4. [MANDATORY PAUSE] - Show completed work
    5. VERIFY:
       - Use git diff to review changes
       - Validate against acceptance criteria
       - Check integration points
    6. [MANDATORY PAUSE] - Present verification results
    7. EVALUATE:
       - IF issues found:
           - Update revision.md with findings
           - Propose corrections
           - [MANDATORY PAUSE] - GOTO step 1 (re-pause)
       - ELSE:
           - Mark task complete
    8. [MANDATORY PAUSE] - AWAIT __next prompt
    9. [UNPAUSE] - ONLY after __next command, continue to next task
```

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
- `__proceed` - Execute current task and generate code
- `__next` - Mark current task complete, move to next
- `__revise` - Request changes to current task
- `__status` - Display current progress
- `__abort` - Cancel current workflow

### System Responses
- `[PAUSED at Task {n}]` - Awaiting user input
- `[EXECUTING Task {n}]` - Processing current task
- `[VERIFIED]` - Git verification complete
- `[READY for NEXT]` - Task complete, awaiting __next

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
- Specification analysis
- Task decomposition
- Architecture validation
- Integration planning
**CRITICAL PAUSE OUTPUT**:
- **MUST** end with "[PAUSED at Task {n}]"
- **FORBIDDEN**: Suggesting next agents
- **MANDATORY**: Full stop after output
- **NO HANDOFFS**: Never suggest "now X agent should..."

### Rust Developer Subagent
**CRITICAL DESIGN REFERENCE REQUIREMENT**:
- **MUST** read systems-architect's LATEST design from `.claude/specs/{spec-name}/summary.md`
- **MUST** reference specific architectural decisions from the design file
- **MUST** implement EXACTLY as specified in systems-architect's output
- **FORBIDDEN**: Deviating from architectural design without explicit user approval
- **MANDATORY**: Cross-reference design patterns and bounded contexts from summary.md

**Implementation Rules**:
- Code generation to `.claude/specs/{spec-name}/task-{n}/code.md` ONLY
- Test proposals to `.claude/specs/{spec-name}/task-{n}/test.md` ONLY
- NEVER use Edit/Write/MultiEdit on actual source files
- ONLY create markdown documentation with code snippets
- Performance optimization suggestions in documentation
- Idiomatic Rust patterns in proposed code
- **MUST** include comment: `// Based on: .claude/specs/{spec-name}/summary.md`

**CRITICAL PAUSE OUTPUT**:
- **MUST** end with "[PAUSED at Task {n}]"
- **FORBIDDEN**: Suggesting next steps
- **MANDATORY**: Complete stop after task
- **NO AUTO-CHAINING**: Wait for user command

### Quality Assurance Subagent
- Test coverage analysis
- Edge case identification
- Performance benchmarking
- Security review
**CRITICAL PAUSE OUTPUT**:
- **MUST** end with "[PAUSED at Task {n}]"
- **FORBIDDEN**: Triggering next actions
- **MANDATORY**: Halt all processing
- **USER CONTROL**: Only user can continue

## Best Practices

1. **Task Granularity**: Keep tasks small enough to complete in one iteration
2. **Clear Boundaries**: Each task should have clear input/output definitions
3. **Test-First**: Define tests before implementation where possible
4. **Documentation**: Update docs in real-time, not as afterthought
5. **Version Control**: Commit after each successful task completion