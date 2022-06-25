#pragma once
#include <stdint.h>

#include <atomic>
#include <functional>
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

namespace haz_ptrs {

template <typename T>
struct DefaultDeleter {
  void operator()(T *ptr) { delete ptr; }
};

template <typename T>
struct DefaultFactory {
  T operator()() {
    static_assert(std::is_default_constructible<T>::value,
                  "T must be default constructible for DefaultFactory");
    return T();
  }
};

template <typename T, typename Deleter>
class Retired {
 public:
  T *ptr;
  uint64_t retired_at;
  Deleter *deleter;

  ~Retired() {
    if (ptr) {
      (*deleter)(ptr);
    }
  };
  Retired(T *ptr, uint64_t retired_at, Deleter *deleter)
      : ptr(ptr), retired_at(retired_at), deleter(deleter) {}
  Retired(const Retired &) = delete;
  Retired &operator=(const Retired &) = delete;
  Retired(Retired &&other)
      : ptr(other.ptr), retired_at(other.retired_at), deleter(other.deleter) {
    other.ptr = nullptr;
  }
  Retired &operator=(Retired &&other) {
    if (ptr) {
      (*deleter)(ptr);
    }
    ptr = other.ptr;
    retired_at = other.retired_at;
    deleter = other.deleter;
    other.ptr = nullptr;
    return *this;
  }
};

static const uint64_t kNoEpoch = (uint64_t)~0;
static const uint64_t kCounterFreq = 16;
static const uint64_t kEpochFreq = 8;

class MinEpoch {
 public:
  virtual uint64_t min_epoch() = 0;
};

template <typename T, typename Deleter>
class Cleaner {
 public:
  using atom_cnt = std::atomic_uint64_t;
  Cleaner(atom_cnt *my_e, atom_cnt *global_e, MinEpoch *min_epoch,
          Deleter *deleter = Deleter{})
      : my_e(my_e),
        global_e(global_e),
        min_epoch(min_epoch),
        deleter(deleter) {}
  ~Cleaner() {
    for (auto &ptr : retired_ptrs) {
      (*deleter)(ptr.ptr);
    }
  }
  void enter() { my_e->store(global_e->load()); }
  void exit() { my_e->store(kNoEpoch); }
  void retire(T *ptr) {
    auto retired = Retired<T, Deleter>(ptr, global_e->load(), deleter);
    retired_ptrs.push_back(std::move(retired));
    counter++;
    if (counter % kCounterFreq == 0) {
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

    auto predicate = [min_epoch_val](const Retired<T, Deleter> &retired) {
      return retired.retired_at < min_epoch_val;
    };
    // remove all items from vectore where predicate is true
    retired_ptrs.erase(
        std::remove_if(retired_ptrs.begin(), retired_ptrs.end(), predicate),
        retired_ptrs.end());
  }
  std::vector<Retired<T, Deleter>> retired_ptrs;
  atom_cnt *my_e;
  atom_cnt *global_e;
  MinEpoch *min_epoch;
  Deleter *deleter;
  uint64_t counter = 0;
};

template <typename T>
struct ThreadStackNode {
  std::atomic<ThreadStackNode *> next;
  T *value;
};

template <typename T>
class ThreadStack {
 public:
  std::atomic<ThreadStackNode<T> *> head;
  ThreadStack() : head(nullptr) {}
  void push(T *value) {
    ThreadStackNode<T> *new_node = new ThreadStackNode<T>{};
    ThreadStackNode<T> *next = head.load();
    new_node->next.store(next);
    new_node->value = value;

    while (!head.compare_exchange_weak(next, new_node)) {
      next = head.load();
      new_node->next.store(next);
    }
  }

  T *pop() {
    ThreadStackNode<T> *node = head.load();
    while (node != nullptr) {
      if (head.compare_exchange_weak(node, node->next.load())) {
        T *e = node->value;
        delete node;
        return e;
      }
      node = head.load();
    }
    return nullptr;
  }

  ~ThreadStack() {
    T *old = pop();
    while (old != nullptr) {
      delete old;
      old = pop();
    }
  }
};

template <typename T, typename Deleter = DefaultDeleter<T>>
class HazEpochs : public MinEpoch {
  static_assert(std::is_invocable_v<Deleter, T *>,
                "Deleter must be invocable with a T*");

  using atom_cnt = std::atomic_uint64_t;
  using _Cleaner = Cleaner<T, Deleter>;

 private:
  ThreadStack<_Cleaner> thread_stack;
  std::vector<std::unique_ptr<atom_cnt>> reservations;
  atom_cnt global_epoch;
  size_t num_threads;
  Deleter *deleter;

