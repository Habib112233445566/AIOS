#!/usr/bin/env python3
"""AIOS Interactive Live AI Shell (Pillar C Kernel Assistant).

Connects MiniMax M2.7 (via Dahl Inference) to real Linux subsystems
over Model Context Protocol (aiosh-mcp JSON-RPC 2.0).
"""

import atexit
import json
import os
import readline  # enables command history and arrow keys in Linux terminal
import subprocess
import sys
from openai import OpenAI

# 1. API Verification
api_key = os.getenv("DAHL_API_KEY")
if not api_key:
    print("[-] Error: DAHL_API_KEY environment variable is not set!")
    print("    Run: export DAHL_API_KEY=\"your_key_here\"")
    sys.exit(1)

# 2. Client Initialization
client = OpenAI(
    base_url="https://inference.dahl.global/v1",
    api_key=api_key,
)
MODEL_ID = os.getenv("DAHL_MODEL_ID", "MiniMaxAI/MiniMax-M2.7")

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
                tools=tools,
                tool_choice="auto",
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
    print("   AIOS Interactive Live AI Shell (MiniMax M2.7 on Linux)")
    print("=" * 65)
    print(f"[*] Connected backend: https://inference.dahl.global/v1")
    print(f"[*] Active model     : {MODEL_ID}")
    print(f"[*] OS Tools loaded  : {len(tools)} management tools")
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
