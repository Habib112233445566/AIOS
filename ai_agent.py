#!/usr/bin/env python3
"""AIOS Interactive Live AI Shell (Pillar C Kernel Assistant).

Connects LLMs (MiniMax M2.7, Qwen 2.5 3B, Llama 3, DeepSeek, etc.)
to real Linux subsystems over Model Context Protocol (aiosh-mcp JSON-RPC 2.0).
Supports local execution (Ollama / vLLM) and cloud APIs (Dahl, OpenRouter, OpenAI).
"""

import argparse
import atexit
import json
import os
import subprocess
import sys
try:
    import readline  # enables command history and arrow keys in Linux terminal
except ImportError:
    pass

# 1. Argument Parsing & Provider Configuration
parser = argparse.ArgumentParser(description="AIOS Interactive Live AI Shell")
parser.add_argument(
    "--provider",
    choices=["dahl", "ollama", "openrouter", "custom"],
    default=os.getenv("AI_PROVIDER", "dahl"),
    help="Inference provider: 'dahl', 'ollama' (local), 'openrouter', or 'custom' (default: dahl)",
)
parser.add_argument(
    "--model",
    default=None,
    help="Model ID (e.g. 'qwen2.5:3b', 'MiniMaxAI/MiniMax-M2.7', 'qwen/qwen-2.5-3b-instruct')",
)
parser.add_argument(
    "--base-url",
    default=None,
    help="API base URL (overrides provider default)",
)
parser.add_argument(
    "--api-key",
    default=None,
    help="API key (overrides environment variables)",
)
parser.add_argument(
    "--tool-filter",
    default=None,
    help="Filter tools by subsystem substring (e.g. 'service', 'package', 'audit', 'fs')",
)

args, _ = parser.parse_known_args()

# Configure Provider
if args.provider == "ollama":
    base_url = args.base_url or os.getenv("OLLAMA_BASE_URL", "http://localhost:11434/v1")
    model_id = args.model or os.getenv("OLLAMA_MODEL", "qwen2.5:3b")
    api_key = args.api_key or os.getenv("OLLAMA_API_KEY", "ollama")
elif args.provider == "dahl":
    base_url = args.base_url or os.getenv("DAHL_BASE_URL", "https://inference.dahl.global/v1")
    model_id = args.model or os.getenv("DAHL_MODEL_ID", "MiniMaxAI/MiniMax-M2.7")
    api_key = args.api_key or os.getenv("DAHL_API_KEY")
    if not api_key:
        print("[-] Error: DAHL_API_KEY environment variable is not set!")
        print("    Run: export DAHL_API_KEY=\"your_key_here\" or use --api-key")
        sys.exit(1)
elif args.provider == "openrouter":
    base_url = args.base_url or "https://openrouter.ai/api/v1"
    model_id = args.model or "qwen/qwen-2.5-3b-instruct"
    api_key = args.api_key or os.getenv("OPENROUTER_API_KEY")
    if not api_key:
        print("[-] Error: OPENROUTER_API_KEY environment variable is not set!")
        sys.exit(1)
else:  # custom
    base_url = args.base_url or os.getenv("OPENAI_BASE_URL", "http://localhost:11434/v1")
    model_id = args.model or os.getenv("MODEL_ID", "qwen2.5:3b")
    api_key = args.api_key or os.getenv("OPENAI_API_KEY", "dummy-key")

# 2. Client Initialization
try:
    from openai import OpenAI
except ImportError:
    print("[-] Error: 'openai' Python package is not installed.")
    print("    Install it via: pip install openai")
    sys.exit(1)

client = OpenAI(
    base_url=base_url,
    api_key=api_key,
)
MODEL_ID = model_id

# 3. Locate and Spawn the aiosh-mcp Server
possible_paths = [
    "./code/aiosh-rust/target/debug/aiosh-mcp",
    "/content/AIOS/code/aiosh-rust/target/debug/aiosh-mcp",
    "./target/debug/aiosh-mcp",
    "./code/aiosh-rust/target/debug/aiosh-mcp.exe",
]

binary_path = None
for p in possible_paths:
    if os.path.exists(p):
        binary_path = p
        break

if not binary_path:
    print("[-] Error: Could not locate 'aiosh-mcp' executable.")
    print("    Run: cd code/aiosh-rust && cargo build --bin aiosh-mcp")
    sys.exit(1)

