use crate::jailbreak_models::*;
use anyhow::Result;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use uuid::Uuid;

pub struct PromptAnalyticsService {
    db: PgPool,
}

impl PromptAnalyticsService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn track_prompt_usage(
        &self,
        prompt_id: Uuid,
        user_id: Uuid,
        target_model: String,
        success: bool,
        response_time_ms: i64,
        feedback: Option<String>,
        rating: Option<i32>,
    ) -> Result<()> {
        // Record the usage
        sqlx::query(
            r#"
            INSERT INTO prompt_usage_records (
                id, prompt_id, user_id, target_model, success, response_time_ms, used_at, feedback, rating
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(prompt_id)
        .bind(user_id)
        .bind(&target_model)
        .bind(success)
        .bind(response_time_ms)
        .bind(Utc::now())
        .bind(&feedback)
        .bind(rating)
        .execute(&self.db)
        .await?;

        // Update prompt statistics
        self.update_prompt_statistics(prompt_id).await?;

        Ok(())
    }

    async fn update_prompt_statistics(&self, prompt_id: Uuid) -> Result<()> {
        let stats = sqlx::query(
            r#"
            SELECT 
                COUNT(*) as total_usage,
                COUNT(CASE WHEN success THEN 1 END) as successful_usage,
                AVG(response_time_ms) as avg_response_time
            FROM prompt_usage_records 
            WHERE prompt_id = $1
            "#,
        )
        .bind(prompt_id)
        .fetch_one(&self.db)
        .await?;

        let total_usage: i64 = stats.get("total_usage");
        let successful_usage: i64 = stats.get("successful_usage");

        let success_rate = if total_usage > 0 {
            successful_usage as f64 / total_usage as f64
        } else {
            0.0
        };

        sqlx::query(
            r#"
            UPDATE jailbreak_prompts 
            SET usage_count = $2,
                success_rate = $3,
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(prompt_id)
        .bind(total_usage)
        .bind(success_rate)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn get_prompt_analytics(&self, prompt_id: Uuid) -> Result<Option<PromptAnalytics>> {
        let prompt = sqlx::query_as::<_, JailbreakPrompt>(
            "SELECT * FROM jailbreak_prompts WHERE id = $1",
        )
        .bind(prompt_id)
        .fetch_optional(&self.db)
        .await?;

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
            "#,
        )
        .bind(prompt_id)
        .fetch_one(&self.db)
        .await?;

        let total_usage: i64 = stats.get("total_usage");
        let successful_usage: i64 = stats.get("successful_usage");
        let avg_response_time: Option<f64> = stats.get("avg_response_time");

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
            "#,
        )
        .bind(prompt_id)
        .fetch_all(&self.db)
        .await?;

        // Get usage trend (last 30 days)
        let trend_data = self.get_usage_trend(prompt_id, 30).await?;

        // Get last used timestamp
        let last_used_row = sqlx::query(
            "SELECT MAX(used_at) as last_used FROM prompt_usage_records WHERE prompt_id = $1",
        )
        .bind(prompt_id)
        .fetch_one(&self.db)
        .await?;

        let last_used: Option<DateTime<Utc>> = last_used_row.get("last_used");

        let analytics = PromptAnalytics {
            prompt_id,
            total_usage,
            successful_usage,
            success_rate,
            average_response_time: avg_response_time.unwrap_or(0.0),
            usage_by_model: model_stats,
            usage_trend: trend_data,
            last_used,
        };

        Ok(Some(analytics))
    }

    async fn get_usage_trend(&self, prompt_id: Uuid, days: i32) -> Result<Vec<UsageTrendData>> {
        let start_date = Utc::now() - Duration::days(days as i64);
        
        let trend_data = sqlx::query_as::<_, UsageTrendData>(
            r#"
            SELECT 
                DATE(used_at)::text as date,
                COUNT(*) as usage_count,
                CASE 
                    WHEN COUNT(*) = 0 THEN 0 
                    ELSE CAST(COUNT(CASE WHEN success THEN 1 END) AS FLOAT) / COUNT(*) 
                END as success_rate
            FROM prompt_usage_records 
            WHERE prompt_id = $1 AND used_at >= $2
            GROUP BY DATE(used_at)
            ORDER BY date DESC
            "#,
        )
        .bind(prompt_id)
        .bind(start_date)
        .fetch_all(&self.db)
        .await?;

        Ok(trend_data)
    }

    pub async fn get_global_analytics(&self) -> Result<GlobalAnalytics> {
        // Total prompts
        let total_prompts: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM jailbreak_prompts WHERE is_active = true"
        )
        .fetch_one(&self.db)
        .await?;

        // Total usage
        let total_usage: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM prompt_usage_records"
        )
        .fetch_one(&self.db)
        .await?;

        // Overall success rate
        let success_stats = sqlx::query(
            "SELECT COUNT(*) as total, COUNT(CASE WHEN success THEN 1 END) as successful FROM prompt_usage_records"
        )
        .fetch_one(&self.db)
        .await?;

        let total: i64 = success_stats.get("total");
        let successful: i64 = success_stats.get("successful");

        let overall_success_rate = if total > 0 {
            successful as f64 / total as f64
        } else {
            0.0
        };

        // Category distribution
        let category_stats = sqlx::query_as::<_, CategoryStats>(
            r#"
            SELECT 
                category,
                COUNT(*) as prompt_count,
                COALESCE(SUM(usage_count), 0) as total_usage
            FROM jailbreak_prompts 
            WHERE is_active = true
            GROUP BY category
            "#,
        )
        .fetch_all(&self.db)
        .await?;

        // Model performance
        let model_performance = sqlx::query_as::<_, ModelPerformance>(
            r#"
            SELECT 
                target_model as model,
                COUNT(*) as usage_count,
                COUNT(CASE WHEN success THEN 1 END) as success_count,
                CASE 
                    WHEN COUNT(*) = 0 THEN 0 
                    ELSE CAST(COUNT(CASE WHEN success THEN 1 END) AS FLOAT) / COUNT(*) 
                END as success_rate,
                AVG(response_time_ms) as avg_response_time
            FROM prompt_usage_records 
            GROUP BY target_model
            "#,
        )
        .fetch_all(&self.db)
        .await?;

        // Top performers
        let top_performers = sqlx::query_as::<_, JailbreakPrompt>(
            "SELECT * FROM jailbreak_prompts WHERE is_active = true ORDER BY success_rate DESC, usage_count DESC LIMIT 10"
        )
        .fetch_all(&self.db)
        .await?;

        // Recent activity (last 7 days)
        let recent_activity = self.get_recent_activity(7).await?;

        Ok(GlobalAnalytics {
            total_prompts: total_prompts.0,
            total_usage: total_usage.0,
            overall_success_rate,
            category_distribution: category_stats,
            model_performance,
            top_performers,
            recent_activity,
        })
    }

    async fn get_recent_activity(&self, days: i32) -> Result<Vec<DailyActivity>> {
        let start_date = Utc::now() - Duration::days(days as i64);
        
        let activity = sqlx::query_as::<_, DailyActivity>(
            r#"
            SELECT 
                DATE(used_at)::text as date,
                COUNT(*) as usage_count,
                COUNT(DISTINCT prompt_id) as unique_prompts,
                COUNT(DISTINCT user_id) as unique_users,
                COUNT(CASE WHEN success THEN 1 END) as successful_usage,
                AVG(response_time_ms) as avg_response_time
            FROM prompt_usage_records 
            WHERE used_at >= $1
            GROUP BY DATE(used_at)
            ORDER BY date DESC
            "#,
        )
        .bind(start_date)
        .fetch_all(&self.db)
        .await?;

        Ok(activity)
    }

    pub async fn get_user_analytics(&self, user_id: Uuid) -> Result<Option<UserAnalytics>> {
        let user_stats = sqlx::query(
            r#"
            SELECT 
                COUNT(*) as total_usage,
                COUNT(DISTINCT prompt_id) as unique_prompts_used,
                COUNT(CASE WHEN success THEN 1 END) as successful_usage,
                AVG(response_time_ms) as avg_response_time,
                MIN(used_at) as first_used,
                MAX(used_at) as last_used
            FROM prompt_usage_records 
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        let total_usage: i64 = user_stats.get("total_usage");
        let unique_prompts_used: i64 = user_stats.get("unique_prompts_used");
        let successful_usage: i64 = user_stats.get("successful_usage");
        let avg_response_time: Option<f64> = user_stats.get("avg_response_time");
        let first_used: Option<DateTime<Utc>> = user_stats.get("first_used");
        let last_used: Option<DateTime<Utc>> = user_stats.get("last_used");

        if total_usage == 0 {
            return Ok(None);
        }

        let success_rate = successful_usage as f64 / total_usage as f64;

        // Favorite categories
        let favorite_categories = sqlx::query_as::<_, FavoriteCategory>(
            r#"
            SELECT 
                jp.category,
                COUNT(*) as usage_count
            FROM prompt_usage_records pur
            JOIN jailbreak_prompts jp ON pur.prompt_id = jp.id
            WHERE pur.user_id = $1
            GROUP BY jp.category
            ORDER BY usage_count DESC
            LIMIT 5
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.db)
        .await?;

        // Preferred models
        let preferred_models = sqlx::query_as::<_, PreferredModel>(
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
            WHERE user_id = $1
            GROUP BY target_model
            ORDER BY usage_count DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.db)
        .await?;

        let analytics = UserAnalytics {
            user_id,
            total_usage,
            unique_prompts_used,
            success_rate,
            average_response_time: avg_response_time.unwrap_or(0.0),
            first_used,
            last_used,
            favorite_categories,
            preferred_models,
        };

        Ok(Some(analytics))
    }

    pub async fn generate_effectiveness_report(&self, days: i32) -> Result<EffectivenessReport> {
        let start_date = Utc::now() - Duration::days(days as i64);

        // Overall effectiveness trends
        let effectiveness_trends = sqlx::query_as::<_, EffectivenessTrend>(
            r#"
            SELECT 
                jp.effectiveness,
                COUNT(pur.id) as usage_count,
                COUNT(CASE WHEN pur.success THEN 1 END) as success_count,
                CASE 
                    WHEN COUNT(pur.id) = 0 THEN 0 
                    ELSE CAST(COUNT(CASE WHEN pur.success THEN 1 END) AS FLOAT) / COUNT(pur.id) 
                END as actual_success_rate
            FROM jailbreak_prompts jp
            LEFT JOIN prompt_usage_records pur ON jp.id = pur.prompt_id AND pur.used_at >= $1
            WHERE jp.is_active = true
            GROUP BY jp.effectiveness
            ORDER BY jp.effectiveness
            "#,
        )
        .bind(start_date)
        .fetch_all(&self.db)
        .await?;

        // Category vs effectiveness correlation
        let category_effectiveness = sqlx::query_as::<_, CategoryEffectiveness>(
            r#"
            SELECT 
                jp.category,
                jp.effectiveness as rated_effectiveness,
                COUNT(pur.id) as usage_count,
                COUNT(CASE WHEN pur.success THEN 1 END) as success_count,
                CASE 
                    WHEN COUNT(pur.id) = 0 THEN 0 
                    ELSE CAST(COUNT(CASE WHEN pur.success THEN 1 END) AS FLOAT) / COUNT(pur.id) 
                END as actual_success_rate
            FROM jailbreak_prompts jp
            LEFT JOIN prompt_usage_records pur ON jp.id = pur.prompt_id AND pur.used_at >= $1
            WHERE jp.is_active = true
            GROUP BY jp.category, jp.effectiveness
            ORDER BY jp.category, jp.effectiveness
            "#,
        )
        .bind(start_date)
        .fetch_all(&self.db)
        .await?;

        // Technique effectiveness
        let technique_effectiveness = sqlx::query_as::<_, TechniqueEffectiveness>(
            r#"
            SELECT 
                jp.technique,
                COUNT(pur.id) as usage_count,
                COUNT(CASE WHEN pur.success THEN 1 END) as success_count,
                CASE 
                    WHEN COUNT(pur.id) = 0 THEN 0 
                    ELSE CAST(COUNT(CASE WHEN pur.success THEN 1 END) AS FLOAT) / COUNT(pur.id) 
                END as success_rate,
                AVG(pur.response_time_ms) as avg_response_time
            FROM jailbreak_prompts jp
            LEFT JOIN prompt_usage_records pur ON jp.id = pur.prompt_id AND pur.used_at >= $1
            WHERE jp.is_active = true
            GROUP BY jp.technique
            ORDER BY success_rate DESC
            "#,
        )
        .bind(start_date)
        .fetch_all(&self.db)
        .await?;

        Ok(EffectivenessReport {
            period_days: days,
            effectiveness_trends,
            category_effectiveness,
            technique_effectiveness,
        })
    }
}

