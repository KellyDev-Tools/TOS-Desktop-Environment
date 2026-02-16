Overview
Below is a versioned JSON‑RPC contract for the output.* surface added to the LCARS SDE RPC bridge. It is designed to be explicit, backward compatible, and safe: every method includes a version field, structured params, and well‑defined error codes. The schema examples use JSON Schema for request/response validation and include sample messages for typical flows (create → append → close → pin/export).

---

Versioning and transport
- Protocol: JSON‑RPC 2.0 over the existing JSON‑RPC bridge (same transport as orbital.open, payload.create, etc.).  
- Schema versioning: Each method accepts a schemaVersion string (semantic MAJOR.MINOR) in params. Servers must reject requests with incompatible MAJOR version.  
- Idempotency: output.create returns an outputId (UUID). Clients should use outputId for subsequent calls.  
- Streaming: output.append supports stream tokens for chunked output. output.close finalizes the stream.  
- Security: All export/pin operations must call portal.request and may return portal_denied error.

---

JSON Schema for RPC methods

Common error object
`json
{
  "type": "object",
  "properties": {
    "code": { "type": "integer" },
    "message": { "type": "string" },
    "data": { "type": ["object","null"] }
  },
  "required": ["code","message"]
}
`

output.create
Purpose: Create an output surface and reserve an outputId.  
Request params schema
`json
{
  "type": "object",
  "properties": {
    "schemaVersion": { "type": "string", "pattern": "^\\d+\\.\\d+$" },
    "commandId": { "type": "string" },
    "mode": { "type": "string", "enum": ["docked","cinematic","anchored"] },
    "title": { "type": "string" },
    "persistent": { "type": "boolean" },
    "profileName": { "type": ["string","null"] },
    "anchor": {
      "type": ["object","null"],
      "properties": {
        "type": { "type": "string", "enum": ["object","cluster","camera"] },
        "id": { "type": "string" }
      },
      "required": ["type","id"]
    }
  },
  "required": ["schemaVersion","mode"]
}
`
Response result schema
`json
{
  "type": "object",
  "properties": {
    "outputId": { "type": "string", "format": "uuid" },
    "createdAt": { "type": "string", "format": "date-time" },
    "mode": { "type": "string" }
  },
  "required": ["outputId","createdAt","mode"]
}
`

output.append
Purpose: Append text or structured chunks to an output surface. Supports streaming.  
Request params schema
`json
{
  "type": "object",
  "properties": {
    "schemaVersion": { "type": "string" },
    "outputId": { "type": "string", "format": "uuid" },
    "chunk": { "type": "string" },
    "stream": { "type": ["string","null"] },
    "format": { "type": "string", "enum": ["text","ansi","json","table","image"], "default": "text" },
    "meta": { "type": ["object","null"] }
  },
  "required": ["schemaVersion","outputId","chunk"]
}
`
Response result schema
`json
{
  "type": "object",
  "properties": {
    "ack": { "type": "boolean" },
    "receivedAt": { "type": "string", "format": "date-time" }
  },
  "required": ["ack","receivedAt"]
}
`

output.close
Purpose: Finalize an output stream and provide exit status.  
Request params schema
`json
{
  "type": "object",
  "properties": {
    "schemaVersion": { "type": "string" },
    "outputId": { "type": "string", "format": "uuid" },
    "status": { "type": "object",
      "properties": {
        "exitCode": { "type": "integer" },
        "signal": { "type": ["string","null"] },
        "summary": { "type": ["string","null"] }
      },
      "required": ["exitCode"]
    },
    "persist": { "type": "boolean", "default": true }
  },
  "required": ["schemaVersion","outputId","status"]
}
`
Response result schema
`json
{
  "type": "object",
  "properties": {
    "closedAt": { "type": "string", "format": "date-time" },
    "sizeBytes": { "type": "integer" }
  },
  "required": ["closedAt"]
}
`

output.pin
Purpose: Pin an output to a target (cluster, object, or dock). Requires portal validation for cross-app pins.  
Request params schema
`json
{
  "type": "object",
  "properties": {
    "schemaVersion": { "type": "string" },
    "outputId": { "type": "string", "format": "uuid" },
    "target": {
      "type": "object",
      "properties": {
        "type": { "type": "string", "enum": ["cluster","object","docked"] },
        "id": { "type": ["string","null"] }
      },
      "required": ["type"]
    },
    "share": { "type": "boolean", "default": false }
  },
  "required": ["schemaVersion","outputId","target"]
}
`
Response result schema
`json
{
  "type": "object",
  "properties": {
    "pinnedAt": { "type": "string", "format": "date-time" },
    "target": { "type": "object" }
  },
  "required": ["pinnedAt","target"]
}
`

