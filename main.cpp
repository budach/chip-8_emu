#include <iostream>
#include <vector>

int main(int argc, const char* argv[]) {
  if (argc != 2) {
    std::cerr << "Usage: " << argv[0] << " <romfile>" << std::endl;
    return 1;
  }

  std::cout << "hello world" << std::endl;
  return 0;
}