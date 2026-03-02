#!/usr/bin/env python3
import json
import base64
import sys
import time

def emit_osc_context(data):
    # Serialize dict to JSON
    json_str = json.dumps(data)
    # Base64 encode it completely to avoid any unprintable characters or early terminations
    b64_str = base64.b64encode(json_str.encode('utf-8')).decode('utf-8')
    # Generate the OSC sequence specified by the TOS Ecosystem Architecture
    sys.stdout.write(f"\033]9004;{b64_str}\007\n")
    sys.stdout.flush()

if __name__ == "__main__":
    print("Initializing TOS Alpha-2 JSON Context Exporter...")
    time.sleep(0.5)

    sample_context = {
        "type": "ApplicationModel",
        "name": "ImageEditor",
        "state": "active",
        "tools": ["Crop", "Resize", "Filters", "Adjust Colors"],
        "active_file": "/workspace/photos/img_001.png",
        "metadata": {
            "resolution": "1920x1080",
            "layers": 3
        }
    }

    print("Emitting semantic context into terminal stream...")
    emit_osc_context(sample_context)
    
    print("Context emitted! The Command Hub's left chip column should now display this data.")
    print("Exiting.")
