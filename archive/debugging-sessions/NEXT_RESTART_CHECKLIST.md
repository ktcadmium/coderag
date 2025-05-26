# Next Restart Checklist - Things to Try Differently

## Before You Restart Claude Code

### 1. Check Other MCP Servers
Look at the other MCP servers in `.mcp.json`:
- Do any of them work? (sequentialthinking, kubernetes, desktop-commander, jira, gitlab)
- If YES: Compare their configuration with coderag
- If NO: The issue is with ALL MCP servers, not just coderag

### 2. Try the Node.js Wrapper Approach
Create this wrapper BEFORE restarting:

```javascript
// /Users/ken/dev/MCP/mcp-coderag/coderag-wrapper.js
const { spawn } = require('child_process');
const readline = require('readline');

console.error('[Wrapper] Starting CodeRAG MCP server...');

const server = spawn('/Users/ken/dev/MCP/mcp-coderag/target/release/coderag-mcp', ['--debug']);
const rl = readline.createInterface({ input: process.stdin });

rl.on('line', (line) => {
  console.error('[Wrapper] Received:', line);
  server.stdin.write(line + '\n');
});

server.stdout.on('data', (data) => {
  console.error('[Wrapper] Sending:', data.toString());
  process.stdout.write(data);
});

server.stderr.on('data', (data) => {
  console.error('[Server]', data.toString());
});
```

Then update `.mcp.json` to use:
```json
"coderag": {
  "command": "node",
  "args": ["/Users/ken/dev/MCP/mcp-coderag/coderag-wrapper.js"]
}
```

### 3. Check Claude Code's Logs
Before restarting, check if Claude Code has logs somewhere:
```bash
# Possible locations
ls -la ~/Library/Logs/Claude*
ls -la ~/.claude/logs/
ls -la /tmp/*claude*
```

## After You Restart - Different Tests

### 1. FIRST: Check ALL MCP Access
Don't just check coderag. Check if ANY MCP tools are available:
- Look for any tools starting with `mcp__`
- Try `mcp__sequentialthinking__sequentialthinking` (if configured)
- Try `mcp__kubernetes__*` tools (if configured)

### 2. Look for Environment Clues
```bash
# Check what environment Claude Code sees
env | grep -E "(MCP|CLAUDE|PATH)" | sort

# Check process info
ps aux | grep -E "(claude|mcp|coderag)"

# Check if there's a parent process that might be relevant
ps -p $PPID
```

### 3. Test with Minimal Config
Create a backup of `.mcp.json` and try with ONLY coderag:
```json
{
  "mcpServers": {
    "coderag": {
      "command": "/Users/ken/dev/MCP/mcp-coderag/target/release/coderag-mcp"
    }
  }
}
```

### 4. Signs to Look For

**Positive Signs:**
- ANY MCP tool appears (even from other servers)
- Error messages mentioning MCP or specific servers
- Different behavior with the Node.js wrapper

**Negative Signs:**
- Complete silence about MCP
- No tools with `mcp__` prefix
- Same behavior regardless of configuration changes

### 5. Questions to Answer

1. **Is this a CodeRAG-specific issue?**
   - Test: Do other MCP servers work?

2. **Is this a configuration loading issue?**
   - Test: Does removing all but coderag from `.mcp.json` change anything?

3. **Is this a stdio communication issue?**
   - Test: Does the Node.js wrapper show any communication?

4. **Is this a permissions/security issue?**
   - Test: Does running Claude Code differently (different terminal, sudo, etc.) help?

## The Key Insight We Need

We need to determine if:
- Claude Code is not loading MCP configurations at all
- Claude Code is loading configs but failing to start servers
- Servers are starting but communication is failing
- Everything works but tools aren't being registered

## Alternative Hypothesis

What if:
- Claude Code only loads MCP config at install time, not startup?
- There's a cache somewhere that needs clearing?
- There's a different config file location we don't know about?
- MCP support requires a flag or environment variable?

## If All Else Fails

Document:
1. Exact Claude Code version and how it was installed
2. Exact OS version and architecture
3. Whether ANY MCP server has EVER worked
4. Whether this is a fresh install or upgrade

This might be a bug that needs reporting.
