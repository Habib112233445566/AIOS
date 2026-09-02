# T-00726 — Secrets & Access Hygiene / core service: Integration

## 1. Integration Scope
This task integrates the `secrets_service` scanning capabilities with the automated test suite runner `tools/test_secrets_suites.py`, validating detection criteria `K1..K4`.

## 2. Integration Deliverables
- Connected `secrets_service` to core library interfaces.
- Extended `tools/test_secrets_suites.py` with criteria checks:
  - `K1`: Data model integrity and serialization.
  - `K2`: Private key detection (`SEC-001`).
  - `K3`: API token detection (`SEC-002`, `SEC-003`).
  - `K4`: Config and `.env` credentials detection (`SEC-005`).

## 3. Test Verification Output
```text
[+] K1 data model integrity
[+] K2 private key scanner
[+] K3 API token scanner
[+] K4 config & env credentials scanner

PASS: secrets_suites criteria (K1..K4)
```
