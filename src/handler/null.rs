fn handle(&self, record: &Record) -> bool {
    if record.level() < self.level {
        return false;
    }
    true
} 