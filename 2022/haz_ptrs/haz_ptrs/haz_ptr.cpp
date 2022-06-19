#include "haz_ptr.hpp"

#include <thread>

void RWSpinLock::rlock() {
  while (true) {
    if (lock.fetch_add(-1, std::memory_order_seq_cst) > 0) {
      return;
    }
    lock.fetch_add(1, std::memory_order_seq_cst);
    std::this_thread::yield();
  }
}

void RWSpinLock::runlock() { lock.fetch_add(1, std::memory_order_seq_cst); }

bool RWSpinLock::try_rlock() {
  if (lock.fetch_add(-1, std::memory_order_seq_cst) > 0) {
    return true;
  }
  lock.fetch_add(1, std::memory_order_seq_cst);
  return false;
}

void RWSpinLock::wlock() {
  while (true) {
    if (lock.fetch_add(kNotLocked, std::memory_order_seq_cst) == 0) {
      return;
    }
    lock.fetch_add(kNotLocked, std::memory_order_seq_cst);
    std::this_thread::yield();
  }
}

void RWSpinLock::wunlock() {
  lock.fetch_add(kNotLocked, std::memory_order_seq_cst);
}

bool RWSpinLock::try_wlock() {
  if (lock.fetch_add(kNotLocked, std::memory_order_seq_cst) == 0) {
    return true;
  }
  lock.fetch_add(kNotLocked, std::memory_order_seq_cst);
  return false;
}