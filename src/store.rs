use crate::task::Task;
use sled::Db;
use std::path::Path;
use std::sync::Arc;

pub struct Store {
    db: Arc<Db>,
}

impl Store {
    pub fn new<P: AsRef<Path>>(path: P) -> sled::Result<Self> {
        Ok(Self {
            db: Arc::new(sled::open(path)?),
        })
    }

    pub fn save(&self, task: &Task) -> sled::Result<()> {
        let tree = self.db.open_tree("tasks")?;
        tree.insert(task.id.as_bytes(), serde_json::to_vec(task).unwrap())?;
        let _ = tree.flush_async();
        Ok(())
    }

    pub fn delete(&self, id: &uuid::Uuid) -> sled::Result<bool> {
        let tree = self.db.open_tree("tasks")?;
        let existed = tree.remove(id.as_bytes())?.is_some();
        if existed {
            let _ = tree.flush_async();
        }
        Ok(existed)
    }

    pub fn get(&self, id: &uuid::Uuid) -> sled::Result<Option<Task>> {
        let tree = self.db.open_tree("tasks")?;
        tree.get(id.as_bytes())?
            .map(|v| serde_json::from_slice(&v).map_err(|e| sled::Error::Io(e.into())))
            .transpose()
    }

    pub fn list(&self) -> sled::Result<Vec<Task>> {
        let tree = self.db.open_tree("tasks")?;
        tree.iter()
            .map(|item| {
                let (_, v) = item?;
                Ok(serde_json::from_slice(&v).map_err(|e| sled::Error::Io(e.into()))?)
            })
            .collect()
    }
}