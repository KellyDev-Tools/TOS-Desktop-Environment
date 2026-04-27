import sys
import json

def main():
    log = open("/tmp/mock_mcp.log", "a")
    log.write("Mock MCP started\n")
    log.flush()
    while True:
        try:
            line = sys.stdin.readline()
            if not line:
                log.write("No more stdin\n")
                break
            
            log.write(f"Received: {line}\n")
            log.flush()
            request = json.loads(line)
            req_id = request.get("id")
            method = request.get("method")
            params = request.get("params", {})

            if method == "resources/read":
                # Mock GitNexus response
                prompt = params.get("prompt", "")
                result = {
                    "content": [
                        {
                            "uri": "git://repo/status",
                            "text": f"GIT STATUS for '{prompt}': On branch main. Your branch is up to date with 'origin/main'. nothing to commit, working tree clean."
                        }
                    ]
                }
                response = {
                    "jsonrpc": "2.0",
                    "id": req_id,
                    "result": result
                }
            else:
                response = {
                    "jsonrpc": "2.0",
                    "id": req_id,
                    "error": {"code": -32601, "message": "Method not found"}
                }
            
            sys.stdout.write(json.dumps(response) + "\n")
            sys.stdout.flush()
        except Exception as e:
            sys.stderr.write(f"Error: {str(e)}\n")
            sys.stderr.flush()

if __name__ == "__main__":
    main()
