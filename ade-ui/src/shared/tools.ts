// GL_ agent tool suite — typed frontend bindings + agent-tool schemas (M5).
//
// The Rust side (`tools.rs`) implements each tool, sandboxed to the selected
// workspace. This module wraps the `invoke` calls with types and exposes a
// provider-agnostic tool schema (`GL_TOOLS`) plus a dispatcher (`callTool`) so a
// model tool-loop can register the suite and route tool calls to the backend.

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

// --- Result shapes ---------------------------------------------------------

export type ReadResult = {
  path: string;
  content: string;
  truncated: boolean;
  lines: number;
};

export type DirEntry = { name: string; isDir: boolean; size: number };

export type CommandOutput = {
  stdout: string;
  stderr: string;
  exitCode: number;
  timedOut: boolean;
};

// --- Typed wrappers --------------------------------------------------------

export const GL = {
  readFile: (path: string, offset?: number, limit?: number) =>
    invoke<ReadResult>("gl_read_file", { path, offset, limit }),

  writeFile: (path: string, content: string) =>
    invoke<{ path: string; bytes: number }>("gl_write_file", { path, content }),

  editFile: (path: string, oldString: string, newString: string, replaceAll = false) =>
    invoke<{ path: string; replacements: number }>("gl_edit_file", {
      path,
      oldString,
      newString,
      replaceAll,
    }),

  deleteFile: (path: string, recursive = false) =>
    invoke<{ path: string; deleted: boolean }>("gl_delete_file", { path, recursive }),

  listDir: (path?: string) => invoke<DirEntry[]>("gl_list_dir", { path }),

  grep: (pattern: string, path?: string, glob?: string) =>
    invoke<CommandOutput>("gl_grep", { pattern, path, glob }),

  glob: (pattern: string, path?: string) =>
    invoke<CommandOutput>("gl_glob", { pattern, path }),

  gitDiff: (staged = false, path?: string) =>
    invoke<CommandOutput>("gl_git_diff", { staged, path }),

  bash: (command: string, timeoutSecs?: number) =>
    invoke<CommandOutput>("gl_bash", { command, timeoutSecs }),

  diagnostics: (command?: string) =>
    invoke<CommandOutput>("gl_diagnostics", { command }),

  askUser: (question: string, options?: string[]) =>
    invoke<string>("gl_ask_user", { question, options }),

  answerUser: (id: number, answer: string) =>
    invoke<void>("gl_answer_user", { id, answer }),

  openBrowser: (url: string) => invoke<void>("gl_open_browser", { url }),
};

/// Subscribe to GL_Ask_User prompts emitted by the backend. The handler must
/// eventually call `GL.answerUser(id, answer)` to unblock the tool.
export function onAskUser(
  handler: (req: { id: number; question: string; options: string[] | null }) => void,
): Promise<() => void> {
  return listen<{ id: number; question: string; options: string[] | null }>(
    "ade://ask-user",
    (e) => handler(e.payload),
  ).then((un) => un);
}

// --- Agent-tool schema (JSON-Schema params) --------------------------------
//
// Provider-neutral; a tool-loop can translate these into Anthropic `tools`,
// OpenAI `functions`, or Gemini `functionDeclarations`.

export type ToolSchema = {
  name: string;
  description: string;
  parameters: Record<string, unknown>;
};

const str = (description: string) => ({ type: "string", description });
const num = (description: string) => ({ type: "number", description });
const bool = (description: string) => ({ type: "boolean", description });
const obj = (
  properties: Record<string, unknown>,
  required: string[] = [],
) => ({ type: "object", properties, required });

