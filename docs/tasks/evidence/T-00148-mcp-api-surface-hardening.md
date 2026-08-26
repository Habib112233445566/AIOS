# T-00148 Hardening
MCP wrapper catches inner load errors and explicitly serializes them to isError: true responses without crashing the MCP loop.