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

    pub fn handle_event(&mut self, event: &ObserverEvents<E>) {
        for observer in self.observers.iter_mut() {
            observer.borrow_mut().on_notify(event);
        }
    }
}

pub enum ObserverEvents<T> {
    Event(T),
    None,
}

pub trait Observer<D> {
    fn on_notify(&mut self, event: &ObserverEvents<D>);
}

#[cfg(test)]
mod tests {
    use super::{EventSystem, Observer, ObserverEvents};
    use std::{cell::RefCell, rc::Rc};

    #[derive(Copy, Clone)]
    struct AnObserver<T: Copy> {
        pub data: Option<T>,
    }

    impl<D> Observer<D> for AnObserver<D>
    where
        D: Copy,
    {
        fn on_notify(&mut self, event: &ObserverEvents<D>) {
            match event {
                ObserverEvents::Event(data) => {
                    self.data = Some(*data);
                }
                ObserverEvents::None => {}
            }
        }
    }

    fn setup_observer<'a, T>() -> Rc<RefCell<AnObserver<T>>> where T: Copy + 'a  {
        let observer = AnObserver{
            data : None
        }; 
        Rc::new(RefCell::new(observer))
    }

    #[derive(Copy, Clone, PartialEq, Debug)]
    enum Event<T: Copy + Clone> {
        Event(T),
        None
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


        assert!(observed.borrow().data.is_some());
        assert_eq!(expected, observed.borrow().data.unwrap());
    }
}
