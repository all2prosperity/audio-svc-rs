import asyncio
import base64
import json
import websockets
import uuid
import wave

async def test_audio_pipeline():
    # 连接WebSocket服务器
    uri = "ws://localhost:3000/api/stream"
    async with websockets.connect(uri) as websocket:
        # 生成会话ID
        session_id = str(uuid.uuid4())
        
        # 发送会话开始消息
        start_session = {
            "type": "start_session",
            "payload": {
                "session_id": session_id,
                "input_format": "pcm",
                "output_format": "pcm", 
                "sample_rate": 24000,
                "output_sample_rate": 24000,
                "round": 0
            }
        }
        await websocket.send(json.dumps(start_session))
        
        # 等待会话开始确认
        response = await websocket.recv()
        resp_data = json.loads(response)
        if resp_data["type"] != "session_started":
            print("Failed to start session")
            return
            
        # 读取音频文件
        with wave.open("test.wav", "rb") as wav_file:
            chunk_size = 48000  # 每个块2秒的音频
            while True:
                chunk = wav_file.readframes(chunk_size)
                if not chunk:
                    break
                    
                # 发送音频数据块
                audio_msg = {
                    "type": "audio_input_chunk",
                    "payload": base64.b64encode(chunk).decode()
                }
                await websocket.send(json.dumps(audio_msg))
                
        # 发送音频结束信号
        await websocket.send(json.dumps({"type": "audio_input_finish"}))
        
        # 接收处理结果
        while True:
            try:
                response = await websocket.recv()
                resp_data = json.loads(response)
                
                if resp_data["type"] == "audio_output_chunk":
                    # 处理返回的音频数据
                    audio_data = base64.b64decode(resp_data["payload"])
                    # 这里可以将音频数据写入文件或做其他处理
                    
                elif resp_data["type"] == "audio_output_finished":
                    print("Audio processing completed")
                    break
                    
            except websockets.exceptions.ConnectionClosed:
                print("Connection closed")
                break

if __name__ == "__main__":
    asyncio.run(test_audio_pipeline())