import asyncio
import websockets

async def test_ws():
    uri = "ws://127.0.0.1:7001"
    print(f"Connecting to {uri}...")
    try:
        async with websockets.connect(uri) as websocket:
            print("✅ Connected!")
            await websocket.send("get_state_delta:0")
            response = await websocket.recv()
            print(f"Received Delta: {response[:100]}...")
    except Exception as e:
        print(f"❌ Error: {e}")

if __name__ == "__main__":
    asyncio.run(test_ws())
