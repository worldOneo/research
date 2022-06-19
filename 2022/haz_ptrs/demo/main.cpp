// Example program
#include <iostream>
#include <string>

#include "haz_ptr.hpp"

// void test(std::string *a) { std::cout << "test: " << a << std::endl; }
static int deletes = 0;
struct MyDeleter {
  void operator()(const std::string *a) {
    // std::cout << "deleter: " << a << std::endl;
    deletes++;
    delete a;
  }
};

int main() {
  auto epocher = new HazEpochs<std::string, MyDeleter>(64);
  auto cleaner = epocher->begin();
  cleaner->enter();
  for (int i = 0; i < 10000; i++) {
    auto str = new std::string("hello");
    cleaner->retire(str);
  }
  std::cout << "deletes: " << deletes << std::endl;
  cleaner->exit();
  for (int i = 0; i < 10000; i++) {
    auto str = new std::string("hello");
    cleaner->retire(str);
  }
  std::cout << "deletes: " << deletes << std::endl;
  return 0;
}
