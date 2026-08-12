# Sources

External baselines used while preparing this context pack:

- OpenAI Codex — AGENTS.md / agent configuration: https://developers.openai.com/codex/agent-configuration/agents-md
- OpenAI Codex — Subagents: https://developers.openai.com/codex/agent-configuration/subagents
- OpenAI Codex — Hooks: https://developers.openai.com/codex/hooks
- OpenAI Codex — MCP: https://developers.openai.com/codex/mcp
- OpenAI Codex — Skills: https://developers.openai.com/codex/build-skills
- OWASP Top 10:2025: https://owasp.org/Top10/2025/

Project policy deliberately avoids duplicating shared/global AI Dev Team hooks, MCP servers and generic reviewer roles. Codex hook sources can accumulate rather than replacing one another, and subagent workflows consume additional context/tokens; therefore project automation is kept domain-specific and opt-in where overlap is likely.
