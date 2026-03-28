use tos_common::shell::*;

#[test]
fn test_osc_priority_parsing() {
    let mut parser = OscParser::new();
    let line = "\x1b]50;3\x07System active";
    let (text, events) = parser.process(line);
    
    assert_eq!(text, "System active");
    assert_eq!(events.len(), 1);
    match &events[0] {
        OscEvent::Priority(p) => assert_eq!(*p, 3),
        _ => panic!("Expected priority event"),
    }
}

#[test]
fn test_osc_no_events() {
    let mut parser = OscParser::new();
    let line = "Regular text";
    let (text, events) = parser.process(line);
    
    assert_eq!(text, "Regular text");
    assert_eq!(events.len(), 0);
}

#[test]
fn test_osc_partial_sequence() {
    let mut parser = OscParser::new();
    let line = "\x1b]50;3"; // Unfinished
    let (text, events) = parser.process(line);
    
    // Parser shouldn't emit events for partial sequences
    assert_eq!(text, "\x1b]50;3");
    assert_eq!(events.len(), 0);
}
