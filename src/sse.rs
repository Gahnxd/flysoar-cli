/// A parsed SSE event with its type and data payload.
#[derive(Debug, Clone)]
pub struct SseEvent {
    pub event: String,
    pub data: String,
}

/// A streaming SSE parser that processes lines one at a time.
/// Call `push_line` for each line received; it returns `Some(SseEvent)`
/// when a complete event block has been accumulated.
pub struct SseParser {
    current_event: Option<String>,
    current_data: Vec<String>,
}

impl SseParser {
    pub fn new() -> Self {
        Self {
            current_event: None,
            current_data: Vec::new(),
        }
    }

    /// Push a raw line (without trailing newline) into the parser.
    /// Returns `Some(SseEvent)` when a complete event block is ready.
    pub fn push_line(&mut self, line: &str) -> Option<SseEvent> {
        let line = line.trim();

        // Empty line = event boundary
        if line.is_empty() {
            if self.current_event.is_some() || !self.current_data.is_empty() {
                let event = self
                    .current_event
                    .take()
                    .unwrap_or_else(|| "message".to_string());
                let data = self.current_data.join("\n");
                self.current_data.clear();
                return Some(SseEvent { event, data });
            }
            return None;
        }

        if let Some(rest) = line.strip_prefix("event:") {
            self.current_event = Some(rest.trim().to_string());
        } else if let Some(rest) = line.strip_prefix("data:") {
            self.current_data.push(rest.trim().to_string());
        }
        // Ignore comments (lines starting with ':') and unknown fields

        None
    }

    /// Flush any remaining buffered data (call when stream ends).
    pub fn flush(&mut self) -> Option<SseEvent> {
        if self.current_event.is_some() || !self.current_data.is_empty() {
            let event = self
                .current_event
                .take()
                .unwrap_or_else(|| "message".to_string());
            let data = self.current_data.join("\n");
            self.current_data.clear();
            Some(SseEvent { event, data })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_offer_event() {
        let mut parser = SseParser::new();
        assert!(parser.push_line("event: offer").is_none());
        assert!(
            parser
                .push_line(r#"data: {"id":"off_123","total_amount":"100.00"}"#)
                .is_none()
        );
        let event = parser.push_line("");
        assert!(event.is_some());
        let event = event.unwrap();
        assert_eq!(event.event, "offer");
        assert!(event.data.contains("off_123"));
    }

    #[test]
    fn test_parse_done_event() {
        let mut parser = SseParser::new();
        parser.push_line("event: done");
        parser.push_line(r#"data: {"offer_count":5}"#);
        let event = parser.push_line("").unwrap();
        assert_eq!(event.event, "done");
    }

    #[test]
    fn test_parse_error_event() {
        let mut parser = SseParser::new();
        parser.push_line("event: error");
        parser.push_line(r#"data: {"error":"Invalid IATA code","status":422}"#);
        let event = parser.push_line("").unwrap();
        assert_eq!(event.event, "error");
        assert!(event.data.contains("Invalid IATA code"));
    }

    #[test]
    fn test_flush_remaining() {
        let mut parser = SseParser::new();
        parser.push_line("event: offer");
        parser.push_line(r#"data: {"id":"off_456"}"#);
        // No empty line — simulate stream end
        let event = parser.flush();
        assert!(event.is_some());
        assert_eq!(event.unwrap().event, "offer");
    }

    #[test]
    fn test_ignores_comments() {
        let mut parser = SseParser::new();
        assert!(parser.push_line(": this is a comment").is_none());
        assert!(parser.push_line("").is_none());
    }
}
