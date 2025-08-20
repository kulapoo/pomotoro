{spec} = $ARGUMENTS

You will help me develop the {spec} given. We will do vibe coding / Human in the Loop approach to finish the {spec}. NO CODE generation!!

# Planning

- "ULTRA THINK" to plan and create the tasks
- Analyze the {spec} using systems-architect subagent "MUST BE USED"
- systems-architect subagent job is to "DO" the following items with the help of other subagents accordingly:
    1. Display summary
        <example>
        # {spec-name}
        ## Summary
            [plan content and solution]
        ## Tasks
            1. create task.rs Entity in domain layer
            2. create "add task" function in task/repository.rs
            3. add unit test

        </example>
    2. Create the following files using systems-architect subagent after __proceed prompt
        - a Task lists with id as number into a {spec-name} file under
            - @.claude/specs/{spec-name}/tasks.md
            - @.claude/specs/{spec-name}/{task-1}/task.md (detailed information)
            - @.claude/specs/{spec-name}/{task-1}/code.md (code implementation)
            - @.claude/specs/{spec-name}/{task-1}/revision.md (Empty until revision, this will keep changing on every iteration)

        - the reponse summary and solution
            @.claude/specs/{spec-name}/summary.md


- "PAUSE" on each TASK item and wait until I the __proceed prompt.
- After User send __proceed, Use git capabality to verify the related changes
- We Evaluate and Iterate when needed until I the __next prompt is sent.
- Proceed to next task and "UNPAUSE"
- Repeat this whole process until we complete all TASKS


