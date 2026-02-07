use crate::jailbreak_models::*;
use anyhow::Result;
use sqlx::{PgPool, Row};
use uuid::Uuid;
use chrono::Utc;

pub struct JailbreakPromptService {
    db: PgPool,
}

impl JailbreakPromptService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn create_prompt(&self, request: CreatePromptRequest, author: String) -> Result<JailbreakPrompt> {
        let prompt = JailbreakPrompt {
            id: Uuid::new_v4(),
            title: request.title,
            content: request.content,
            category: request.category,
            technique: request.technique,
            effectiveness: request.effectiveness,
            risk_level: request.risk_level,
            target_models: request.target_models,
            description: request.description,
            tags: request.tags,
            author,
            version: "1.0".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            usage_count: 0,
            success_rate: 0.0,
            is_active: true,
            requires_ultra_tier: request.requires_ultra_tier,
        };

        let created = sqlx::query_as::<_, JailbreakPrompt>(
            r#"
            INSERT INTO jailbreak_prompts (
                id, title, content, category, technique, effectiveness, risk_level,
                target_models, description, tags, author, version, created_at, updated_at,
                usage_count, success_rate, is_active, requires_ultra_tier
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
            RETURNING *
            "#
        )
        .bind(prompt.id)
        .bind(&prompt.title)
        .bind(&prompt.content)
        .bind(prompt.category as PromptCategory)
        .bind(prompt.technique as Technique)
        .bind(prompt.effectiveness as EffectivenessRating)
        .bind(prompt.risk_level as RiskLevel)
        .bind(&prompt.target_models)
        .bind(&prompt.description)
        .bind(&prompt.tags)
        .bind(&prompt.author)
        .bind(&prompt.version)
        .bind(prompt.created_at)
        .bind(prompt.updated_at)
        .bind(prompt.usage_count)
        .bind(prompt.success_rate)
        .bind(prompt.is_active)
        .bind(prompt.requires_ultra_tier)
        .fetch_one(&self.db)
        .await?;

        Ok(created)
    }

