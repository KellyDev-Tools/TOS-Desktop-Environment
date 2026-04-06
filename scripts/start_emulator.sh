#!/bin/bash
# Helper script to launch TOS Android Emulators

EMULATOR_BIN="$HOME/android-sdk/emulator/emulator"

if [ "$1" == "tablet" ]; then
    echo "[TOS] Launching Pixel Tablet Emulator..."
    $EMULATOR_BIN -avd TOS_Pixel_Tablet -no-boot-anim &
elif [ "$1" == "pixel" ]; then
    echo "[TOS] Launching Pixel 7 Emulator..."
    $EMULATOR_BIN -avd TOS_Pixel_7 -no-boot-anim &
else
    echo "Usage: ./scripts/start_emulator.sh [pixel|tablet]"
    echo "Available instances:"
    $EMULATOR_BIN -list-avds
fi
