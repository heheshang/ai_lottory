use crate::models::analysis::{
    AnalysisRequest, ColdNumbersResponse, HotNumbersResponse, NumberFrequency, NumberStatistics,
};
use crate::models::lottery::{LotteryDraw, LotteryDrawDB};
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use sqlx::{Pool, Sqlite};
use std::collections::HashMap;

pub struct AnalysisService {
    pool: Pool<Sqlite>,
}

impl AnalysisService {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    pub async fn get_hot_numbers(&self, request: AnalysisRequest) -> Result<HotNumbersResponse> {
        let draws = self.get_draws_for_analysis(&request).await?;
        let number_frequencies = self.calculate_number_frequencies(&draws).await?;

        // Sort by frequency (descending) and calculate hot scores
        let mut frequencies: Vec<NumberFrequency> = number_frequencies.into_values().collect();
        frequencies.sort_by(|a, b| {
            b.frequency
                .partial_cmp(&a.frequency)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Calculate hot scores (higher frequency = higher hot score)
        let max_freq = frequencies.first().map(|f| f.frequency).unwrap_or(0);
        for freq in &mut frequencies {
            freq.hot_score = if max_freq > 0 {
                freq.frequency as f64 / max_freq as f64
            } else {
                0.0
            };
        }

        let analysis_period = if let Some(days) = request.days {
            format!("Last {} days", days)
        } else if let Some(count) = request.draw_count {
            format!("Last {} draws", count)
        } else {
            "All time".to_string()
        };

        Ok(HotNumbersResponse {
            numbers: frequencies,
            analysis_period,
            total_draws_analyzed: draws.len() as u32,
        })
    }

    pub async fn get_cold_numbers(&self, request: AnalysisRequest) -> Result<ColdNumbersResponse> {
        let draws = self.get_draws_for_analysis(&request).await?;
        let number_frequencies = self.calculate_number_frequencies(&draws).await?;

        // Sort by frequency (ascending) and calculate cold scores
        let mut frequencies: Vec<NumberFrequency> = number_frequencies.into_values().collect();
        frequencies.sort_by(|a, b| {
            a.frequency
                .partial_cmp(&b.frequency)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Calculate cold scores (lower frequency = higher cold score)
        let max_freq = frequencies.iter().map(|f| f.frequency).max().unwrap_or(0);
        for freq in &mut frequencies {
            freq.cold_score = if max_freq > 0 {
                1.0 - (freq.frequency as f64 / max_freq as f64)
            } else {
                0.0
            };
        }

        let analysis_period = if let Some(days) = request.days {
            format!("Last {} days", days)
        } else if let Some(count) = request.draw_count {
            format!("Last {} draws", count)
        } else {
            "All time".to_string()
        };

        Ok(ColdNumbersResponse {
            numbers: frequencies,
            analysis_period,
            total_draws_analyzed: draws.len() as u32,
        })
    }

    pub async fn get_number_statistics(
        &self,
        number: u32,
        lottery_type: &str,
    ) -> Result<NumberStatistics> {
        let draws_db = sqlx::query_as::<_, LotteryDrawDB>(
            "SELECT id, draw_date, winning_numbers, bonus_number, jackpot_amount, lottery_type, created_at FROM lottery_draws WHERE lottery_type = ? ORDER BY draw_date ASC"
        )
        .bind(lottery_type)
        .fetch_all(&self.pool)
        .await?;

        let draws: Vec<LotteryDraw> = draws_db.into_iter().map(|db_draw| db_draw.into()).collect();

        let mut appearances = Vec::new();
        let mut total_draws = 0;

        for draw in &draws {
            total_draws += 1;
            if draw.winning_numbers.contains(&number) {
                appearances.push(draw.draw_date);
            }
        }

        if appearances.is_empty() {
            return Ok(NumberStatistics {
                number,
                total_draws,
                frequency: 0.0,
                average_gap: 0.0,
                current_gap: total_draws,
                longest_gap: total_draws,
                shortest_gap: 0,
            });
        }

        // Calculate gaps
        let mut gaps = Vec::new();
        let mut last_appearance: Option<DateTime<Utc>> = None;

        for draw in &draws {
            if draw.winning_numbers.contains(&number) {
                if let Some(last_date) = last_appearance {
                    let gap = (draw.draw_date.signed_duration_since(last_date).num_days()) as u32;
                    gaps.push(gap);
                }
                last_appearance = Some(draw.draw_date);
            }
        }

        // Current gap (draws since last appearance)
        let current_gap = if let Some(last_appearance) = appearances.last() {
            let days_since_last = (Utc::now() - *last_appearance).num_days() as u32;
            // Convert to approximate number of draws (assuming daily draws)
            days_since_last
        } else {
            total_draws
        };

        let average_gap = if gaps.is_empty() {
            0.0
        } else {
            gaps.iter().sum::<u32>() as f64 / gaps.len() as f64
        };

        let longest_gap = gaps.iter().max().unwrap_or(&total_draws).clone();
        let shortest_gap = gaps.iter().min().unwrap_or(&0).clone();

        Ok(NumberStatistics {
            number,
            total_draws,
            frequency: appearances.len() as f64 / total_draws as f64,
            average_gap,
            current_gap,
            longest_gap,
            shortest_gap,
        })
    }

    async fn get_draws_for_analysis(&self, request: &AnalysisRequest) -> Result<Vec<LotteryDraw>> {
        let draws_db = if let Some(days) = request.days {
            let cutoff_date = Utc::now() - Duration::days(days as i64);
            sqlx::query_as::<_, LotteryDrawDB>(
                "SELECT id, draw_date, winning_numbers, bonus_number, jackpot_amount, lottery_type, created_at FROM lottery_draws WHERE lottery_type = ? AND draw_date >= ? ORDER BY draw_date DESC"
            )
            .bind(&request.lottery_type)
            .bind(cutoff_date.to_rfc3339())
            .fetch_all(&self.pool)
            .await?
        } else if let Some(count) = request.draw_count {
            sqlx::query_as::<_, LotteryDrawDB>(
                "SELECT id, draw_date, winning_numbers, bonus_number, jackpot_amount, lottery_type, created_at FROM lottery_draws WHERE lottery_type = ? ORDER BY draw_date DESC LIMIT ?"
            )
            .bind(&request.lottery_type)
            .bind(count)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, LotteryDrawDB>(
                "SELECT id, draw_date, winning_numbers, bonus_number, jackpot_amount, lottery_type, created_at FROM lottery_draws WHERE lottery_type = ? ORDER BY draw_date DESC"
            )
            .bind(&request.lottery_type)
            .fetch_all(&self.pool)
            .await?
        };

        let draws: Vec<LotteryDraw> = draws_db.into_iter().map(|db_draw| db_draw.into()).collect();
        Ok(draws)
    }

    async fn calculate_number_frequencies(
        &self,
        draws: &[LotteryDraw],
    ) -> Result<HashMap<u32, NumberFrequency>> {
        let mut frequencies = HashMap::new();

        // Initialize frequency map for numbers 1-69 (common lottery range)
        for i in 1..=69 {
            frequencies.insert(
                i,
                NumberFrequency {
                    number: i,
                    frequency: 0,
                    last_drawn: None,
                    hot_score: 0.0,
                    cold_score: 0.0,
                },
            );
        }

        // Count frequencies
        for draw in draws {
            for &number in &draw.winning_numbers {
                if let Some(freq) = frequencies.get_mut(&number) {
                    freq.frequency += 1;
                    freq.last_drawn = Some(draw.draw_date);
                }
            }
        }

        Ok(frequencies)
    }
}
