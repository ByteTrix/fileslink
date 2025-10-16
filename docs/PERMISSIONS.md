# FilesLink Permissions

## Overview
Permissions are managed via a JSON config file at `config/permissions.json`.

## Configuration Format
```json
{
  "allow_all": "*",
  "chats": {
    "chat1": ["1234567", 2345678],
    "chat2": "*"
  }
}
```

## Examples
- Grant access to all users:
  ```json
  { "allow_all": "*", "chats": {} }
  ```
- Grant access to specific users:
  ```json
  { "allow_all": ["1234567", "2345678"], "chats": {} }
  ```
- Special permissions for chats:
  ```json
  { "allow_all": 4567890, "chats": { "chat1": ["1234567", 2345678], "chat2": "*" } }
  ```

## Updating Permissions
- Permissions update on restart or via CLI:
  ```bash
  ./fileslink-cli update-permissions
  ```
