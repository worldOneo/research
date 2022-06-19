#pragma once
#include <stdint.h>

#include <atomic>
#include <list>
#include <thread>
#include <vector>

class RWSpinLock {
 public:
  RWSpinLock() : lock(kNotLocked) {}
  void rlock();
  void runlock();
  void wlock();
  void wunlock();
  bool try_rlock();
  bool try_wlock();

 private:
  std::atomic_int64_t lock;
  static const int64_t kNotLocked = 0x2000000000;
};

struct DefaultDeleter {
  template <typename T>
  void operator()(T *ptr) {
    delete ptr;
  }
};

template <typename T, typename Deleter>
class Retired {
 public:
  T *ptr;
  uint64_t retired_at;
  Deleter deleter;

  ~Retired() {
    if (ptr) {
      deleter(ptr);
    }
  };
  Retired(T *ptr, uint64_t retired_at, Deleter deleter)
      : ptr(ptr), retired_at(retired_at), deleter(deleter) {}
};

static const uint64_t kNoEpoch = (uint64_t)~0;
static const uint64_t kEpochFreq = 1000;

class MinEpoch {
 public:
  virtual uint64_t min_epoch() = 0;
};

template <typename T, typename Deleter = DefaultDeleter>
class Cleaner {
 public:
  using atom_cnt = std::atomic_uint64_t;
  Cleaner(atom_cnt *my_e, atom_cnt *global_e, MinEpoch *min_epoch,
          const Deleter deleter = Deleter{})
      : my_e(my_e),
        global_e(global_e),
        min_epoch(min_epoch),
        deleter(deleter) {}
  ~Cleaner() {
    for (auto &ptr : retired_ptrs) {
      deleter(ptr);
    }
  }
  void enter() { my_e->store(global_e->load()); }
  void exit() { my_e->store(kNoEpoch); }
  void retire(const T *ptr) {
    auto retired = std::make_unique<Retired<const T, Deleter>>(
        ptr, global_e->load(), deleter);
    retired_ptrs.push_back(std::move(retired));
    counter++;
    if (counter % kEpochFreq == 0) {
      global_e->fetch_add(1);
    }
    if (retired_ptrs.size() % kEpochFreq == 0) {
      empty();
    }
  }

 private:
  void empty() {
    auto min_epoch_val = min_epoch->min_epoch();
    if (min_epoch_val == kNoEpoch) {
      retired_ptrs.clear();
      return;
    }

    auto predicate =
        [min_epoch_val](
            const std::unique_ptr<Retired<const T, Deleter>> &retired) {
          return retired->retired_at < min_epoch_val;
        };
    // remove all items from vectore where predicate is true
    retired_ptrs.erase(
        std::remove_if(retired_ptrs.begin(), retired_ptrs.end(), predicate),
        retired_ptrs.end());
  }
  std::vector<std::unique_ptr<Retired<const T, Deleter>>> retired_ptrs;
  atom_cnt *my_e;
  atom_cnt *global_e;
  MinEpoch *min_epoch;
  Deleter deleter;
  uint64_t counter = 0;
};

template <typename T, typename Deleter = DefaultDeleter>
class HazEpochs : public MinEpoch {
  static_assert(std::is_invocable_v<Deleter, T *>,
                "Deleter must be invocable with a T*");

  using atom_cnt = std::atomic_uint64_t;

 private:
  struct ThreadStackNode {
    std::atomic<ThreadStackNode *> next;
    Cleaner<T, Deleter> *cleaner;
  };
  class ThreadStack {
   public:
    std::atomic<ThreadStackNode *> head;
    ThreadStack() : head(nullptr) {}
    void push(Cleaner<T, Deleter> *cleaner) {
      ThreadStackNode *new_node = new ThreadStackNode{};
      ThreadStackNode *next = head.load();
      new_node->next.store(next);
      new_node->cleaner = cleaner;

      while (!head.compare_exchange_weak(next, new_node)) {
        next = head.load();
        new_node->next.store(next);
      }
    }
    Cleaner<T, Deleter> *pop() {
      ThreadStackNode *node = head.load();
      while (node != nullptr) {
        if (head.compare_exchange_weak(node, node->next.load())) {
          Cleaner<T, Deleter> *e = node->cleaner;
          delete node;
          return e;
        }
        node = head.load();
      }
      return nullptr;
    }
  };
  ThreadStack thread_stack;
  std::vector<std::unique_ptr<atom_cnt>> reservations;
  atom_cnt global_epoch;
  size_t num_threads;
  Deleter deleter;

 public:
  HazEpochs(const size_t num_threads, const Deleter deleter = Deleter{})
      : num_threads(num_threads), deleter(deleter) {
    atom_cnt *cnts = (atom_cnt *)calloc(num_threads, sizeof(atom_cnt));
    for (size_t i = 0; i < num_threads; i++) {
      cnts[i].store(kNoEpoch);
      reservations.emplace_back(std::unique_ptr<atom_cnt>(cnts + i));
    }
    global_epoch.store(0);
    for (auto &reservation : reservations) {
      auto cleaner = new Cleaner<T, Deleter>(reservation.get(), &global_epoch,
                                             this, deleter);
      thread_stack.push(cleaner);
    }
  }

  uint64_t min_epoch() override {
    uint64_t min = kNoEpoch;
    for (auto &reservation : reservations) {
      auto epoch = reservation->load();
      if (epoch < min) {
        min = epoch;
      }
    }
    return min;
  }

  Cleaner<T, Deleter> *begin() {
    auto e = thread_stack.pop();
    if (e == nullptr) {
      throw std::runtime_error("Maximum number of threads reached");
    }
    return e;
  }

  void end(Cleaner<T, Deleter> *e) { thread_stack.push(e); }
};