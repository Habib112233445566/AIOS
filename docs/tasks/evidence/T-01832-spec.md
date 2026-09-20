# T-01832: Network Bootstrap / MCP/API Surface: Specification

## 1. Overview
- **Task ID**: `T-01832`
- **Sub-Epic**: 4 (MCP/API Surface)
- **Goal**: Formally specify the MCP tools, input schemas, response structures, PEP gating, and error cases for Network Bootstrap.

---

## 2. MCP Tool Specifications

### 1. `aios.network.list`
- **Description**: "List discovered network interfaces on the host with status, type, MTU, and MAC address"
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "sysfs_path": { "type": "string", "description": "Optional custom sysfs root directory for hermetic discovery" },
      "procfs_path": { "type": "string", "description": "Optional custom procfs root directory" },
      "resolv_path": { "type": "string", "description": "Optional custom resolv.conf file path" }
    }
  }
  ```
- **Response**:
  ```json
  {
    "interfaces": [
      {
        "name": "eth0",
        "iftype": "ethernet",
        "operstate": "up",
        "mac_address": "02:42:ac:11:00:02",
        "mtu": 1500,
        "flags": ["UP", "BROADCAST", "RUNNING", "MULTICAST"]
      }
    ],
    "count": 1
  }
  ```
- **PEP & Audit**: `require_grant: false`, action: `"list"`, records count.

---

### 2. `aios.network.show`
- **Description**: "Inspect details for a specific network interface"
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "interface": { "type": "string", "description": "Name of the network interface to inspect (e.g., eth0, lo)" },
      "sysfs_path": { "type": "string", "description": "Optional custom sysfs root directory" },
      "procfs_path": { "type": "string", "description": "Optional custom procfs root directory" },
      "resolv_path": { "type": "string", "description": "Optional custom resolv.conf file path" }
    },
    "required": ["interface"]
  }
  ```
- **Response**:
  ```json
  {
    "interface": {
      "name": "eth0",
      "iftype": "ethernet",
      "operstate": "up",
      "mac_address": "02:42:ac:11:00:02",
      "mtu": 1500,
      "flags": ["UP", "BROADCAST", "RUNNING", "MULTICAST"]
    }
  }
  ```
- **Error Codes**: `MISSING_INTERFACE_NAME`, `INVALID_INTERFACE_NAME`, `INTERFACE_NOT_FOUND`, `SHOW_FAILED`.

---

### 3. `aios.network.routes`
- **Description**: "Query host IPv4 routing table"
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "procfs_path": { "type": "string", "description": "Optional custom procfs root directory" }
    }
  }
  ```
- **Response**:
  ```json
  {
    "routes": [
      {
        "destination": "0.0.0.0/0",
        "gateway": "192.168.1.1",
        "metric": 100,
        "interface": "eth0"
      }
    ],
    "count": 1
  }
  ```

---

### 4. `aios.network.dns`
- **Description**: "Query host DNS resolver configuration (nameservers and search domains)"
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "resolv_path": { "type": "string", "description": "Optional custom resolv.conf file path" }
    }
  }
  ```
- **Response**:
  ```json
  {
    "dns": {
      "nameservers": ["1.1.1.1", "8.8.8.8"],
      "search_domains": ["aios.local"]
    }
  }
  ```

---

### 5. `aios.network.state`
- **Description**: "Retrieve complete host network state snapshot (interfaces, routes, DNS, and hostname)"
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "sysfs_path": { "type": "string", "description": "Optional custom sysfs root directory" },
      "procfs_path": { "type": "string", "description": "Optional custom procfs root directory" },
      "resolv_path": { "type": "string", "description": "Optional custom resolv.conf file path" }
    }
  }
  ```
- **Response**:
  ```json
  {
    "state": {
      "timestamp": "2026-09-20T10:00:00Z",
      "hostname": "aios-node",
      "interfaces": [...],
      "routes": [...],
      "dns": { "nameservers": [...], "search_domains": [...] }
    }
  }
  ```

---

### 6. `aios.network.up`
- **Description**: "Bring network interface link up"
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "interface": { "type": "string", "description": "Name of the network interface to bring up" },
      "sysfs_path": { "type": "string", "description": "Optional custom sysfs root directory" }
    },
    "required": ["interface"]
  }
  ```
- **Response**: `{ "interface": "eth0", "status": "up" }`
- **PEP & Audit**: Consequential action, emits audit record with actor, interface, and status.

---

### 7. `aios.network.down`
- **Description**: "Bring network interface link down"
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "interface": { "type": "string", "description": "Name of the network interface to bring down" },
      "sysfs_path": { "type": "string", "description": "Optional custom sysfs root directory" }
    },
    "required": ["interface"]
  }
  ```
- **Response**: `{ "interface": "eth0", "status": "down" }`
- **PEP & Audit**: Consequential action, emits audit record with actor, interface, and status.
