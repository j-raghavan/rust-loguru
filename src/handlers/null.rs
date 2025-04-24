impl Handler for NullHandler {
    fn handle(&mut self, record: &Record) -> bool {
        println!("NullHandler::handle: enabled = {}, level = {:?}, record level = {:?}", self.enabled(), self.level(), record.level());
        self.enabled() && record.level() >= self.level()
    }
} 