export const GL_TOOLS: ToolSchema[] = [
  {
    name: "GL_Read_File",
    description: "Read a UTF-8 text file in the workspace. Optional 1-indexed offset/limit select a line range.",
    parameters: obj(
      { path: str("Workspace-relative file path"), offset: num("1-indexed start line"), limit: num("Max lines to read") },
      ["path"],
    ),
  },
  {
    name: "GL_List_Dir",
    description: "List immediate entries of a workspace directory (dirs first, then files).",
    parameters: obj({ path: str("Workspace-relative directory (defaults to root)") }),
  },
  {
    name: "GL_Grep",
    description: "Search file contents for a pattern under the workspace (OS-native search). Optional path subtree and filename glob.",
    parameters: obj(
      { pattern: str("Text/regex pattern"), path: str("Subtree to search"), glob: str("Filename filter, e.g. *.rs") },
      ["pattern"],
    ),
  },
  {
    name: "GL_Glob",
    description: "Find files by name pattern under the workspace (OS-native). Returns matching paths.",
    parameters: obj({ pattern: str("Filename pattern, e.g. *.svelte"), path: str("Subtree to search") }, ["pattern"]),
  },
  {
    name: "GL_Git_Diff",
    description: "Run git diff in the workspace. Optionally diff the staged index or narrow to a path.",
    parameters: obj({ staged: bool("Diff the staged index"), path: str("Limit to a path") }),
  },
  {
    name: "GL_Diagnostics",
    description: "Run the project's type/lint check (auto-detected: Cargo/TS/npm/Python) and return output.",
    parameters: obj({ command: str("Override the auto-detected check command") }),
  },
  {
    name: "GL_Edit_File",
    description: "Replace an exact substring in a workspace file. old_string must be unique unless replace_all is set.",
    parameters: obj(
      {
        path: str("Workspace-relative file path"),
        old_string: str("Exact text to replace"),
        new_string: str("Replacement text"),
        replace_all: bool("Replace every occurrence"),
      },
      ["path", "old_string", "new_string"],
    ),
  },
  {
    name: "GL_Write_File",
    description: "Create or overwrite a workspace file (creates parent directories).",
    parameters: obj({ path: str("Workspace-relative file path"), content: str("File contents") }, ["path", "content"]),
  },
  {
    name: "GL_Delete_File",
    description: "Delete a workspace file or directory. Set recursive to remove a non-empty directory.",
    parameters: obj({ path: str("Workspace-relative path"), recursive: bool("Recurse into directories") }, ["path"]),
  },
  {
    name: "GL_Bash",
    description: "Run a shell command with the workspace as the working directory; returns stdout/stderr/exit code.",
    parameters: obj({ command: str("Shell command"), timeout_secs: num("Timeout in seconds (default 60, max 600)") }, ["command"]),
  },
  {
    name: "GL_Ask_User",
    description: "Ask the user a question through the UI and wait for their answer. Optionally offer choices.",
    parameters: obj({ question: str("Question to ask"), options: { type: "array", items: { type: "string" }, description: "Optional choices" } }, ["question"]),
  },
  {
    name: "GL_Open_Browser",
    description: "Open an http(s) URL in the user's default browser.",
    parameters: obj({ url: str("The http(s) URL to open") }, ["url"]),
  },
  {
    name: "GL_OpenPreview",
    description:
      "Render an artifact in the preview pane beside the chat: HTML, Markdown, a Mermaid diagram, or code. Use this to show the user a result they can look at.",
    parameters: obj(
      {
        title: str("Short title shown on the preview tab"),
        kind: { type: "string", enum: ["html", "markdown", "mermaid", "code"], description: "Artifact type" },
        content: str("The artifact source (HTML/markdown/mermaid/code)"),
        language: str("Language label when kind is code"),
      },
      ["kind", "content"],
    ),
  },
];

/// Dispatches a tool call by name (as the model emits it) to the backend.
/// `args` uses the schema's snake_case keys; this maps them to the invoke args.
export function callTool(name: string, args: Record<string, unknown>): Promise<unknown> {
  switch (name) {
    case "GL_Read_File":
      return GL.readFile(args.path as string, args.offset as number, args.limit as number);
    case "GL_List_Dir":
      return GL.listDir(args.path as string | undefined);
    case "GL_Grep":
      return GL.grep(args.pattern as string, args.path as string, args.glob as string);
    case "GL_Glob":
      return GL.glob(args.pattern as string, args.path as string);
    case "GL_Git_Diff":
      return GL.gitDiff(args.staged as boolean, args.path as string);
    case "GL_Diagnostics":
      return GL.diagnostics(args.command as string | undefined);
    case "GL_Edit_File":
      return GL.editFile(
        args.path as string,
        args.old_string as string,
        args.new_string as string,
        (args.replace_all as boolean) ?? false,
      );
    case "GL_Write_File":
      return GL.writeFile(args.path as string, args.content as string);
    case "GL_Delete_File":
      return GL.deleteFile(args.path as string, (args.recursive as boolean) ?? false);
    case "GL_Bash":
      return GL.bash(args.command as string, args.timeout_secs as number);
    case "GL_Ask_User":
      return GL.askUser(args.question as string, args.options as string[] | undefined);
    case "GL_Open_Browser":
      return GL.openBrowser(args.url as string);
    case "GL_OpenPreview":
      return openPreview(args);
    default:
      return Promise.reject(new Error(`unknown tool: ${name}`));
  }
}

/// GL_OpenPreview is handled entirely in the frontend: it populates the artifact
/// store and reveals the preview pane (no backend round-trip).
async function openPreview(args: Record<string, unknown>): Promise<string> {
  const { openArtifact } = await import("../features/chat/artifact.svelte");
  const kind = (args.kind as string) ?? "markdown";
  openArtifact({
    title: (args.title as string) ?? "Preview",
    kind: kind as "html" | "markdown" | "mermaid" | "code",
    content: (args.content as string) ?? "",
    language: args.language as string | undefined,
  });
  return `Opened preview: ${(args.title as string) ?? kind}`;
}
