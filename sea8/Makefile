COMPILER = gcc
COMMONFLAGS = -Wall -Wextra -Werror -Wshadow -Wformat=2 -pipe -std=c17
DEBUGFLAGS = -O0 -g3
RELEASEFLAGS = -flto -march=native -O3 -s
LDFLAGS = -lraylib -lopengl32 -lgdi32 -lwinmm

FILES = main.c
EXECUTABLE = sea8.exe

release:
	$(COMPILER) $(COMMONFLAGS) $(RELEASEFLAGS) $(FILES) -o $(EXECUTABLE) $(LDFLAGS)
	strip --strip-all -R .comment -R .note $(EXECUTABLE)

debug:
	$(COMPILER) $(COMMONFLAGS) $(DEBUGFLAGS) $(FILES) -o $(EXECUTABLE) $(LDFLAGS)