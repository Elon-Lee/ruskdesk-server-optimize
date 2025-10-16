#!/usr/bin/env python3
"""
RustDesk Protobuf Protocol Test Client
测试RustDesk服务器的protobuf协议
"""

import socket
import sys
import time
import struct
from libs.hbb_common.protos.rendezvous_pb2 import RendezvousMessage, RegisterPeer

def create_punch_hole_message():
    """创建一个PunchHoleRequest消息"""
    punch_hole = PunchHoleRequest()
    punch_hole.id = "129473391"
    punch_hole.nat_type = 0
    punch_hole.local_port = 12345
    punch_hole.license_key = "123"
    punch_hole.server_key = b"Wt+jdXMqZIDM9jwfF7OdrE8ho2XjO48YdmcDPgtaaH0="
    
    rendezvous_msg = RendezvousMessage()
    rendezvous_msg.punch_hole_request.CopyFrom(punch_hole)
    
    return rendezvous_msg.SerializeToString()

def create_register_pk_message():
    """创建一个RegisterPk消息"""
    register_pk = RegisterPk()
    register_pk.id = "129473391"
    register_pk.uuid = b"123"
    register_pk.pk = b"test_key_12345"
    register_pk.custom_key = "123"
    
    rendezvous_msg = RendezvousMessage()
    rendezvous_msg.register_pk.CopyFrom(register_pk)
    
    return rendezvous_msg.SerializeToString()

def encode_length(length):
    """使用RustDesk的自定义长度编码格式"""
    if length <= 0x3F:  # 63
        return struct.pack('<B', (length << 2))
    elif length <= 0x3FFF:  # 16383
        return struct.pack('<H', (length << 2) | 0x1)
    elif length <= 0x3FFFFF:  # 16777215
        h = (length << 2) | 0x2
        return struct.pack('<HB', h & 0xFFFF, h >> 16)
    elif length <= 0x3FFFFFFF:  # 1073741823
        return struct.pack('<I', (length << 2) | 0x3)
    else:
        raise ValueError("Message too large")

def send_protobuf_message(host, port):
    """发送protobuf消息到RustDesk服务器"""
    try:
        # 创建TCP连接
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.settimeout(10)
        
        print(f"连接到 {host}:{port}...")
        sock.connect((host, port))
        print("连接成功!")
        
        # 创建protobuf消息
        protobuf_data = create_register_peer_message()
        print(f"Protobuf消息大小: {len(protobuf_data)} 字节")
        
        # 使用RustDesk的自定义长度编码
        length_header = encode_length(len(protobuf_data))
        print(f"长度头: {length_header.hex()}")
        
        # 发送长度头
        sock.send(length_header)
        
        # 发送protobuf数据
        sock.send(protobuf_data)
        print("消息已发送!")
        
        # 等待响应
        print("等待响应...")
        # 先读取长度头
        response_length_data = sock.recv(4)
        print(f"收到长度头数据: {len(response_length_data)} 字节, 内容: {response_length_data.hex()}")
        
        if len(response_length_data) >= 1:
            # 解析长度头
            first_byte = response_length_data[0]
            header_len = (first_byte & 0x3) + 1
            print(f"第一个字节: 0x{first_byte:02x}, 头部长度: {header_len}")
            
            if len(response_length_data) < header_len:
                # 需要读取更多数据
                remaining = header_len - len(response_length_data)
                print(f"需要读取更多数据: {remaining} 字节")
                response_length_data += sock.recv(remaining)
                print(f"完整长度头: {response_length_data.hex()}")
            
            # 解析长度
            if header_len == 1:
                response_length = (first_byte >> 2) & 0x3F
            elif header_len == 2:
                response_length = struct.unpack('<H', response_length_data[:2])[0] >> 2
            elif header_len == 3:
                h = struct.unpack('<HB', response_length_data[:3])
                response_length = ((h[0] | (h[1] << 16)) >> 2) & 0x3FFFFF
            else:  # header_len == 4
                response_length = struct.unpack('<I', response_length_data[:4])[0] >> 2
            
            print(f"解析的响应长度: {response_length} 字节")
            
            if response_length > 0 and response_length < 10000:  # 合理的长度范围
                # 读取响应数据
                response_data = sock.recv(response_length)
                print(f"收到响应: {len(response_data)} 字节")
                
                # 尝试解析响应
                try:
                    response_msg = RendezvousMessage()
                    response_msg.ParseFromString(response_data)
                    print("响应解析成功!")
                    print(f"响应类型: {response_msg.WhichOneof('union')}")
                    if response_msg.HasField('register_pk'):
                        print(f"RegisterPk响应: {response_msg.register_pk}")
                    elif response_msg.HasField('register_pk_response'):
                        print(f"RegisterPkResponse: {response_msg.register_pk_response}")
                except Exception as e:
                    print(f"响应解析失败: {e}")
                    print(f"原始响应数据: {response_data.hex()}")
            else:
                print(f"响应长度异常: {response_length}")
        else:
            print("未收到有效的响应长度")
            
    except Exception as e:
        print(f"连接失败: {e}")
    finally:
        sock.close()
        print("连接已关闭")

def main():
    if len(sys.argv) != 3:
        print("用法: python3 test_protobuf_client.py <host> <port>")
        print("示例: python3 test_protobuf_client.py 127.0.0.1 21116")
        sys.exit(1)
    
    host = sys.argv[1]
    port = int(sys.argv[2])
    
    print(f"测试RustDesk服务器 {host}:{port}")
    print("=" * 50)
    
    send_protobuf_message(host, port)

if __name__ == "__main__":
    main()
