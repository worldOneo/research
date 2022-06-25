#include <optional>

#include "haz_ptr.hpp"

namespace lockfree_map {
template <typename T>
using _At = std::atomic<T>;
template <typename First, typename Second>
struct DoublePointer {
  using _fsp = std::pair<First, Second>;
  using _Sptr = haz_ptrs::VersionedPtr<Second>;
  _At<__int128_t> ptr;
  DoublePointer() { ptr.store(0); }

  _fsp load() {
    __int128_t val = ptr.load();
    return std::pair((First)(val >> 64), (Second)(val));
  }

  std::pair<bool, _fsp> store(First f, Second s) {
    __int128_t val = ptr.load();
    auto pair = std::pair((First)(val >> 64), (Second)(val));
    auto newVal = (((__int128_t)f) << 64) | ((__int128_t)s);
    bool res = ptr.compare_exchange_weak(val, newVal);
    return std::pair(res, pair);
  }

  void rstore(First f, Second s) {
    auto newVal = (((__int128_t)f) << 64) + ((__int128_t)s);
    ptr.store(newVal);
  }
};

static const size_t kBucketSize = 8;

template <typename Value>
struct HashBucket {
  std::atomic_size_t kvs[kBucketSize];
  Value vs[kBucketSize];
  haz_ptrs::VersionedPtr<HashBucket<Value>> next;
};

template <typename Key, typename Value>
class HashMap {
  static_assert(std::is_trivially_copyable_v<Value>,
                "Value must be trivially copyable");
  static_assert(std::is_trivially_copyable_v<Key>,
                "Key must be trivially copyable");

 private:
  using _KV = std::pair<Key, Value>;
  using _Bucket = HashBucket<haz_ptrs::VersionedPtr<_KV>>;
  _At<_Bucket *> elements;
  // _At<HashMap<Key, Value> *> next;
  std::atomic_uint64_t count;
  haz_ptrs::HazVersions<_Bucket> bucket_pool;
  haz_ptrs::HazVersions<_KV> kv_pool;
  std::size_t max_size;
  std::size_t size;
  size_t size_minus_one;
  size_t totem_a;
  size_t totem_b;
  std::pair<size_t, _Bucket *> get_bucket(Key key) {
    size_t hash = std::hash<Key>{}(key);
    _Bucket *buckets = elements.load();
    size_t bucket = hash & size_minus_one;
    return std::pair(hash, &buckets[bucket]);
  }

  static const size_t kTotemA = ~((size_t)0);
  static const size_t kTotemB = kTotemA - 1;

 public:
  static const size_t kInitialSize = 64;
  HashMap(size_t size = kInitialSize, size_t totem_a = kTotemA,
          size_t totem_b = kTotemB)
      : size(size),
        size_minus_one(size - 1),
        totem_a(totem_a),
        totem_b(totem_b) {
    _Bucket *buckets = new _Bucket[kInitialSize];
    elements.store(buckets);
  }

  bool insert(Key key, Value value) {
    if (count.fetch_add(1) > max_size) {
      // TODO: Resize
      return false;
    }
  retry:
    auto [hash, bucket] = get_bucket(key);
    auto pool = kv_pool.begin();
    auto kv = pool->allocate();
    auto val = kv.get();
    val->first = key;
    val->second = value;
    do {
      for (int i = 0; i < kBucketSize; i++) {
        size_t refhash = bucket->kvs[i].load();
        if (refhash != hash && refhash != 0) {
          continue;
        }
        if (refhash == 0 &&
            !bucket->kvs[i].compare_exchange_strong(refhash, hash)) {
          goto retry;
        }
        auto [current, old] = bucket->vs[i].load();
        if (current != nullptr && current->first != key) {
          continue;
        }
        auto [replaced, swaped] = bucket->vs[i].replace(old, kv);
        if (!replaced) {
          goto retry;
        }
        if (current != nullptr) {
          pool->retire(&swaped);
        }

        kv_pool.end(pool);
        return true;
      }
      bucket = bucket->next.get();
    } while (bucket != nullptr);
    // TODO: Expand bucket chain
    throw std::runtime_error("Reached end of bucket");
    goto retry;
    return true;
  }

  std::optional<Value> get(const Key key) {
  retry:
    auto reader = bucket_pool.read();
    auto [hash, bucket] = get_bucket(key);
    do {
      for (int i = 0; i < kBucketSize; i++) {
        auto refhash = bucket->kvs[i].load();
        if (hash == refhash) {
          auto ref = bucket->vs[i].get();
          if (key != ref->first) continue;
          if (!reader.validate()) goto retry;
          return std::optional(ref->second);
        }
      }
      bucket = bucket->next.get();
    } while (bucket != nullptr);
    return std::nullopt;
  }
};
}  // namespace lockfree_map
