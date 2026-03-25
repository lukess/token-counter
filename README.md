# token-counter

A fast CLI tool to count tokens in URL responses using OpenAI's [tiktoken](https://github.com/openai/tiktoken) tokenizer.

## Features

- Count tokens from any URL response (REST APIs, web pages, etc.)
- Native MCP (Model Context Protocol) server support for `tools/list` requests
- Multiple tokenizer encodings (o200k_base, cl100k_base, p50k_base, r50k_base)
- JSON output for scripting
- Custom headers support
- TLS certificate verification skip option

## Installation

```bash
cargo build --release
cp target/release/token-counter /usr/local/bin/
```

## Usage

```bash
# Basic usage - count tokens from a URL
token-counter https://api.example.com/data

# MCP server - get token count for tools/list response
token-counter -M https://mcp-server.example.com/mcp

# Use specific encoding (default: o200k_base)
token-counter -m cl100k_base https://api.example.com/data

# JSON output
token-counter --json https://api.example.com/data

# Skip TLS verification
token-counter -k https://self-signed.example.com/api

# Custom headers
token-counter --headers "Authorization:Bearer token123" https://api.example.com/data
```

## Options

| Option | Description |
|--------|-------------|
| `-m, --model <MODEL>` | Tokenizer encoding (default: o200k_base) |
| `-M, --mcp` | Treat URL as MCP server (sends tools/list request) |
| `-t, --timeout <SECS>` | Request timeout in seconds (default: 30) |
| `-k, --insecure` | Skip TLS certificate verification |
| `-j, --json` | Output result as JSON |
| `--headers <HEADERS>` | Custom headers (format: 'Key1:Value1,Key2:Value2') |
| `--x-from <VALUE>` | X-FROM header value (default: token-counter) |

## Supported Encodings

| Encoding | Models |
|----------|--------|
| `o200k_base` | GPT-4o, GPT-4o mini (omni models) |
| `o200k_harmony` | Alias for o200k_base |
| `cl100k_base` | GPT-4, GPT-3.5-turbo, text-embedding-ada-002 |
| `p50k_base` | Codex models, text-davinci-002/003 |
| `r50k_base` | GPT-3 models (davinci, curie, babbage, ada) |

## Example Output

```
URL: https://api.example.com/data
Content-Type: application/json
Character count: 175278
Token count (o200k_base): 43153
```

JSON output:
```json
{
  "url": "https://api.example.com/data",
  "token_count": 43153,
  "char_count": 175278,
  "model": "o200k_base",
  "is_mcp": false,
  "content_type": "application/json"
}
```

## License

MIT