    pub async fn get_prompt_by_id(&self, id: Uuid) -> Result<Option<JailbreakPrompt>> {
        let prompt = sqlx::query_as::<_, JailbreakPrompt>(
            "SELECT * FROM jailbreak_prompts WHERE id = $1 AND is_active = true"
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await?;

        Ok(prompt)
    }

    pub async fn update_prompt(&self, id: Uuid, request: UpdatePromptRequest) -> Result<Option<JailbreakPrompt>> {
        let existing = self.get_prompt_by_id(id).await?;
        if existing.is_none() {
            return Ok(None);
        }

        let prompt = existing.unwrap();
        let updated_prompt = JailbreakPrompt {
            title: request.title.unwrap_or(prompt.title),
            content: request.content.unwrap_or(prompt.content),
            category: request.category.unwrap_or(prompt.category),
            technique: request.technique.unwrap_or(prompt.technique),
            effectiveness: request.effectiveness.unwrap_or(prompt.effectiveness),
            risk_level: request.risk_level.unwrap_or(prompt.risk_level),
            target_models: request.target_models.unwrap_or(prompt.target_models),
            description: request.description.or(prompt.description),
            tags: request.tags.unwrap_or(prompt.tags),
            is_active: request.is_active.unwrap_or(prompt.is_active),
            requires_ultra_tier: request.requires_ultra_tier.unwrap_or(prompt.requires_ultra_tier),
            updated_at: Utc::now(),
            ..prompt
        };

        let updated = sqlx::query_as::<_, JailbreakPrompt>(
            r#"
            UPDATE jailbreak_prompts SET
                title = $2, content = $3, category = $4, technique = $5, effectiveness = $6,
                risk_level = $7, target_models = $8, description = $9, tags = $10,
                updated_at = $11, is_active = $12, requires_ultra_tier = $13
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(id)
        .bind(&updated_prompt.title)
        .bind(&updated_prompt.content)
        .bind(updated_prompt.category as PromptCategory)
        .bind(updated_prompt.technique as Technique)
        .bind(updated_prompt.effectiveness as EffectivenessRating)
        .bind(updated_prompt.risk_level as RiskLevel)
        .bind(&updated_prompt.target_models)
        .bind(&updated_prompt.description)
        .bind(&updated_prompt.tags)
        .bind(updated_prompt.updated_at)
        .bind(updated_prompt.is_active)
        .bind(updated_prompt.requires_ultra_tier)
        .fetch_one(&self.db)
        .await?;

        Ok(Some(updated))
    }

    pub async fn delete_prompt(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            "UPDATE jailbreak_prompts SET is_active = false WHERE id = $1"
        )
        .bind(id)
        .execute(&self.db)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn search_prompts(&self, request: PromptSearchRequest) -> Result<Vec<JailbreakPrompt>> {
        let limit = request.limit.unwrap_or(50).min(100);
        let offset = request.offset.unwrap_or(0);
        
        let mut query = "SELECT * FROM jailbreak_prompts WHERE is_active = true".to_string();
        let mut params = Vec::new();
        let mut param_count = 0;

        if let Some(query_text) = &request.query {
            param_count += 1;
            query.push_str(&format!(" AND (title ILIKE ${} OR content ILIKE ${} OR description ILIKE ${})", param_count, param_count, param_count));
            params.push(format!("%{}%", query_text));
        }

        if let Some(category) = &request.category {
            param_count += 1;
            query.push_str(&format!(" AND category = ${}::prompt_category", param_count));
            params.push(format!("{:?}", category).to_lowercase());
        }

        if let Some(technique) = &request.technique {
            param_count += 1;
            query.push_str(&format!(" AND technique = ${}::technique", param_count));
            params.push(format!("{:?}", technique).to_lowercase());
        }

        if let Some(effectiveness) = &request.effectiveness {
            param_count += 1;
            query.push_str(&format!(" AND effectiveness = ${}::effectiveness_rating", param_count));
            params.push(format!("{:?}", effectiveness).to_lowercase());
        }

        if let Some(risk_level) = &request.risk_level {
            param_count += 1;
            query.push_str(&format!(" AND risk_level = ${}::risk_level", param_count));
            params.push(format!("{:?}", risk_level).to_lowercase());
        }

        if let Some(requires_ultra_tier) = request.requires_ultra_tier {
            param_count += 1;
            query.push_str(&format!(" AND requires_ultra_tier = ${}", param_count));
            params.push(requires_ultra_tier.to_string());
        }

        // Add sorting
        let sort_by = request.sort_by.unwrap_or(PromptSortBy::CreatedAt);
        let sort_order = request.sort_order.unwrap_or(SortOrder::Desc);
        
        let sort_column = match sort_by {
            PromptSortBy::CreatedAt => "created_at",
            PromptSortBy::UpdatedAt => "updated_at",
            PromptSortBy::UsageCount => "usage_count",
            PromptSortBy::SuccessRate => "success_rate",
            PromptSortBy::Effectiveness => "effectiveness",
            PromptSortBy::Title => "title",
        };

        let order_direction = match sort_order {
            SortOrder::Asc => "ASC",
            SortOrder::Desc => "DESC",
        };

        query.push_str(&format!(" ORDER BY {} {} LIMIT {} OFFSET {}", sort_column, order_direction, limit, offset));

        let mut query_builder = sqlx::query_as::<_, JailbreakPrompt>(&query);
        for param in params {
            query_builder = query_builder.bind(param);
        }
        
        let prompts = query_builder
            .fetch_all(&self.db)
            .await?;

        Ok(prompts)
    }

    pub async fn get_prompts_by_category(&self, category: PromptCategory) -> Result<Vec<JailbreakPrompt>> {
        let prompts = sqlx::query_as::<_, JailbreakPrompt>(
            "SELECT * FROM jailbreak_prompts WHERE category = $1 AND is_active = true ORDER BY created_at DESC"
        )
        .bind(category as PromptCategory)
        .fetch_all(&self.db)
        .await?;

        Ok(prompts)
    }

    pub async fn get_popular_prompts(&self, limit: i64) -> Result<Vec<JailbreakPrompt>> {
        let prompts = sqlx::query_as::<_, JailbreakPrompt>(
            "SELECT * FROM jailbreak_prompts WHERE is_active = true ORDER BY usage_count DESC, success_rate DESC LIMIT $1"
        )
        .bind(limit)
        .fetch_all(&self.db)
        .await?;

        Ok(prompts)
    }

    pub async fn record_usage(&self, prompt_id: Uuid, user_id: Uuid, target_model: String, success: bool, response_time_ms: i64) -> Result<()> {
        // Record usage
        sqlx::query(
            r#"
            INSERT INTO prompt_usage_records (id, prompt_id, user_id, target_model, success, response_time_ms, used_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#
        )
        .bind(Uuid::new_v4())
        .bind(prompt_id)
        .bind(user_id)
        .bind(&target_model)
        .bind(success)
        .bind(response_time_ms)
        .bind(Utc::now())
        .execute(&self.db)
        .await?;

        // Update prompt statistics
        sqlx::query(
            r#"
            UPDATE jailbreak_prompts 
            SET usage_count = usage_count + 1,
                success_rate = (
                    SELECT CASE 
                        WHEN COUNT(*) = 0 THEN 0 
                        ELSE CAST(SUM(CASE WHEN success THEN 1 ELSE 0 END) AS FLOAT) / COUNT(*) 
                    END
                    FROM prompt_usage_records 
                    WHERE prompt_id = $1
                )
            WHERE id = $1
            "#
        )
        .bind(prompt_id)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn get_analytics(&self, prompt_id: Uuid) -> Result<Option<PromptAnalytics>> {
        let prompt = self.get_prompt_by_id(prompt_id).await?;
        if prompt.is_none() {
            return Ok(None);
        }

        // Get usage statistics
        let stats = sqlx::query(
            r#"
            SELECT 
                COUNT(*) as total_usage,
                COUNT(CASE WHEN success THEN 1 END) as successful_usage,
                AVG(response_time_ms) as avg_response_time
            FROM prompt_usage_records 
            WHERE prompt_id = $1
            "#
        )
        .bind(prompt_id)
        .fetch_one(&self.db)
        .await?;

        let total_usage: i64 = stats.get("total_usage");
        let successful_usage: i64 = stats.get("successful_usage");
        let _avg_response_time: Option<f64> = stats.get("avg_response_time");

        let success_rate = if total_usage > 0 {
            successful_usage as f64 / total_usage as f64
        } else {
            0.0
        };

        // Get usage by model
        let model_stats = sqlx::query_as::<_, ModelUsageStats>(
            r#"
            SELECT 
                target_model as model,
                COUNT(*) as usage_count,
                COUNT(CASE WHEN success THEN 1 END) as success_count,
                CASE 
                    WHEN COUNT(*) = 0 THEN 0 
                    ELSE CAST(COUNT(CASE WHEN success THEN 1 END) AS FLOAT) / COUNT(*) 
                END as success_rate
            FROM prompt_usage_records 
            WHERE prompt_id = $1
            GROUP BY target_model
            "#
        )
        .bind(prompt_id)
        .fetch_all(&self.db)
        .await?;

        // Get last used timestamp
        let last_used = sqlx::query(
            "SELECT MAX(used_at) as last_used FROM prompt_usage_records WHERE prompt_id = $1"
        )
        .bind(prompt_id)
        .fetch_one(&self.db)
        .await?;

        let _last_used_timestamp: Option<chrono::DateTime<chrono::Utc>> = last_used.get("last_used");

        let analytics = PromptAnalytics {
            prompt_id,
            total_uses: total_usage,
            unique_users: total_usage, // Simplified - should count distinct users
            average_effectiveness: success_rate * 100.0, // Convert to percentage
            trending: total_usage > 10 && success_rate > 0.7, // Simple trending logic
            popular_models: model_stats,
            usage_by_day: vec![], // TODO: Implement daily usage
        };

        Ok(Some(analytics))
    }
}
