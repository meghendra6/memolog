---
name: doctor
description: Diagnose installation/setup issues using the built-in doctor script.
argument-hint: "Target path or issue description"
agent: Sisyphus
tools: ["runInTerminal", "readFile", "textSearch", "codebase", "problems"]
---

# Doctor Mode

Task:
${input:task:Describe the target path and symptoms.}

## Workflow

| Step | Action                                                        |
| ---- | ------------------------------------------------------------- |
| 1    | Run `scripts/doctor.sh` with target path (default: repo root) |
| 2    | Summarize errors/warnings with concrete fix steps             |
| 3    | Ask before changing files                                     |
| 4    | Provide rerun command to confirm resolution                   |

## Prerequisites

| Missing      | Solution                          |
| ------------ | --------------------------------- |
| bash/python3 | Call out and suggest WSL/Git Bash |
