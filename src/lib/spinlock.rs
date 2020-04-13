use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering::{Acquire, Release};

pub struct SpinLock<T> {
  lock: AtomicBool,
  data: UnsafeCell<T>,
}

unsafe impl<T> core::marker::Sync for SpinLock<T> {}
unsafe impl<T> core::marker::Send for SpinLock<T> {}

impl<T> SpinLock<T> {
  pub const fn new(value: T) -> SpinLock<T> {
    SpinLock {
      lock: AtomicBool::new(false),
      data: UnsafeCell::new(value),
    }
  }

  pub fn lock(&self) -> SpinLockGuard<T> {
    while !self.lock.compare_and_swap(false, true, Acquire) {
      unsafe {
        asm!("wfe");
      }
    }
    SpinLockGuard {
      lock: &self.lock,
      data: unsafe {
        &mut *self.data.get()
      },
    }
  }

  pub fn unlock(&self) {
    self.lock.store(false, Release);
    unsafe {
      asm!("sev");
    }
  }
}

pub struct SpinLockGuard<'a, T: ?Sized + 'a> {
  lock: &'a AtomicBool,
  data: &'a mut T,
}

impl<'a, T: ?Sized> Drop for SpinLockGuard<'a, T> {
  fn drop(&mut self) {
    self.lock.store(false, Release);
    unsafe {
      asm!("sev");
    }
  }
}

impl<'a, T: ?Sized> Deref for SpinLockGuard<'a, T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    &*self.data
  }
}

impl<'a, T: ?Sized> DerefMut for SpinLockGuard<'a, T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut *self.data
  }
}