 public:
  HazEpochs(const size_t num_threads, Deleter *deleter = new Deleter())
      : num_threads(num_threads), deleter(deleter) {
    atom_cnt *cnts = (atom_cnt *)calloc(num_threads, sizeof(atom_cnt));
    for (size_t i = 0; i < num_threads; i++) {
      cnts[i].store(kNoEpoch);
      reservations.emplace_back(std::unique_ptr<atom_cnt>(cnts + i));
    }
    global_epoch.store(0);
    for (auto &reservation : reservations) {
      auto cleaner =
          new _Cleaner(reservation.get(), &global_epoch, this, deleter);
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

  _Cleaner *begin() {
    auto e = thread_stack.pop();
    if (e == nullptr) {
      throw std::runtime_error("Maximum number of threads reached");
    }
    return e;
  }

  void end(_Cleaner *e) { thread_stack.push(e); }
};

template <typename T>
class VersionedNode {
 private:
  std::atomic_uint64_t birth;
  T data;

 public:
  VersionedNode(uint64_t epoch, T &&data) : data(std::move(data)) {
    birth.store(epoch);
  }

  void allocate(uint64_t epoch) { birth.store(epoch); }

  T *get() { return &data; }
};

template <typename T>
class VersionedPtr {
  using _Ptr = VersionedPtr<T>;

 public:
  VersionedPtr(VersionedNode<T> *ptr, uint64_t version) {
    versionPtr.store((((__int128_t)ptr) << 64) | ((__int128_t)version));
  }
  VersionedPtr() { versionPtr.store(0); }
  VersionedPtr(_Ptr &&other) {
    versionPtr.store(other.versionPtr);
    other.versionPtr.store(0);
  }
  /*VersionedPtr<T> &operator=(VersionedPtr<T> &&other) {
    versionPtr.store(other.versionPtr);
    other.versionPtr.store(0);
    return *this;
  }*/

  std::pair<bool, _Ptr> replace(__int128_t old, _Ptr &ptr) {
    return std::make_pair<bool, _Ptr>(
        versionPtr.compare_exchange_strong(old, ptr.versionPtr.load()),
        _Ptr(old));
  }

  bool mark(__int128_t old) {
    if (old & kMarker) {
      return true;
    }
    __int128_t n = old | kMarker;
    return versionPtr.compare_exchange_strong(old, n);
  };

  _Ptr take() { return _Ptr(versionPtr.load()); }

  bool clear(__int128_t old) {
    return versionPtr.compare_exchange_strong(old, 0);
  }

  std::pair<T *, __int128_t> load() const {
    auto v = versionPtr.load() & kHideMarker;
    VersionedNode<T> *data = ((VersionedNode<T> *)((uintptr_t)(v >> 64)));
    if (data == nullptr) {
      return std::make_pair(nullptr, v);
    }
    return std::make_pair(data->get(), v);
  }

  std::pair<VersionedNode<T> *, __int128_t> load_r() const {
    auto v = versionPtr.load() & kHideMarker;
    VersionedNode<T> *data = ((VersionedNode<T> *)((uintptr_t)(v >> 64)));
    return std::make_pair(data, v);
  }
  T *get() const { return load().first; }
  VersionedNode<T> *get_r() const { return load_r().first; }

 private:
  VersionedPtr(__int128_t i) { versionPtr.store(i); }
  // [64-Bit Pointer][1-Bit Marker][63-Bit Version] = 128
  std::atomic<__int128_t> versionPtr{0};
  static const __int128_t kVersionMask = (~(__int128_t)0) >> 63;
  static const __int128_t kMarker = ((__int128_t)1) << 63;
  static const __int128_t kHideMarker = (~((__int128_t)0)) ^ kMarker;
};

template <typename T>
static constexpr VersionedPtr<T> vnull = VersionedPtr<T>();

class VersionedReader {
 public:
  VersionedReader(std::atomic<uint64_t> *global_e) : global_e(global_e) {
    restart();
  }
  bool validate() {
    auto global_epoch = global_e->load();
    return epoch == global_epoch;
  }
  void restart() { epoch = global_e->load(); }

 private:
  std::atomic<uint64_t> *global_e;
  uint64_t epoch;
};

template <typename T, typename Factory>
class VersionPool {
  static_assert(std::is_invocable_r_v<T, Factory>,
                "Factory must return a pointer to a T");
  using _Ptr = VersionedPtr<T>;
  using _T = VersionedNode<T>;

 public:
  VersionPool(std::atomic<uint64_t> *global_e, Factory *factory)
      : global_e(global_e), factory(factory) {}

  _Ptr allocate() {
    uint64_t epoch = global_e->load();
    if (!pool.empty()) {
      Retired ptr = std::move(pool.back());
      pool.pop_back();

      if (ptr.version == epoch) {
        global_e->fetch_add(1);
      }
      return VersionedPtr(ptr.ptr.release(), epoch);
    }
    return VersionedPtr(new VersionedNode(epoch, (*factory)()), epoch);
  }

  void retire(_Ptr *ptr) {
    pool.push_back(Retired(ptr->get_r(), global_e->load()));
  }

 private:
  class Retired {
   public:
    Retired(_T *ptr, uint64_t version)
        : ptr(std::unique_ptr<_T>(ptr)), version(version) {}
    std::unique_ptr<_T> ptr;
    uint64_t version;
  };

  std::atomic<uint64_t> *global_e;
  std::vector<Retired> pool;
  Factory *factory;
};

template <typename T, typename Factory = DefaultFactory<T>>
class HazVersions {
  static_assert(std::is_invocable_r_v<T, Factory>,
                "Factory must return a pointer to a T");
  using _Pool = VersionPool<T, Factory>;

 private:
  ThreadStack<_Pool> thread_stack;
  std::unique_ptr<Factory> factory;

 public:
  std::atomic<uint64_t> global_e;
  HazVersions() : factory(std::make_unique<Factory>()) {}
  HazVersions(std::unique_ptr<Factory> factory) : factory(std::move(factory)) {}
  _Pool *begin() {
    auto e = thread_stack.pop();
    if (e == nullptr) {
      return new VersionPool<T, Factory>(&global_e, factory.get());
    }
    return e;
  }

  void end(_Pool *e) { thread_stack.push(e); }
  VersionedReader read() { return VersionedReader(&global_e); }
};

}  // namespace haz_ptrs