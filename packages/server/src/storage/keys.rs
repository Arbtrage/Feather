pub struct KeyBuilder {
    namespace: String,
}

impl KeyBuilder {
    pub fn new(namespace: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
        }
    }

    pub fn queue_pending(&self, queue: &str) -> String {
        format!("fe:{}:queue:{}:pending", self.namespace, queue)
    }

    pub fn queue_leased(&self, queue: &str) -> String {
        format!("fe:{}:queue:{}:leased", self.namespace, queue)
    }

    pub fn job(&self, id: &str) -> String {
        format!("fe:{}:job:{}", self.namespace, id)
    }

    pub fn job_events(&self, id: &str) -> String {
        format!("fe:{}:job:{}:events", self.namespace, id)
    }

    pub fn workers_active(&self) -> String {
        format!("fe:{}:workers:active", self.namespace)
    }

    pub fn worker(&self, id: &str) -> String {
        format!("fe:{}:workers:{}", self.namespace, id)
    }

    pub fn recent_jobs(&self) -> String {
        format!("fe:{}:index:jobs:recent", self.namespace)
    }

    pub fn job_prefix(&self) -> String {
        format!("fe:{}:job:", self.namespace)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn namespace_isolation() {
        let k = KeyBuilder::new("acme");
        assert_eq!(k.job("abc"), "fe:acme:job:abc");
    }
}
