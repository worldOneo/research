#include <benchmark/benchmark.h>

#include <string>

#include "haz_ptr.hpp"

static void BM_HazPtr(benchmark::State &state) {
  static HazEpochs<int> *epocher;
  static const int kNumItems = 100'000;
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

static void BM_Instant(benchmark::State &state) {
  static const int kNumItems = 100'000;
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

BENCHMARK(BM_HazPtr)->ThreadRange(1, 6);
BENCHMARK(BM_Instant)->ThreadRange(1, 6);

BENCHMARK_MAIN();