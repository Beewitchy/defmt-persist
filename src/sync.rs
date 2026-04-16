
#[cfg(not(feature = "esp-sync"))]
pub type RestoreState = critical_section::RestoreState;

#[cfg(feature = "esp-sync")]
pub type RestoreState = esp_sync::RestoreState;

pub use internal::{acquire, release};

#[cfg(not(feature = "esp-sync"))]
pub mod internal {
    #[inline(always)]
    pub unsafe fn acquire() -> critical_section::RestoreState {
        unsafe { critical_section::acquire() }
    }

    #[inline(always)]
    pub unsafe fn release(restore_state: critical_section::RestoreState) {
        unsafe { critical_section::release(restore_state) }
    }
}

#[cfg(feature = "esp-sync")]
pub mod internal {
	use esp_sync::RawMutex;

    pub static LOCK: RawMutex = RawMutex::new();

    #[inline(always)]
    pub unsafe fn acquire() -> esp_sync::RestoreState {
        unsafe { LOCK.acquire() }
    }

    #[inline(always)]
    pub unsafe fn release(restore_state: esp_sync::RestoreState) {
        unsafe { LOCK.release(restore_state) }
    }
}