// Additional data structures for analytics
#[derive(Debug, Serialize, Deserialize)]
pub struct GlobalAnalytics {
    pub total_prompts: i64,
    pub total_usage: i64,
    pub overall_success_rate: f64,
    pub category_distribution: Vec<CategoryStats>,
    pub model_performance: Vec<ModelPerformance>,
    pub top_performers: Vec<JailbreakPrompt>,
    pub recent_activity: Vec<DailyActivity>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct CategoryStats {
    pub category: String,
    pub prompt_count: i64,
    pub total_usage: i64,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ModelPerformance {
    pub model: String,
    pub usage_count: i64,
    pub success_count: i64,
    pub success_rate: f64,
    pub avg_response_time: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct DailyActivity {
    pub date: String,
    pub usage_count: i64,
    pub unique_prompts: i64,
    pub unique_users: i64,
    pub successful_usage: i64,
    pub avg_response_time: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserAnalytics {
    pub user_id: Uuid,
    pub total_usage: i64,
    pub unique_prompts_used: i64,
    pub success_rate: f64,
    pub average_response_time: f64,
    pub first_used: Option<DateTime<Utc>>,
    pub last_used: Option<DateTime<Utc>>,
    pub favorite_categories: Vec<FavoriteCategory>,
    pub preferred_models: Vec<PreferredModel>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct FavoriteCategory {
    pub category: String,
    pub usage_count: i64,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct PreferredModel {
    pub model: String,
    pub usage_count: i64,
    pub success_count: i64,
    pub success_rate: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EffectivenessReport {
    pub period_days: i32,
    pub effectiveness_trends: Vec<EffectivenessTrend>,
    pub category_effectiveness: Vec<CategoryEffectiveness>,
    pub technique_effectiveness: Vec<TechniqueEffectiveness>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct EffectivenessTrend {
    pub effectiveness: String,
    pub usage_count: i64,
    pub success_count: i64,
    pub actual_success_rate: f64,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct CategoryEffectiveness {
    pub category: String,
    pub rated_effectiveness: String,
    pub usage_count: i64,
    pub success_count: i64,
    pub actual_success_rate: f64,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct TechniqueEffectiveness {
    pub technique: String,
    pub usage_count: i64,
    pub success_count: i64,
    pub success_rate: f64,
    pub avg_response_time: Option<f64>,
}
