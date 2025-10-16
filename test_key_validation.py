#!/usr/bin/env python3
"""
测试RustDesk服务器密钥验证
模拟客户端发送punch hole请求
"""

import socket
import struct
import time
import sys

def create_punch_hole_request(key="123"):
    """创建punch hole请求消息"""
    # 这是一个简化的测试，实际协议更复杂
    # 我们主要测试服务器是否能正确处理密钥验证
    
    # 模拟punch hole请求的基本结构
    # 实际实现需要根据RustDesk协议规范
    message = b"PUNCH_HOLE_REQUEST"
    key_bytes = key.encode('utf-8')
    
    # 简单的消息格式：长度 + 消息类型 + 密钥长度 + 密钥
    msg_len = len(message) + len(key_bytes) + 8
    data = struct.pack('<I', msg_len)  # 消息长度
    data += message
    data += struct.pack('<I', len(key_bytes))  # 密钥长度
    data += key_bytes
    
    return data

def test_server_connection(host="127.0.0.1", port=21115, key="123"):
    """测试服务器连接和密钥验证"""
    print(f"测试连接到 {host}:{port}")
    print(f"使用密钥: {key}")
    print()
    
    try:
        # 创建socket连接
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.settimeout(10)
        
        print("正在连接...")
        sock.connect((host, port))
        print("✓ 连接成功")
        
        # 发送punch hole请求
        print("发送punch hole请求...")
        request = create_punch_hole_request(key)
        sock.send(request)
        print("✓ 请求已发送")
        
        # 等待响应
        print("等待服务器响应...")
        time.sleep(2)
        
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
    print("=== RustDesk 服务器密钥验证测试 ===")
    print()
    
    # 测试有效密钥
    print("1. 测试有效密钥 '123'")
    test_server_connection(key="123")
    print()
    
    # 测试无效密钥
    print("2. 测试无效密钥 'invalid'")
    test_server_connection(key="invalid")
    print()
    
    # 测试空密钥
    print("3. 测试空密钥")
    test_server_connection(key="")
    print()
    
    print("测试完成！")
    print("请检查服务器日志以查看详细的验证过程。")

if __name__ == "__main__":
    main()
