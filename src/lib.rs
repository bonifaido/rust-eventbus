// #![feature(test)]
extern crate anymap;
// extern crate test;

use anymap::{AnyMap, Entry};
use std::any::Any;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::mem;

#[repr(C)]
struct TraitObject {
    pub data: *mut (),
    pub vtable: *mut (),
}

pub struct EventBus {
    handlers: AnyMap,
}

pub struct DeadEvent(Box<dyn Any>);

struct HandlerPtr<T> {
    handler: Box<dyn Fn(&T)>,
    trait_object: TraitObject,
}

impl<T> HandlerPtr<T> {
    fn new(handler: Box<dyn Fn(&T)>) -> Self {
        let trait_object: TraitObject = unsafe { mem::transmute(&*handler) };
        HandlerPtr { handler, trait_object }
    }
}

impl<T> Hash for HandlerPtr<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.trait_object.vtable as u64)
    }
}

impl<T> PartialEq for HandlerPtr<T> {
    fn eq(&self, other: &Self) -> bool {
        self.trait_object.vtable == other.trait_object.vtable
    }
}

impl<T> Eq for HandlerPtr<T> {}

type Handlers<T> = HashSet<HandlerPtr<T>>;

impl EventBus {
    pub fn new() -> EventBus {
        EventBus { handlers: AnyMap::new() }
    }

    pub fn register<T: Any, H: Fn(&T) + 'static>(&mut self, handler: H) {
        let handler_ptr = HandlerPtr::new(Box::new(handler));
        match self.handlers.entry::<Handlers<T>>() {
            Entry::Occupied(inner) => {
                inner.into_mut().insert(handler_ptr);
            }
            Entry::Vacant(inner) => {
                let mut h = HashSet::new();
                h.insert(handler_ptr);
                inner.insert(h);
            }
        }
    }

    pub fn unregister<T: Any, H: Fn(&T) + 'static>(&mut self, handler: H) {
        if let Some(handlers) = self.handlers.get_mut::<Handlers<T>>() {
            handlers.remove(&HandlerPtr::new(Box::new(handler)));
        }
    }

    pub fn unregister_all<T: Any>(&mut self) {
        self.handlers.remove::<Handlers<T>>();
    }

    #[inline(always)]
    pub fn post<T: Any>(&self, arg: T) {
        if self.dispatch(&arg) == 0 {
            let dead_event = DeadEvent(Box::new(arg));
            self.dispatch(&dead_event);
        }
    }

    fn dispatch<T: Any>(&self, arg: &T) -> usize {
        if let Some(handlers) = self.handlers.get::<Handlers<T>>() {
            for handler in handlers {
                (*handler.handler)(&arg);
            }
            handlers.len()
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::{channel, Sender};
    // use test::Bencher;

    #[test]
    fn eventbus_dispatch() {
        fn test_handler(tx: &Sender<i32>) {
            tx.send(1234).unwrap();
        }

        let mut bus = EventBus::new();

        bus.register(test_handler);

        let (tx, rx) = channel();

        bus.post(tx);

        assert_eq!(1234, rx.recv().unwrap());
    }

    #[test]
    fn eventbus_works() {
        fn my_handler(arg: &String) {
            println!("my_handler {:?}", arg);
        }

        fn my_handler_2(arg: &String) {
            println!("my_handler2 {:?}", arg);
        }

        fn my_other_handler(arg: &&str) {
            println!("my_other_handler {:?}", arg);
        }

        fn dead_event_handler(arg: &DeadEvent) {
            let &DeadEvent(ref event) = arg;
            match event.downcast_ref::<u64>() {
                Some(my_lost_number) => println!("dead_event_handler: {}", my_lost_number),
                None => println!("dead_event_handler: this wasn't for me {:?}", event)
            }
        }

        let mut bus = EventBus::new();
        bus.register(my_handler);
        bus.register(my_handler);
        bus.register(my_handler_2);
        bus.register(my_other_handler);
        bus.register(|arg: &i32| println!("my_closure_handler {:?}", arg));
        bus.register(dead_event_handler);
        bus.unregister(my_other_handler);

        bus.unregister_all::<String>();

        bus.post("Hello World".to_string());
        bus.post("Hello World");
        bus.post(123 as i32);
        bus.post(123123123 as u64);
    }

    // #[bench]
    // fn bench_single_handler(b: &mut Bencher) {
    //     fn my_str_handler(_: &&str) {}
    //
    //     let mut bus = EventBus::new();
    //     bus.register(my_str_handler);
    //     b.iter(|| bus.post("handle me"));
    // }
}
