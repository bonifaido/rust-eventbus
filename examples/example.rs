extern crate eventbus;

fn main() {
    fn my_string_handler(arg: &String) {
        println!("my_string_handler {:?}", arg);
    }

    fn dead_event_handler(arg: &eventbus::DeadEvent) {
        let &eventbus::DeadEvent(ref event) = arg;
        match event.downcast_ref::<String>() {
            Some(string) => println!("dead_event_handler: {}", string),
            None => { println!("dead_event_handler: not printable {:?}", event) }
        }
    }

    let mut bus = eventbus::EventBus::new();

    bus.register(my_string_handler);
    bus.register(dead_event_handler);

    bus.post("hello!!!!".to_string());

    bus.register(|arg: &i32| println!("my_closure_handler {:?}", arg));

    bus.post(12);
}
