// game_integration.h
#ifndef ORACLE_STEAM_INTEGRATION_H
#define ORACLE_STEAM_INTEGRATION_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

// Protection API
bool OracleSteam_Init(void);
bool OracleSteam_Validate(uint64_t key);
uint64_t OracleSteam_GetNextKey(void);
void OracleSteam_Heartbeat(void);
bool OracleSteam_VerifyIntegrity(uint64_t checksum);
bool OracleSteam_CheckDebugger(void);

// Standard Steam API
bool SteamAPI_Init(void);
void SteamAPI_Shutdown(void);
bool SteamAPI_RestartAppIfNecessary(uint32_t app_id);
void SteamAPI_RunCallbacks(void);

#ifdef __cplusplus
}
#endif

#endif // ORACLE_STEAM_INTEGRATION_H


// game_main.c - Example integration
#include "oracle_steam_integration.h"
#include <stdio.h>
#include <stdlib.h>
#include <time.h>

static uint64_t validation_key = 0;
static uint64_t last_validation_time = 0;

// Obfuscated check - inline assembly to make reversing harder
#ifdef _MSC_VER
__forceinline
#else
__attribute__((always_inline))
#endif
static bool validate_protection(void) {
    // Get current time
    time_t now = time(NULL);
    
    // Validate every 5 seconds
    if (now - last_validation_time < 5) {
        return true;
    }
    
    // Get next validation key
    uint64_t next_key = OracleSteam_GetNextKey();
    
    // Validate with Oracle Steam
    if (!OracleSteam_Validate(next_key)) {
        // Protection failed - trigger crash
        // This is obfuscated to make it hard to patch out
        void* ptr = NULL;
        *(int*)ptr = 0xDEAD;
        return false;
    }
    
    validation_key = next_key;
    last_validation_time = now;
    return true;
}

// Call this in your game's main loop
void game_update(void) {
    // Validate protection (obfuscated)
    if (!validate_protection()) {
        exit(0xDEADC0DE);
    }
    
    // Check for debugger
    if (!OracleSteam_CheckDebugger()) {
        // Debugger detected - crash
        abort();
    }
    
    // Your game logic here
    // ...
    
    // Run Steam callbacks
    SteamAPI_RunCallbacks();
}

int main(int argc, char** argv) {
    printf("Starting game with Oracle Steam...\n");
    
    // Initialize Oracle Steam protection
    if (!OracleSteam_Init()) {
        printf("FATAL: Oracle Steam protection init failed!\n");
        printf("Please ensure Oracle Steam is running.\n");
        return 1;
    }
    
    // Verify DLL integrity
    // In production, calculate actual checksum
    uint64_t dll_checksum = 0xCAFEBABEDEADBEEF;
    if (!OracleSteam_VerifyIntegrity(dll_checksum)) {
        printf("FATAL: DLL integrity check failed!\n");
        return 2;
    }
    
    // Initialize Steam API
    if (!SteamAPI_Init()) {
        printf("FATAL: Steam API init failed!\n");
        return 3;
    }
    
    // Get initial validation key
    validation_key = OracleSteam_GetNextKey();
    last_validation_time = time(NULL);
    
    printf("Oracle Steam initialized successfully!\n");
    
    // Main game loop
    bool running = true;
    while (running) {
        game_update();
        
        // Sleep to control framerate
        // In real game, use proper frame timing
#ifdef _WIN32
        Sleep(16); // ~60 FPS
#else
        usleep(16000);
#endif
    }
    
    // Cleanup
    SteamAPI_Shutdown();
    
    return 0;
}

/*
 * COMPILATION INSTRUCTIONS:
 * 
 * Windows (MSVC):
 *   cl game_main.c /link steam_api64.lib /OUT:game.exe
 * 
 * Windows (MinGW):
 *   gcc game_main.c -L. -lsteam_api64 -o game.exe
 * 
 * Linux:
 *   gcc game_main.c -L. -lsteam_api -ldl -lpthread -o game
 * 
 * The steam_api64.dll/libsteam_api.so should be in the same directory
 * as the game executable.
 */