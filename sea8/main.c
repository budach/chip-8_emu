#include "raylib.h"
#include <stdio.h>
#include <time.h>

int main()
{
    struct timespec time_start, time_stop;
    struct timespec sleep_duration = { 0 };

    const int screenWidth = 800;
    const int screenHeight = 450;

    InitWindow(screenWidth, screenHeight, "raylib [core] example - basic window");

    while (!WindowShouldClose()) {
        clock_gettime(CLOCK_MONOTONIC, &time_start);

        BeginDrawing();
        ClearBackground(BLACK);
        DrawRectangle(100, 50, 10, 10, WHITE);
        EndDrawing();

        clock_gettime(CLOCK_MONOTONIC, &time_stop);

        long diff_nsec = time_stop.tv_nsec - time_start.tv_nsec; // assumes diff < 1 second
        if (diff_nsec < 16666666) {
            sleep_duration.tv_nsec = 16666666 - diff_nsec;
            nanosleep(&sleep_duration, NULL);
        }
    }

    CloseWindow();

    return 0;
}