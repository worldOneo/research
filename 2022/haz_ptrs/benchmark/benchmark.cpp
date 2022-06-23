#include <benchmark/benchmark.h>

#include <string>

#include "haz_ptr.hpp"

using namespace haz_ptrs;

static const int kNumItems = 100'000;
static void BM_EBR(benchmark::State &state) {
  static HazEpochs<int> *epocher;
  if (state.thread_index() == 0) {
    epocher = new HazEpochs<int>(64);
  }
  int i = 0;
  for (auto _ : state) {
    auto cleaner = epocher->begin();
    state.PauseTiming();
    int **ints = (int **)calloc(kNumItems, sizeof(int *));
    for (int j = 0; j < kNumItems; j++) {
      ints[j] = new int(j);
    }
    state.ResumeTiming();
    for (int i = 0; i < kNumItems; i++) {
      cleaner->retire(ints[i]);
    }
    i++;
    delete ints;
    epocher->end(cleaner);
  }
  state.SetItemsProcessed(kNumItems * i * state.threads());
}

static void BM_VBR(benchmark::State &state) {
  static HazVersions<int> *versions;
  if (state.thread_index() == 0) {
    versions = new HazVersions<int>();
  }
  int i = 0;
  for (auto _ : state) {
    auto allocator = versions->begin();
    state.PauseTiming();
    VersionedPtr<int> *ints =
        (VersionedPtr<int> *)calloc(kNumItems, sizeof(VersionedPtr<int>));

    for (int i = 0; i < kNumItems; i++) {
      auto val = allocator->allocate();
      ints[i] = std::move(val);
    }
    state.ResumeTiming();
    for (int i = 0; i < kNumItems; i++) {
      allocator->retire(std::move(ints[i]));
    }
    delete ints;
    versions->end(allocator);
    i++;
  }
  state.SetItemsProcessed(kNumItems * i * state.threads());
  if (state.thread_index() == 0) {
    delete versions;
  }
}

static void BM_Instant(benchmark::State &state) {
  int i = 0;
  for (auto _ : state) {
    state.PauseTiming();
    int **ints = (int **)calloc(kNumItems, sizeof(int *));
    for (int i = 0; i < kNumItems; i++) {
      ints[i] = new int(i);
    }
    state.ResumeTiming();
    for (int i = 0; i < kNumItems; i++) {
      delete ints[i];
    }
    i++;
    delete ints;
  }
  state.SetItemsProcessed(kNumItems * i * state.threads());
}

BENCHMARK(BM_EBR)->ThreadRange(1, 6);
BENCHMARK(BM_VBR)->ThreadRange(1, 6);
BENCHMARK(BM_Instant)->ThreadRange(1, 6);

BENCHMARK_MAIN();