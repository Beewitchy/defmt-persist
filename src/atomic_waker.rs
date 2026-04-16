//! Atomic waker abstraction using either critical section or esp-sync primitives

pub type AtomicWaker = internal::AtomicWaker;

#[cfg(not(feature = "esp-sync"))]
pub mod internal {
    //! Atomic waker abstraction, uses critical section.
    //!
    //! Specifically for the usecase of within drivers where a single future waits for events.

    use core::{cell::UnsafeCell, task::Waker};

    /// Utility struct used to register and wake a waker across the select branches
    pub struct AtomicWaker {
        /// The waker.
        waker: UnsafeCell<Option<Waker>>,
    }

    // SAFETY: We protect the `UnsafeCell` with critical sections.
    unsafe impl Send for AtomicWaker {}
    // SAFETY: We protect the `UnsafeCell` with critical sections.
    unsafe impl Sync for AtomicWaker {}

    impl AtomicWaker {
        /// Create a new atomic waker.
        pub const fn new() -> Self {
            Self {
                waker: UnsafeCell::new(None),
            }
        }

        /// Register a waker. Overwrites the previous waker, if any.
        pub fn register(&self, new_waker: &Waker) {
            // SAFETY: We protect the `UnsafeCell` with critical sections and do not recursively call
            // nor access `self.waker` mutably more than once.
            critical_section::with(|_| unsafe { &mut *self.waker.get() }.replace(new_waker.clone()));
        }

        /// Wake the registered waker, if any.
        pub fn wake(&self) {
            // SAFETY: We protect the `UnsafeCell` with critical sections and do not recursively call
            // nor access `self.waker` mutably more than once.
            if let Some(w) = critical_section::with(|_| unsafe { &mut *self.waker.get() }.take()) {
                w.wake();
            }
        }
    }
}

#[cfg(feature = "esp-sync")]
pub mod internal {
    use core::task::Waker;

    use embassy_sync::waitqueue::GenericAtomicWaker;
    use esp_sync::RawMutex;

    /// Utility struct to register and wake a waker.
    pub struct AtomicWaker {
        waker: GenericAtomicWaker<RawMutex>,
    }

    impl AtomicWaker {
        /// Create a new `AtomicWaker`.
        #[allow(clippy::new_without_default)]
        pub const fn new() -> Self {
            Self {
                waker: GenericAtomicWaker::new(RawMutex::new()),
            }
        }

        /// Register a waker. Overwrites the previous waker, if any.
        #[inline]
        pub fn register(&self, w: &Waker) {
            self.waker.register(w);
        }

        /// Wake the registered waker, if any.
        #[esp_hal::ram]
        pub fn wake(&self) {
            self.waker.wake();
        }
    }
}