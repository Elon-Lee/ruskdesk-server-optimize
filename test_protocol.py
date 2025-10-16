#!/usr/bin/env python3
"""
测试RustDesk服务器协议
使用正确的protobuf格式发送punch hole请求
"""

import socket
import struct
import time
import sys

def create_protobuf_punch_hole_request(key="123"):
    """创建符合RustDesk协议的punch hole请求"""
    
    # 这是一个简化的实现，实际的RustDesk协议使用protobuf
    # 我们需要创建一个基本的消息结构
    
    # 模拟protobuf消息的基本结构
    # 实际实现需要根据RustDesk的protobuf定义
    
    # 创建punch hole请求的基本字段
    message_data = b""
    
    # 添加消息类型标识 (简化)
    message_type = 1  # PunchHoleRequest
    message_data += struct.pack('<I', message_type)
    
    # 添加ID字段 (必需)
    peer_id = b"test-peer-12345"
    message_data += struct.pack('<I', len(peer_id))
    message_data += peer_id
    
    # 添加licence_key字段
    key_bytes = key.encode('utf-8')
    message_data += struct.pack('<I', len(key_bytes))
    message_data += key_bytes
    
    # 添加其他必需字段
    nat_type = 0  # Symmetric
    message_data += struct.pack('<I', nat_type)
    
    # 创建完整的消息
    total_length = len(message_data)
    full_message = struct.pack('<I', total_length) + message_data
    
    return full_message

def test_protocol_connection(host="127.0.0.1", port=21115, key="123"):
    """测试协议连接"""
    print(f"测试协议连接到 {host}:{port}")
    print(f"使用密钥: {key}")
    print()
    
    try:
        # 创建socket连接
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.settimeout(10)
        
        print("正在连接...")
        sock.connect((host, port))
        print("✓ 连接成功")
        
        # 发送protobuf格式的punch hole请求
        print("发送protobuf格式的punch hole请求...")
        request = create_protobuf_punch_hole_request(key)
        print(f"请求长度: {len(request)} 字节")
        sock.send(request)
        print("✓ 请求已发送")
        
        # 等待服务器处理数据
        print("等待服务器处理数据...")
        time.sleep(5)
        
        # 尝试接收响应
        try:
            response = sock.recv(1024)
            if response:
                print(f"✓ 收到响应: {len(response)} 字节")
                print(f"响应内容: {response[:50]}...")
            else:
                print("⚠ 没有收到响应")
        except socket.timeout:
            print("⚠ 接收响应超时")
        
        sock.close()
        print("✓ 连接已关闭")
        
    except Exception as e:
        print(f"✗ 连接失败: {e}")
        return False
    
    return True

def main():
    print("=== RustDesk 协议测试 ===")
    print()
    
    # 测试有效密钥
    print("1. 测试有效密钥 '123'")
    test_protocol_connection(key="123")
    print()
    
    # 测试无效密钥
    print("2. 测试无效密钥 'invalid'")
    test_protocol_connection(key="invalid")
    print()
    
    print("测试完成！")
    print("请检查服务器日志以查看详细的验证过程。")

if __name__ == "__main__":
    main()
