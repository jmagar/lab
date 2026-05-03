# https://geminicli.com/docs/reference/tools.md
# Tools reference

Gemini CLI uses tools to interact with your local environment, access
information, and perform actions on your behalf. These tools extend the model's
capabilities beyond text generation, letting it read files, execute commands,
and search the web.

## How to use Gemini CLI's tools

Tools are generally invoked automatically by Gemini CLI when it needs to perform
an action. However, you can also trigger specific tools manually using shorthand
syntax.

### Automatic execution and security

When the model wants to use a tool, Gemini CLI evaluates the request against its
security policies.

- **User confirmation:** You must manually approve tools that modify files or
  execute shell commands (mutators). The CLI shows you a diff or the exact
  command before you confirm.
- **Sandboxing:** You can run tool executions in secure, containerized
  environments to isolate changes from your host system. For more details, see
  the [Sandboxing](/docs/cli/sandbox) guide.
- **Trusted folders:** You can configure which directories allow the model to
  use system tools. For more details, see the
  [Trusted folders](/docs/cli/trusted-folders) guide.

Review confirmation prompts carefully before allowing a tool to execute.

### How to use manually-triggered tools

You can directly trigger key tools using special syntax in your prompt:

- **[File access](/docs/tools/file-system#read_many_files) (`@`):** Use the `@`
  symbol followed by a file or directory path to include its content in your
  prompt. This triggers the `read_many_files` tool.
- **[Shell commands](/docs/tools/shell) (`!`):** Use the `!` symbol followed by
  a system command to execute it directly. This triggers the `run_shell_command`
  tool.

## How to manage tools

Using built-in commands, you can inspect available tools and configure how they
behave.

### Tool discovery

Use the `/tools` command to see what tools are currently active in your session.

- **`/tools`**: Lists all registered tools with their display names.
- **`/tools desc`**: Lists all tools with their full descriptions.

This is especially useful for verifying that
[MCP servers](/docs/tools/mcp-server) or custom tools are loaded correctly.

### Tool configuration

You can enable, disable, or configure specific tools in your settings. For
example, you can set a specific pager for shell commands or configure the
browser used for web searches. See the [Settings](/docs/cli/settings) guide for
details.

## Available tools

The following sections list all available tools, categorized by their primary
function. For detailed parameter information, see the linked documentation for
each tool.

### Execution

| Tool                                     | Kind      | Description                                                                                                              |
| :--------------------------------------- | :-------- | :----------------------------------------------------------------------------------------------------------------------- |
| [`run_shell_command`](/docs/tools/shell) | `Execute` | Executes arbitrary shell commands. Supports interactive sessions and background processes. Requires manual confirmation. |

### File System

| Tool                                         | Kind     | Description                                                                                           |
| :------------------------------------------- | :------- | :---------------------------------------------------------------------------------------------------- |
| [`glob`](/docs/tools/file-system)            | `Search` | Finds files matching specific glob patterns across the workspace.                                     |
| [`grep_search`](/docs/tools/file-system)     | `Search` | Searches for a regular expression pattern within file contents. Legacy alias: `search_file_content`.  |
| [`list_directory`](/docs/tools/file-system)  | `Read`   | Lists the names of files and subdirectories within a specified path.                                  |
| [`read_file`](/docs/tools/file-system)       | `Read`   | Reads the content of a specific file. Supports text, images, audio, and PDF.                          |
| [`read_many_files`](/docs/tools/file-system) | `Read`   | Reads and concatenates content from multiple files. Often triggered by the `@` symbol in your prompt. |
| [`replace`](/docs/tools/file-system)         | `Edit`   | Performs precise text replacement within a file. Requires manual confirmation.                        |
| [`write_file`](/docs/tools/file-system)      | `Edit`   | Creates or overwrites a file with new content. Requires manual confirmation.                          |

### Interaction

| Tool                               | Kind          | Description                                                                            |
| :--------------------------------- | :------------ | :------------------------------------------------------------------------------------- |
| [`ask_user`](/docs/tools/ask-user) | `Communicate` | Requests clarification or missing information via an interactive dialog.               |
| [`write_todos`](/docs/tools/todos) | `Other`       | Maintains an internal list of subtasks. The model uses this to track its own progress. |

### Task Tracker (Experimental)

<!-- prettier-ignore -->
> [!NOTE]
> This is an experimental feature currently under active development. Enable via `experimental.taskTracker`.

| Tool                                            | Kind    | Description                                                                 |
| :---------------------------------------------- | :------ | :-------------------------------------------------------------------------- |
| [`tracker_create_task`](/docs/tools/tracker)    | `Other` | Creates a new task in the experimental tracker.                             |
| [`tracker_update_task`](/docs/tools/tracker)    | `Other` | Updates an existing task's status, description, or dependencies.            |
| [`tracker_get_task`](/docs/tools/tracker)       | `Other` | Retrieves the full details of a specific task.                              |
| [`tracker_list_tasks`](/docs/tools/tracker)     | `Other` | Lists tasks in the tracker, optionally filtered by status, type, or parent. |
| [`tracker_add_dependency`](/docs/tools/tracker) | `Other` | Adds a dependency between two tasks, ensuring topological execution.        |
| [`tracker_visualize`](/docs/tools/tracker)      | `Other` | Renders an ASCII tree visualization of the current task graph.              |

### MCP

| Tool                                              | Kind     | Description                                                            |
| :------------------------------------------------ | :------- | :--------------------------------------------------------------------- |
| [`list_mcp_resources`](/docs/tools/mcp-resources) | `Search` | Lists all available resources exposed by connected MCP servers.        |
| [`read_mcp_resource`](/docs/tools/mcp-resources)  | `Read`   | Reads the content of a specific Model Context Protocol (MCP) resource. |

### Memory

| Tool                                             | Kind    | Description                                                                          |
| :----------------------------------------------- | :------ | :----------------------------------------------------------------------------------- |
| [`activate_skill`](/docs/tools/activate-skill)   | `Other` | Loads specialized procedural expertise from the `.gemini/skills` directory.          |
| [`get_internal_docs`](/docs/tools/internal-docs) | `Think` | Accesses Gemini CLI's own documentation for accurate answers about its capabilities. |
| [`save_memory`](/docs/tools/memory)              | `Think` | Persists specific facts and project details to your `GEMINI.md` file.                |

### Planning

| Tool                                      | Kind   | Description                                                                              |
| :---------------------------------------- | :----- | :--------------------------------------------------------------------------------------- |
| [`enter_plan_mode`](/docs/tools/planning) | `Plan` | Switches the CLI to a safe, read-only "Plan Mode" for researching complex changes.       |
| [`exit_plan_mode`](/docs/tools/planning)  | `Plan` | Finalizes a plan, presents it for review, and requests approval to start implementation. |

### System

| Tool            | Kind    | Description                                                                                                        |
| :-------------- | :------ | :----------------------------------------------------------------------------------------------------------------- |
| `complete_task` | `Other` | Finalizes a subagent's mission and returns the result to the parent agent. This tool is not available to the user. |

### Task Tracking

| Tool                     | Kind    | Description                                                                 |
| :----------------------- | :------ | :-------------------------------------------------------------------------- |
| `tracker_add_dependency` | `Think` | Adds a dependency between two existing tasks in the tracker.                |
| `tracker_create_task`    | `Think` | Creates a new task in the internal tracker to monitor progress.             |
| `tracker_get_task`       | `Think` | Retrieves the details and current status of a specific tracked task.        |
| `tracker_list_tasks`     | `Think` | Lists all tasks currently being tracked.                                    |
| `tracker_update_task`    | `Think` | Updates the status or details of an existing task.                          |
| `tracker_visualize`      | `Think` | Generates a visual representation of the current task dependency graph.     |
| `update_topic`           | `Think` | Updates the current topic and status to keep the user informed of progress. |

### Web

| Tool                                          | Kind     | Description                                                                                                                                                                                                                                                                     |
| :-------------------------------------------- | :------- | :------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| [`google_web_search`](/docs/tools/web-search) | `Search` | Performs a Google Search to find up-to-date information.                                                                                                                                                                                                                        |
| [`web_fetch`](/docs/tools/web-fetch)          | `Fetch`  | Retrieves and processes content from specific URLs. **Warning:** This tool can access local and private network addresses (for example, localhost), which may pose a security risk if used with untrusted prompts. In Plan Mode, this tool requires explicit user confirmation. |

## Under the hood

For developers, the tool system is designed to be extensible and robust. The
`ToolRegistry` class manages all available tools.

You can extend Gemini CLI with custom tools by configuring
`tools.discoveryCommand` in your settings or by connecting to MCP servers.

<!-- prettier-ignore -->
> [!NOTE]
> For a deep dive into the internal Tool API and how to implement your
> own tools in the codebase, see the `packages/core/src/tools/` directory in
> GitHub.

## Next steps

- Learn how to [Set up an MCP server](/docs/tools/mcp-server).
- Explore [Agent Skills](/docs/cli/skills) for specialized expertise.
- See the [Command reference](/docs/reference/commands) for slash commands.