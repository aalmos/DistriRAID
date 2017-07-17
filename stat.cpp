#include <sys/stat.h>
#include <iostream>

int main(void) {
  struct stat buf;
  stat(".", &buf);
  std::cout << buf.st_blksize << std::endl;   
  return 0;
}
