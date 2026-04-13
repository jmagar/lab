---
description: Save session documentation with Neo4j memory integration
allowed-tools: Write, Bash, mcp__neo4j-memory__create_entities, mcp__neo4j-memory__create_relations, mcp__neo4j-memory__add_observations
---

# Save Session Documentation

Document the **entire conversation session** (not just recent work) as a markdown file at `$ARGUMENTS`. If no path is provided, save to `docs/sessions/YYYY-MM-DD-description.md` in the current working directory.

Path safety rules:
- Keep this workflow in-repo. If the target is outside the current repo root, stop and report the path issue.
- If the target filename already exists, do not overwrite. Append a suffix like `-v2`, `-v3`, etc.

## Documentation Requirements

Include in the markdown file:
1. **Session Overview**: Brief summary of what was accomplished
2. **Timeline**: Chronological breakdown of major activities
3. **Key Findings**: Important discoveries with file paths and line numbers where relevant
4. **Technical Decisions**: Reasoning behind implementation choices
5. **Files Modified**: List of all files created/modified with purpose
6. **Commands Executed**: Critical bash commands and their results
7. **Behavior Changes (Before/After)**: User-visible or system-visible behavior changes caused by this session
8. **Verification Evidence**: Table with `command | expected | actual | status`
9. **Source IDs + Collections Touched**: Every embed/retrieve source ID and collection used, with outcome
10. **Risks and Rollback**: Concise risk notes and rollback path for non-trivial changes
11. **Decisions Not Taken**: Alternatives considered but rejected, with brief rationale
12. **Open Questions**: Unresolved items or assumptions that need follow-up
13. **Next Steps**: Any remaining tasks or follow-up items

Content quality rules:
- Facts only. Do not infer values that were not observed in tool/command output.
- If something is uncertain, place it in **Open Questions** instead of stating it as fact.
- Keep sections concise (target max 5 bullets per section), but exceed when needed to preserve material implementation details, critical evidence, or safety context.
- Use file:line references (e.g., `server.ts:45`) for code-specific findings.

## Neo4j Memory Integration

After saving the markdown file, extract and store session knowledge in Neo4j:

### Entities to Create:
- **Files**: All files created, modified, or discussed (type: "file")
- **Services**: Services deployed, configured, or debugged (type: "service")
- **Features**: Features implemented or planned (type: "feature")
- **Bugs**: Issues discovered and fixed (type: "bug")
- **Technologies**: Libraries, frameworks, tools used (type: "technology")
- **Concepts**: Important patterns, techniques, or approaches (type: "concept")

### Relations to Create:
- Files → Services: `BELONGS_TO`, `CONFIGURES`, `IMPLEMENTS`
- Features → Files: `IMPLEMENTED_IN`, `REQUIRES`
- Bugs → Files: `FOUND_IN`, `FIXED_IN`
- Technologies → Features: `USED_BY`, `ENABLES`
- Services → Technologies: `DEPENDS_ON`, `USES`

### Observations to Add:
For each entity, add observations capturing:
- What was done (e.g., "Implemented OAuth2 token refresh")
- Why it was done (e.g., "Required for persistent authentication")
- How it was done (e.g., "Used @azure/msal-node library")
- When it was done (current session date)
- Any challenges or gotchas encountered

### Payload Format Guardrail (required)
- Always send Neo4j MCP payloads as native objects, not JSON strings.
- Never wrap entities/relations/observations items as stringified JSON.
- Valid shape examples:
  - `entities: [{ name, type, observations: [...] }]`
  - `relations: [{ source, target, relationType }]`
  - `observations: [{ entityName, observations: [...] }]`
- If a Neo4j call returns a schema/model-type validation error, retry once using object literals and report the retry in the final response.

Use the neo4j-memory MCP tools:
1. `mcp__neo4j-memory__create_entities` - Create all identified entities
2. `mcp__neo4j-memory__create_relations` - Link related entities
3. `mcp__neo4j-memory__add_observations` - Enrich with session details
