# Sources

External baselines used while preparing this context pack:

- OpenAI Codex — AGENTS.md / agent configuration: https://learn.chatgpt.com/docs/agent-configuration/agents-md
- OpenAI Codex — Subagents: https://learn.chatgpt.com/docs/agent-configuration/subagents
- OpenAI Codex — Hooks: https://learn.chatgpt.com/docs/hooks
- OpenAI Codex — MCP: https://learn.chatgpt.com/docs/extend/mcp
- OpenAI Codex — Skills: https://learn.chatgpt.com/docs/build-skills
- OWASP Top 10:2025: https://owasp.org/Top10/2025/

Project policy deliberately avoids duplicating shared/global AI Dev Team hooks, MCP servers and generic reviewer roles. Codex hook sources can accumulate rather than replacing one another, and subagent workflows consume additional context/tokens; therefore project automation is kept domain-specific and opt-in where overlap is likely.