output.search
Purpose: Search within a specific output surface.  
Request params schema
`json
{
  "type": "object",
  "properties": {
    "schemaVersion": { "type": "string" },
    "outputId": { "type": "string", "format": "uuid" },
    "query": { "type": "string" },
    "caseSensitive": { "type": "boolean", "default": false },
    "regex": { "type": "boolean", "default": false },
    "limit": { "type": "integer", "default": 100 }
  },
  "required": ["schemaVersion","outputId","query"]
}
`
Response result schema
`json
{
  "type": "object",
  "properties": {
    "matches": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "line": { "type": "integer" },
          "offset": { "type": "integer" },
          "snippet": { "type": "string" }
        },
        "required": ["line","snippet"]
      }
    }
  },
  "required": ["matches"]
}
`

output.export
Purpose: Export transcript via portal to a format. Must validate via portal.request.  
Request params schema
`json
{
  "type": "object",
  "properties": {
    "schemaVersion": { "type": "string" },
    "outputId": { "type": "string", "format": "uuid" },
    "format": { "type": "string", "enum": ["text","json","pdf"], "default": "text" },
    "options": { "type": ["object","null"] }
  },
  "required": ["schemaVersion","outputId","format"]
}
`
Response result schema
`json
{
  "type": "object",
  "properties": {
    "exportId": { "type": "string" },
    "portalRequestId": { "type": ["string","null"] },
    "status": { "type": "string", "enum": ["pending","completed","denied","failed"] }
  },
  "required": ["exportId","status"]
}
`

output.setProfile
Purpose: Apply a terminal profile to an existing output surface.  
Request params schema
`json
{
  "type": "object",
  "properties": {
    "schemaVersion": { "type": "string" },
    "outputId": { "type": "string", "format": "uuid" },
    "profileName": { "type": "string" }
  },
  "required": ["schemaVersion","outputId","profileName"]
}
`
Response result schema
`json
{
  "type": "object",
  "properties": {
    "appliedAt": { "type": "string", "format": "date-time" },
    "profileName": { "type": "string" }
  },
  "required": ["appliedAt","profileName"]
}
`

---

Error codes and semantics
- -32000 invalid_params — request failed validation.  
- -32001 not_found — outputId or target not found.  
- -32002 permission_denied — operation blocked by policy or portal.  
- -32003 quota_exceeded — output size or rate limit exceeded.  
- -32004 portal_denied — portal refused export/pin.  
- -32005 schemaversionmismatch — incompatible MAJOR version.  
- -32099 internal_error — server error; include data with diagnostics.

Errors follow the common error object schema and should include data with machine‑readable details when available.

---

Examples

Create output
`json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "output.create",
  "params": {
    "schemaVersion": "1.0",
    "commandId": "cmd-42",
    "mode": "docked",
    "title": "ls -la /home/tim",
    "persistent": true,
    "profileName": "LCARS"
  }
}
`
Response
`json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "outputId": "3fa85f64-5717-4562-b3fc-2c963f66afa6",
    "createdAt": "2026-02-01T19:35:00Z",
    "mode": "docked"
  }
}
`

Append streaming chunks
`json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "output.append",
  "params": {
    "schemaVersion": "1.0",
    "outputId": "3fa85f64-5717-4562-b3fc-2c963f66afa6",
    "chunk": "total 8\n-rw-r--r-- 1 tim users  0 Feb  1 19:30 file1",
    "stream": "stream-001",
    "format": "text"
  }
}
`

Close output
`json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "output.close",
  "params": {
    "schemaVersion": "1.0",
    "outputId": "3fa85f64-5717-4562-b3fc-2c963f66afa6",
    "status": { "exitCode": 0, "summary": "completed" },
    "persist": true
  }
}
`

Export transcript (portal flow)
`json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "output.export",
  "params": {
    "schemaVersion": "1.0",
    "outputId": "3fa85f64-5717-4562-b3fc-2c963f66afa6",
    "format": "pdf"
  }
}
`
Possible error response if portal denies
`json
{
  "jsonrpc": "2.0",
  "id": 4,
  "error": {
    "code": -32004,
    "message": "portal_denied",
    "data": { "portal": "file-save", "reason": "user_denied" }
  }
}
`

---

Developer notes and next steps
- Schema evolution: Bump MAJOR when breaking changes occur; servers must return schemaversionmismatch for incompatible requests.  
- Validation: Implement server‑side JSON Schema validation for all params. Reject early with invalid_params.  
- Streaming semantics: Use stream token to group chunks; output.close without stream finalizes entire output.  
- Mirroring rule: If mode is cinematic and persistent is true, server must create a mirrored docked transcript unless client sets mirror=false in params and user has explicitly allowed non‑mirrored cinematic outputs.  
- Portal integration: output.export and output.pin must call portal.request and propagate portal errors as portal_denied.  
- Testing: Add unit tests for schema validation, error paths, portal denial, and large stream handling.

---

If you want, I can now generate the machine‑readable JSON Schema files for each method (ready to drop into a validator) and a short server stub in your preferred language that enforces the schema and error mapping. Which language should the stub use.