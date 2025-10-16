#!/usr/bin/env python3
"""
RustDesk Protobuf Protocol Test Client
测试RustDesk服务器的protobuf协议
"""

import socket
import sys
import time
import struct
from libs.hbb_common.protos.rendezvous_pb2 import RendezvousMessage, RegisterPeer, PunchHoleRequest, RegisterPk

def create_punch_hole_message():
    """创建一个PunchHoleRequest消息"""
    punch_hole = PunchHoleRequest()
    punch_hole.id = "129473391"
    punch_hole.nat_type = 0
    punch_hole.udp_port = 12345
    punch_hole.licence_key = "123"
    
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

def send_punch_hole_request(host, port):
    """发送PunchHoleRequest消息"""
    try:
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.settimeout(5)
        
        print(f"连接到 {host}:{port}...")
        sock.connect((host, port))
        print("连接成功!")
        
        # 发送PunchHoleRequest
        punch_data = create_punch_hole_message()
        punch_length_header = encode_length(len(punch_data))
        
        print("发送PunchHoleRequest...")
        sock.send(punch_length_header)
        sock.send(punch_data)
        
        # 等待响应
        print("等待PunchHoleResponse...")
        sock.settimeout(2.0)
        response_data = b""
        try:
            while True:
                chunk = sock.recv(1024)
                if not chunk:
                    break
                response_data += chunk
        except socket.timeout:
            pass
        
        print(f"收到PunchHoleResponse: {len(response_data)} 字节")
        return len(response_data) > 0
        
    except Exception as e:
        print(f"PunchHoleRequest失败: {e}")
        return False
    finally:
        sock.close()

def send_register_pk_request(host, port):
    """发送RegisterPk消息"""
    try:
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.settimeout(5)
        
        print(f"连接到 {host}:{port}...")
        sock.connect((host, port))
        print("连接成功!")
        
        # 发送RegisterPk消息
        protobuf_data = create_register_pk_message()
        print(f"RegisterPk消息大小: {len(protobuf_data)} 字节")
        
        length_header = encode_length(len(protobuf_data))
        print(f"长度头: {length_header.hex()}")
        
        sock.send(length_header)
        sock.send(protobuf_data)
        print("RegisterPk消息已发送!")
        
        # 等待响应
        print("等待RegisterPkResponse...")
        sock.settimeout(3.0)
        response_data = b""
        try:
            while True:
                chunk = sock.recv(1024)
                if not chunk:
                    break
                response_data += chunk
                print(f"收到数据块: {len(chunk)} 字节, 总数据: {len(response_data)} 字节")
        except socket.timeout:
            print("接收超时")
        
        print(f"总共收到响应数据: {len(response_data)} 字节")
        print(f"响应数据: {response_data.hex()}")
        
        # 解析响应
        if len(response_data) >= 1:
            first_byte = response_data[0]
            header_len = (first_byte & 0x3) + 1
            print(f"第一个字节: 0x{first_byte:02x}, 头部长度: {header_len}")
            
            if len(response_data) >= header_len:
                n = first_byte
                if header_len > 1:
                    n |= response_data[1] << 8
                if header_len > 2:
                    n |= response_data[2] << 16
                if header_len > 3:
                    n |= response_data[3] << 24
                response_length = n >> 2
                
                print(f"解析的响应长度: {response_length} 字节")
                
                if response_length > 0 and response_length < 10000:
                    start_pos = header_len
                    end_pos = start_pos + response_length
                    
                    if end_pos <= len(response_data):
                        response_payload = response_data[start_pos:end_pos]
                        print(f"提取的响应载荷: {len(response_payload)} 字节")
                        
                        try:
                            response_msg = RendezvousMessage()
                            response_msg.ParseFromString(response_payload)
                            print("响应解析成功!")
                            print(f"响应类型: {response_msg.WhichOneof('union')}")
                            
                            if response_msg.HasField('register_pk_response'):
                                print(f"RegisterPkResponse: {response_msg.register_pk_response}")
                                return True
                        except Exception as e:
                            print(f"响应解析失败: {e}")
                            print(f"原始响应载荷: {response_payload.hex()}")
        
        return False
        
    except Exception as e:
        print(f"RegisterPkRequest失败: {e}")
        return False
    finally:
        sock.close()

def send_protobuf_message(host, port):
    """发送protobuf消息到RustDesk服务器"""
    print("步骤1: 发送PunchHoleRequest...")
    if send_punch_hole_request(host, port):
        print("PunchHoleRequest成功!")
        
        print("\n步骤2: 发送RegisterPk...")
        if send_register_pk_request(host, port):
            print("RegisterPk成功!")
        else:
            print("RegisterPk失败!")
    else:
        print("PunchHoleRequest失败!")
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
