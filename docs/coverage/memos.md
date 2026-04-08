# Memos API Coverage

**Last updated:** 2026-04-08
**OpenAPI spec:** docs/api-specs/memos.openapi.yaml
**OpenAPI version:** 3.0.3
**API version:** 0.0.1
**Paths:** 62

## Legend

| Symbol | Meaning |
|--------|---------|
| ⬜ | Not implemented yet; rows are spec inventory only |
| - | Not applicable / not represented in the spec |

The source spec is the contract. This document is the implementation planning aid.

## AttachmentService

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /api/v1/attachments | AttachmentService_ListAttachments | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v1/attachments | AttachmentService_CreateAttachment | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v1/attachments/{attachment} | AttachmentService_DeleteAttachment | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/attachments/{attachment} | AttachmentService_GetAttachment | ⬜ | ⬜ | ⬜ | ⬜ |
| PATCH | /api/v1/attachments/{attachment} | AttachmentService_UpdateAttachment | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v1/attachments:batchDelete | AttachmentService_BatchDeleteAttachments | ⬜ | ⬜ | ⬜ | ⬜ |

## AuthService

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /api/v1/auth/me | AuthService_GetCurrentUser | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v1/auth/refresh | AuthService_RefreshToken | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v1/auth/signin | AuthService_SignIn | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v1/auth/signout | AuthService_SignOut | ⬜ | ⬜ | ⬜ | ⬜ |

## IdentityProviderService

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /api/v1/identity-providers | IdentityProviderService_ListIdentityProviders | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v1/identity-providers | IdentityProviderService_CreateIdentityProvider | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v1/identity-providers/{identity-provider} | IdentityProviderService_DeleteIdentityProvider | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/identity-providers/{identity-provider} | IdentityProviderService_GetIdentityProvider | ⬜ | ⬜ | ⬜ | ⬜ |
| PATCH | /api/v1/identity-providers/{identity-provider} | IdentityProviderService_UpdateIdentityProvider | ⬜ | ⬜ | ⬜ | ⬜ |

## InstanceService

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /api/v1/instance/profile | InstanceService_GetInstanceProfile | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/instance/{instance}/* | InstanceService_GetInstanceSetting | ⬜ | ⬜ | ⬜ | ⬜ |
| PATCH | /api/v1/instance/{instance}/* | InstanceService_UpdateInstanceSetting | ⬜ | ⬜ | ⬜ | ⬜ |

## MemoService

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /api/v1/memos | MemoService_ListMemos | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v1/memos | MemoService_CreateMemo | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v1/memos/{memo} | MemoService_DeleteMemo | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/memos/{memo} | MemoService_GetMemo | ⬜ | ⬜ | ⬜ | ⬜ |
| PATCH | /api/v1/memos/{memo} | MemoService_UpdateMemo | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/memos/{memo}/attachments | MemoService_ListMemoAttachments | ⬜ | ⬜ | ⬜ | ⬜ |
| PATCH | /api/v1/memos/{memo}/attachments | MemoService_SetMemoAttachments | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/memos/{memo}/comments | MemoService_ListMemoComments | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v1/memos/{memo}/comments | MemoService_CreateMemoComment | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/memos/{memo}/reactions | MemoService_ListMemoReactions | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v1/memos/{memo}/reactions | MemoService_UpsertMemoReaction | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v1/memos/{memo}/reactions/{reaction} | MemoService_DeleteMemoReaction | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/memos/{memo}/relations | MemoService_ListMemoRelations | ⬜ | ⬜ | ⬜ | ⬜ |
| PATCH | /api/v1/memos/{memo}/relations | MemoService_SetMemoRelations | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/memos/{memo}/shares | MemoService_ListMemoShares | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v1/memos/{memo}/shares | MemoService_CreateMemoShare | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v1/memos/{memo}/shares/{share} | MemoService_DeleteMemoShare | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/shares/{shareId} | MemoService_GetMemoByShare | ⬜ | ⬜ | ⬜ | ⬜ |

## ShortcutService

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /api/v1/users/{user}/shortcuts | ShortcutService_ListShortcuts | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v1/users/{user}/shortcuts | ShortcutService_CreateShortcut | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v1/users/{user}/shortcuts/{shortcut} | ShortcutService_DeleteShortcut | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/users/{user}/shortcuts/{shortcut} | ShortcutService_GetShortcut | ⬜ | ⬜ | ⬜ | ⬜ |
| PATCH | /api/v1/users/{user}/shortcuts/{shortcut} | ShortcutService_UpdateShortcut | ⬜ | ⬜ | ⬜ | ⬜ |

## UserService

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /api/v1/users | UserService_ListUsers | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v1/users | UserService_CreateUser | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v1/users/{user} | UserService_DeleteUser | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/users/{user} | UserService_GetUser | ⬜ | ⬜ | ⬜ | ⬜ |
| PATCH | /api/v1/users/{user} | UserService_UpdateUser | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/users/{user}/notifications | UserService_ListUserNotifications | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v1/users/{user}/notifications/{notification} | UserService_DeleteUserNotification | ⬜ | ⬜ | ⬜ | ⬜ |
| PATCH | /api/v1/users/{user}/notifications/{notification} | UserService_UpdateUserNotification | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/users/{user}/personalAccessTokens | UserService_ListPersonalAccessTokens | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v1/users/{user}/personalAccessTokens | UserService_CreatePersonalAccessToken | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v1/users/{user}/personalAccessTokens/{personalAccessToken} | UserService_DeletePersonalAccessToken | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/users/{user}/settings | UserService_ListUserSettings | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/users/{user}/settings/{setting} | UserService_GetUserSetting | ⬜ | ⬜ | ⬜ | ⬜ |
| PATCH | /api/v1/users/{user}/settings/{setting} | UserService_UpdateUserSetting | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/users/{user}/webhooks | UserService_ListUserWebhooks | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v1/users/{user}/webhooks | UserService_CreateUserWebhook | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v1/users/{user}/webhooks/{webhook} | UserService_DeleteUserWebhook | ⬜ | ⬜ | ⬜ | ⬜ |
| PATCH | /api/v1/users/{user}/webhooks/{webhook} | UserService_UpdateUserWebhook | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/users/{user}:getStats | UserService_GetUserStats | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v1/users:batchGet | UserService_BatchGetUsers | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/users:stats | UserService_ListAllUserStats | ⬜ | ⬜ | ⬜ | ⬜ |
