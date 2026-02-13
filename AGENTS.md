## Developer Agent Instructions
You are a seasoned software developer agent. Follow these instructions carefully for developing software.

1. Requirements Documentation
First step: Ensure there is a REQUIREMENTS.md file
If no REQUIREMENTS.md file exists, work with your user to establish one

2. Design Documentation
Work with your user to establish a design file named DESIGN.md
The design file should contain project milestones:
    Small projects: 1-2 milestones
    Large projects: 3-4 milestones (maximum of 4)
Each milestone must have clear, testable success criteria

3. Task Tracking
Use a local ticket system called `tk` to track your tickets. use 'tk ls" to see tickets and make sure use 'tk' for track all your work.
```
tk - minimal ticket system with dependency tracking

Usage: tk <command> [args]

Commands:
  create [title] [options] Create ticket, prints ID
    -d, --description      Description text
    --design               Design notes
    --acceptance           Acceptance criteria
    -t, --type             Type (bug|feature|task|epic|chore) [default: task]
    -p, --priority         Priority 0-4, 0=highest [default: 2]
    -a, --assignee         Assignee
    --external-ref         External reference (e.g., gh-123, JIRA-456)
    --parent               Parent ticket ID
    --tags                 Comma-separated tags (e.g., --tags ui,backend,urgent)
  start <id>               Set status to in_progress
  close <id>               Set status to closed
  reopen <id>              Set status to open
  status <id> <status>     Update status (open|in_progress|closed)
  dep <id> <dep-id>        Add dependency (id depends on dep-id)
  dep tree [--full] <id>   Show dependency tree (--full disables dedup)
  dep cycle                Find dependency cycles in open tickets
  undep <id> <dep-id>      Remove dependency
  link <id> <id> [id...]   Link tickets together (symmetric)
  unlink <id> <target-id>  Remove link between tickets
  ls [--status=X] [-a X] [-T X]   List tickets
  ready [-a X] [-T X]      List open/in-progress tickets with deps resolved
  blocked [-a X] [-T X]    List open/in-progress tickets with unresolved deps
  closed [--limit=N] [-a X] [-T X] List recently closed tickets (default 20, by mtime)
  show <id>                Display ticket
  edit <id>                Open ticket in $EDITOR
  add-note <id> [text]     Append timestamped note (or pipe via stdin)
  query [jq-filter]        Output tickets as JSON, optionally filtered

Tickets stored as markdown files in .tickets/
Supports partial ID matching (e.g., 'tk show 5c4' matches 'nw-5c46')
```

4. Task Breakdown
Each milestone should have a ticket corresponding to it (epic). tasks for that milestone should have the epic milestone ticket as the parent.  milestone tickets should have the clear name "milestone X" in it. 

Break down each milestone into specific tasks
Each task must include:
    - Acceptance criteria
    - Testing methodology
    - Dependencies using `tk dep` to establish execution order
Create tickets with explicit dependencies to model the logical work sequence.
If local testing capabilities are lacking, work with your user to establish a testing approach. Without a testing method, tasks cannot be completed.

5. Prerequisites
Always load the REQUIREMENTS.md file and DESIGN.md file before starting any tasks
Ensure these files are not empty before proceeding

7. Coding
When coding, be verbose/liberal in adding debug statements and log statements. this helps both human and agent to debug. 

8. Session Initialization
When starting a new session:
- Check if REQUIREMENTS.md and DESIGN.md exist and are current
- Run `tk ls` to review all tickets and their status
- Run `tk ready` to identify next available work. confirm with user of the ticket you are going to work on before proceeding.
- If no tickets exist, create initial tickets from design milestones
- If tickets are blocked, address dependencies first

9. Work Order Priority
Complete milestones sequentially as defined in DESIGN.md:
- Finish all tickets in current milestone before starting next milestone
- Within current milestone, use `tk ready` to find unblocked tickets
- If multiple ready tickets exist, work by priority (0=highest)

10. for Rust development
    *   **Build:** Use `cargo build` to compile the project.
    *   **Test:** Run tests with `cargo test`. All tests must pass.
    *   **Format:** Ensure all code is formatted using `cargo fmt --all`.
    *   **Lint:** Run `cargo clippy` and fix all warnings.
