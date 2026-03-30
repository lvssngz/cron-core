use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub name: String,
    pub cron: String,
    pub command: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Task {
    pub fn new(name: &str, cron: &str, command: &str) -> Result<Self> {
        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            cron: cron.to_string(),
            command: command.to_string(),
            enabled: true,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn next_tick(&self) -> Option<DateTime<Utc>> {
        let schedule = cron::Schedule::from_str(&self.cron).ok()?;
        schedule.upcoming(Utc).next()
    }
}
