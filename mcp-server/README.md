# Cyberpunk MCP Server

MCP (Model Context Protocol) server for [cyberpunk-rs](https://github.com/jonaylor89/cyberpunk-rs) - Connect LLMs like Claude to your audio processing server.

## Quick Start

### Option 1: npx (Recommended)
```bash
# Start your cyberpunk-rs server first
cargo run  # or use your deployed server

# Run the MCP server (connects to localhost:8080 by default)
npx @cyberpunk-rs/mcp-server

# Or connect to a remote server
npx @cyberpunk-rs/mcp-server --server=https://your-cyberpunk-server.run.app
```

### Option 2: Install globally
```bash
npm install -g @cyberpunk-rs/mcp-server
cyberpunk-mcp --server=http://localhost:8080
```

## Claude Desktop Integration

Add this to your Claude Desktop config:

**macOS:** `~/Library/Application Support/Claude/claude_desktop_config.json`
**Windows:** `%APPDATA%/Claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "cyberpunk-audio": {
      "command": "npx",
      "args": ["@cyberpunk-rs/mcp-server", "--server=http://localhost:8080"],
      "env": {}
    }
  }
}
```

For a deployed server:
```json
{
  "mcpServers": {
    "cyberpunk-audio": {
      "command": "npx", 
      "args": ["@cyberpunk-rs/mcp-server", "--server=https://your-app.run.app"],
      "env": {}
    }
  }
}
```

## Available Tools

### 🎵 `process_audio`
Process audio files with effects and transformations:
- **Time operations**: start_time, duration, speed, reverse
- **Volume**: volume, normalize
- **Filters**: lowpass, highpass, bass, treble  
- **Effects**: echo, chorus, flanger (use "light", "medium", "heavy")
- **Fades**: fade_in, fade_out

### 👀 `preview_audio_params`
Preview processing parameters without actually processing the audio

### 💚 `get_server_health`
Check if your cyberpunk-rs server is running

## Usage Examples

Ask Claude:
- "Process this audio with a medium echo: https://example.com/song.mp3"
- "Slow down this track to half speed and add a fade in"
- "Add heavy bass boost and normalize the levels"
- "Take the first 30 seconds and reverse it"

## Requirements

- Node.js 18+
- Running cyberpunk-rs server
- MCP-compatible LLM (Claude Desktop, etc.)

## Configuration

Set the cyberpunk server URL:
```bash
# Via command line
cyberpunk-mcp --server=https://your-server.com

# Via environment variable
export CYBERPUNK_SERVER_URL=https://your-server.com
cyberpunk-mcp
```

## Development

```bash
git clone https://github.com/jonaylor89/cyberpunk-rs.git
cd cyberpunk-rs/mcp-server
npm install
npm start
```

## License

MIT