mcp = subprocess.Popen(
    [binary_path],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    text=True,
    bufsize=1,
)


def cleanup():
    if mcp and mcp.poll() is None:
        mcp.terminate()


atexit.register(cleanup)


def mcp_request(method, params={}):
    req = {"jsonrpc": "2.0", "id": 1, "method": method, "params": params}
    mcp.stdin.write(json.dumps(req) + "\n")
    mcp.stdin.flush()
    line = mcp.stdout.readline()
    if not line:
        return {}
    return json.loads(line)


# 4. Manifest & Tool Discovery
manifest = mcp_request("tools/list")
raw_tools = manifest.get("result", {}).get("tools", [])

if args.tool_filter:
    filter_term = args.tool_filter.lower()
    raw_tools = [t for t in raw_tools if filter_term in t.get("name", "").lower()]

tools = [
    {
        "type": "function",
        "function": {
            "name": t["name"],
            "description": t.get("description", ""),
            "parameters": t.get("inputSchema", {"type": "object", "properties": {}}),
        },
    }
    for t in raw_tools
]

SYSTEM_PROMPT = (
    "You are the AIOS Kernel Assistant, an S-rank autonomous operating system "
    "intelligence. You have direct control of the host Linux operating system "
    "through the provided MCP tools.\n"
    "When the user asks you to inspect the system, check services, look at processes, "
    "modify services, search packages, or read system files, ALWAYS invoke the appropriate "
    "tools. Be concise, precise, and authoritative. Present system status in structured tables."
)

conversation_history = [
    {"role": "system", "content": SYSTEM_PROMPT}
]


def execute_turn(user_text: str):
    conversation_history.append({"role": "user", "content": user_text})

    # Loop to allow multi-step tool execution
    while True:
        try:
            response = client.chat.completions.create(
                model=MODEL_ID,
                messages=conversation_history,
                tools=tools if tools else None,
                tool_choice="auto" if tools else None,
            )
        except Exception as e:
            print(f"\n[-] API Error: {e}")
            return

        choice = response.choices[0]
        msg = choice.message
        conversation_history.append(msg)

        if msg.tool_calls:
            for tc in msg.tool_calls:
                fn_name = tc.function.name
                try:
                    fn_args = json.loads(tc.function.arguments)
                except Exception:
                    fn_args = {}

                print(f"⚙️  [Tool Call] {fn_name}({json.dumps(fn_args, separators=(',', ':'))})")

                raw_res = mcp_request("tools/call", {
                    "name": fn_name,
                    "arguments": fn_args,
                })
                res_data = raw_res.get("result", {})

                if "content" in res_data and res_data["content"]:
                    res_str = res_data["content"][0].get("text", "")
                else:
                    res_str = json.dumps(res_data)

                # Show preview of output
                preview = res_str if len(res_str) <= 180 else res_str[:180] + "..."
                print(f"   ↳ [Result] {preview}")

                conversation_history.append({
                    "role": "tool",
                    "tool_call_id": tc.id,
                    "content": res_str,
                })
        else:
            # Clean AI response
            content = msg.content or ""
            # Strip <think> tags if model returns reasoning blocks
            if "<think>" in content and "</think>" in content:
                parts = content.split("</think>", 1)
                content = parts[1].strip()
            print(f"\n{content}\n")
            break


def main():
    print("=" * 65)
    print("          AIOS Interactive Live AI Shell (Linux)")
    print("=" * 65)
    print(f"[*] Provider         : {args.provider}")
    print(f"[*] Backend URL      : {base_url}")
    print(f"[*] Active model     : {MODEL_ID}")
    filter_note = f" (filter: '{args.tool_filter}')" if args.tool_filter else ""
    print(f"[*] OS Tools loaded  : {len(tools)} management tools{filter_note}")
    print("[*] Type your prompt or command below.")
    print("[*] Type 'exit', 'quit', or press Ctrl+C to terminate.\n")

    while True:
        try:
            prompt = input("AIOS (Linux) > ").strip()
            if not prompt:
                continue
            if prompt.lower() in ["exit", "quit", "q"]:
                print("Exiting AIOS shell. Goodbye!")
                break
            execute_turn(prompt)
        except (KeyboardInterrupt, EOFError):
            print("\nExiting AIOS shell. Goodbye!")
            break


if __name__ == "__main__":
    main()
