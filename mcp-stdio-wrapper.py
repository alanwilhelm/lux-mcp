#!/usr/bin/env python3
"""
MCP Protocol Wrapper for Lux Server
This wrapper ensures proper stdio handling for MCP protocol compliance
"""

import sys
import subprocess
import os
import json

def main():
    # Set up environment
    env = os.environ.copy()
    
    # Ensure unbuffered I/O
    env['PYTHONUNBUFFERED'] = '1'
    
    # Path to the actual server
    server_path = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'target/release/lux-mcp')
    
    # Start the server as a subprocess
    proc = subprocess.Popen(
        [server_path],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        env=env,
        bufsize=0  # Unbuffered
    )
    
    # Forward stdin to the subprocess
    try:
        while True:
            line = sys.stdin.readline()
            if not line:
                break
            
            # Write to subprocess
            proc.stdin.write(line.encode())
            proc.stdin.flush()
            
            # Read response
            response = proc.stdout.readline()
            if response:
                sys.stdout.write(response.decode())
                sys.stdout.flush()
                
    except KeyboardInterrupt:
        pass
    finally:
        proc.terminate()
        proc.wait()

if __name__ == '__main__':
    main()