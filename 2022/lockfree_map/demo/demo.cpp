#include <iostream>

#include "lockfree_map.hpp"

int main() {
  auto map = new lockfree_map::HashMap<int, int>();
  map->insert(1, 1);
  std::optional<int> test = map->get(1);
  std::cout << test.has_value() << std::endl;
}