MAIN_COMPILER = g++
COMMONFLAGS = -Wall -Wextra -Werror -Wshadow -Wconversion -Wdouble-promotion -Wpedantic -Wformat=2 -Winline -funsigned-char -pipe -std=c++20
DEBUGFLAGS = -O0 -g
RELEASEFLAGS = -flto -funroll-loops -fno-rtti -march=native -O3 -s

FILES = main.cpp
EXECUTABLE = chip8_cpp.exe

release:
#	cppcheck -q --enable=all --check-level=exhaustive --suppress=missingIncludeSystem --language=c++ --std=c++20 $(FILES)
	$(MAIN_COMPILER) $(COMMONFLAGS) $(RELEASEFLAGS) -o $(EXECUTABLE) $(FILES)
	strip --strip-all -R .comment -R .note $(EXECUTABLE)

debug:
	$(MAIN_COMPILER) $(COMMONFLAGS) $(DEBUGFLAGS) -o $(EXECUTABLE) $(FILES)

clean:
	rm $(EXECUTABLE)