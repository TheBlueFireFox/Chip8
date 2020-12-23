use std::{cell::RefCell, rc::Rc};

pub struct EventSystem<E> {
    observers: Vec<Rc<RefCell<dyn Observer<E>>>>,
}

impl<E> EventSystem<E> {
    pub fn new() -> Self {
        EventSystem {
            observers: Vec::new(),
        }
    }

    pub fn register_observer(&mut self, observer: Rc<RefCell<dyn Observer<E>>>) {
        self.observers.push(observer);
    }

    pub fn handle_event(&mut self, event: &E) {
        for observer in self.observers.iter_mut() {
            observer.borrow_mut().on_notify(event);
        }
    }
}

pub trait Observer<D> {
    fn on_notify(&mut self, event: &D);
}

#[cfg(test)]
mod tests {
    use super::{EventSystem, Observer};
    use std::{cell::RefCell, rc::Rc};

    #[derive(Copy, Clone, PartialEq, Debug)]
    pub enum ObserverEvents<T> {
        Event(T),
        None,
    }

    impl<T> Default for ObserverEvents<T> {
        fn default() -> Self {
            ObserverEvents::None
        }
    }

    #[derive(Copy, Clone, Default)]
    struct AnObserver<T: Copy> {
        pub data: T
    }

    impl<D> Observer<ObserverEvents<D>> for AnObserver<ObserverEvents<D>>
    where
        D: Copy + Default,
    {
        fn on_notify(&mut self, event: &ObserverEvents<D>) {
            match event {
                ObserverEvents::Event(_) => {
                    self.data = *event;
                }
                ObserverEvents::None => {}
            }
        }
    }

    fn setup_observer<'a, T: Default>() -> Rc<RefCell<AnObserver<T>>>
    where
        T: Copy + Default + 'a,
    {
        Rc::new(RefCell::new(Default::default()))
    }

    #[derive(Copy, Clone, PartialEq, Debug)]
    enum Event<T: Copy> {
        Event(T),
        None,
    }

    impl<T: Default + Copy> Default for Event<T> {
        fn default() -> Self {
            Event::None
        }
    }

    #[test]
    fn test_event_system() {
        // creating the observer

        let mut es = EventSystem::new();
        let observed = setup_observer();
        es.register_observer(observed.clone());
        let expected = Event::Event(42);
        let event = ObserverEvents::Event(expected);
        es.handle_event(&event);

        assert_ne!(ObserverEvents::None, observed.borrow().data);
        assert_eq!(ObserverEvents::Event(expected), observed.borrow().data);

        let event = ObserverEvents::Event(Event::None);
        es.handle_event(&event);

        assert_eq!(event, observed.borrow().data);
    }
}