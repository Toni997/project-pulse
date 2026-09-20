use super::*;

fn event(start_sample: usize, end_sample: usize) -> SchedulerEvent {
    SchedulerEvent::new(
        "clip".to_string(),
        SchedulerNodeKind::Clip,
        start_sample,
        end_sample,
    )
}

#[test]
fn a_buffer_overlapping_the_event_is_scheduled() {
    let event = event(10, 20);
    assert!(event.is_scheduled_at_position(0, 11)); // last sample of the buffer is the first of the event
    assert!(event.is_scheduled_at_position(19, 5)); // first sample of the buffer is the last of the event
    assert!(event.is_scheduled_at_position(5, 20)); // buffer covers the whole event
    assert!(event.is_scheduled_at_position(12, 2)); // buffer inside the event
}

#[test]
fn a_buffer_that_only_touches_the_event_is_not_scheduled() {
    let event = event(10, 20);
    assert!(!event.is_scheduled_at_position(0, 10)); // buffer ends exactly where the event starts
    assert!(!event.is_scheduled_at_position(20, 5)); // buffer starts exactly where the event ends
}

#[test]
fn a_buffer_clear_of_the_event_is_not_scheduled() {
    let event = event(10, 20);
    assert!(!event.is_scheduled_at_position(0, 5));
    assert!(!event.is_scheduled_at_position(30, 5));
}

#[test]
fn an_event_lies_after_a_buffer_once_it_starts_at_or_past_the_buffer_end() {
    let event = event(10, 20);
    assert!(event.is_scheduled_after_buffer(0, 10));
    assert!(event.is_scheduled_after_buffer(0, 5));
    assert!(!event.is_scheduled_after_buffer(0, 11));
    assert!(!event.is_scheduled_after_buffer(15, 4));
}
