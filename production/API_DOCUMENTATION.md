# Mr.DarkPromth API - Endpoint Documentation

## Base URL
```
https://bt-shop-dark.online/api
```

## Authentication
Most endpoints require JWT token in Authorization header:
```
Authorization: Bearer <token>
```

## Endpoints

### Health
```http
GET /health
```
Response: `OK`

### Authentication

#### Register
```http
POST /auth/register
Content-Type: application/json

{
  "username": "string",
  "email": "string",
  "password": "string"
}
```

#### Login
```http
POST /auth/login
Content-Type: application/json

{
  "email": "string",
  "password": "string"
}
```

### Billing

#### Get Plans
```http
GET /billing/plans
```
Response:
```json
[
  {
    "id": "uuid",
    "name": "Premium",
    "price": "299.00",
    "tier": "premium",
    "features": ["unlimited_chat", "jailbreak_prompts"]
  }
]
```

#### Create Payment
```http
POST /billing/payments
Authorization: Bearer <token>
Content-Type: application/json

{
  "plan_id": "uuid",
  "method": "promptpay"
}
```

### AI Chat

#### Get Models
```http
GET /chat/models
Authorization: Bearer <token>
```

#### Send Message
```http
POST /chat
Authorization: Bearer <token>
Content-Type: application/json

{
  "model": "gpt-4",
  "message": "string",
  "conversation_id": "uuid (optional)"
}
```

### Jailbreak Tools

#### List Prompts
```http
GET /tools/jailbreak/prompts
Authorization: Bearer <token>
```

#### Search Prompts
```http
POST /tools/jailbreak/search
Authorization: Bearer <token>
Content-Type: application/json

{
  "query": "string",
  "category": "dan",
  "technique": "direct_instruction"
}
```

#### Execute Prompt
```http
POST /tools/jailbreak/execute
Authorization: Bearer <token>
Content-Type: application/json

{
  "prompt_id": "uuid",
  "target_model": "gpt-4",
  "variables": {}
}
```

### User Management

#### Get Profile
```http
GET /user/profile
Authorization: Bearer <token>
```

#### Update Tier
```http
PUT /user/tier
Authorization: Bearer <token>
Content-Type: application/json

{
  "tier": "premium"
}
```

## Error Responses

### 400 Bad Request
```json
{
  "error": "Invalid request format"
}
```

### 401 Unauthorized
```json
{
  "error": "Invalid or expired token"
}
```

### 403 Forbidden
```json
{
  "error": "Insufficient tier level"
}
```

### 500 Internal Server Error
```json
{
  "error": "Internal server error"
}
```

## Rate Limits
- Free tier: 10 requests/minute
- Premium: 100 requests/minute  
- Ultra: 1000 requests/minute
