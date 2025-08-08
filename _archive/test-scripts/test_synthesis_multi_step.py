#!/usr/bin/env python3
"""Test synthesis evolution through multiple biased_reasoning steps"""

import json
import subprocess
import time
import re
import psycopg2

def send_mcp_message(proc, message):
    """Send a message to the MCP server"""
    proc.stdin.write((json.dumps(message) + '\n').encode())
    proc.stdin.flush()
    time.sleep(0.5)

def read_response(proc, timeout=5):
    """Read response from MCP server"""
    import select
    responses = []
    
    # Use select to read available data with timeout
    start_time = time.time()
    while time.time() - start_time < timeout:
        ready, _, _ = select.select([proc.stdout], [], [], 0.1)
        if ready:
            line = proc.stdout.readline().decode().strip()
            if line:
                try:
                    responses.append(json.loads(line))
                except json.JSONDecodeError:
                    pass
        else:
            # No more data available
            if responses:
                break
    
    return responses

def extract_session_id(content):
    """Extract session ID from content"""
    match = re.search(r'Session ID: (bias_[a-f0-9]+)', content)
    return match.group(1) if match else None

def check_database(session_id):
    """Check database for synthesis states"""
    conn = psycopg2.connect(
        host="localhost",
        database="lux_mcp",
        user="lux_user",
        password="lux_password"
    )
    cur = conn.cursor()
    
    # Check synthesis states
    cur.execute("""
        SELECT 
            s.session_external_id,
            ss.version,
            LEFT(ss.current_understanding, 80) as understanding,
            ss.confidence_score,
            ss.clarity_score,
            ss.step_number
        FROM sessions s
        JOIN synthesis_states ss ON s.id = ss.session_id
        WHERE s.session_external_id = %s
        ORDER BY ss.version;
    """, (session_id,))
    
    print(f"\n📊 Synthesis states for session {session_id}:")
    print("Version | Step | Confidence | Clarity | Understanding")
    print("-" * 70)
    for row in cur.fetchall():
        print(f"{row[1]:7} | {row[5]:4} | {row[3]:10.2f} | {row[4]:7.2f} | {row[2][:50]}...")
    
    # Check insights
    cur.execute("""
        SELECT 
            LEFT(i.insight_text, 80) as insight,
            i.confidence,
            i.source_step
        FROM insights i
        JOIN synthesis_states ss ON i.synthesis_state_id = ss.id
        JOIN sessions s ON ss.session_id = s.id
        WHERE s.session_external_id = %s
        ORDER BY i.created_at;
    """, (session_id,))
    
    insights = cur.fetchall()
    if insights:
        print(f"\n💡 Insights collected ({len(insights)} total):")
        for i, (insight, conf, step) in enumerate(insights[:5]):  # Show first 5
            print(f"   {i+1}. [{conf:.2f}] (Step {step}) {insight[:60]}...")
    
    conn.close()

def main():
    print("🧪 Testing synthesis evolution through multiple steps...\n")
    
    # Start MCP server
    proc = subprocess.Popen(
        ['./target/release/lux-mcp'],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        env={
            "RUST_LOG": "info",
            "DATABASE_URL": "postgres://lux_user:lux_password@localhost/lux_mcp",
            "OPENAI_API_KEY": os.environ.get("OPENAI_API_KEY", ""),
            "OPENROUTER_API_KEY": os.environ.get("OPENROUTER_API_KEY", "")
        }
    )
    
    # Initialize
    print("📝 Step 1: Initialize MCP server...")
    send_mcp_message(proc, {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "1.0"}
        }
    })
    
    send_mcp_message(proc, {"jsonrpc": "2.0", "method": "notifications/initialized"})
    responses = read_response(proc)
    print(f"✅ Initialized (got {len(responses)} responses)")
    
    # Step 1: Initial query
    print("\n📝 Step 2: Initial Query...")
    send_mcp_message(proc, {
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "biased_reasoning",
            "arguments": {
                "query": "Should a 5-person startup use microservices architecture?",
                "max_analysis_rounds": 3
            }
        }
    })
    
    responses = read_response(proc, timeout=10)
    session_id = None
    
    for resp in responses:
        if resp.get('id') == 2:
            content = resp.get('result', {}).get('content', [{}])[0].get('text', '')
            print(f"✅ Got Query response ({len(content)} chars)")
            session_id = extract_session_id(content)
            if session_id:
                print(f"🔑 Session ID: {session_id}")
            break
    
    if not session_id:
        print("❌ No session ID found")
        proc.kill()
        return
    
    # Continue with reasoning steps
    for step_num in range(3, 6):
        print(f"\n📝 Step {step_num-1}: Continue reasoning...")
        send_mcp_message(proc, {
            "jsonrpc": "2.0",
            "id": step_num,
            "method": "tools/call",
            "params": {
                "name": "biased_reasoning",
                "arguments": {
                    "query": "Should a 5-person startup use microservices architecture?",
                    "session_id": session_id,
                    "max_analysis_rounds": 3
                }
            }
        })
        
        responses = read_response(proc, timeout=30)  # Give more time for reasoning
        for resp in responses:
            if resp.get('id') == step_num:
                content = resp.get('result', {}).get('content', [{}])[0].get('text', '')
                # Extract step type from content
                if "Reasoning Step" in content:
                    print(f"✅ Got Reasoning response")
                elif "Bias Analysis" in content:
                    print(f"✅ Got Bias Analysis response")
                elif "Final Synthesis" in content:
                    print(f"✅ Got Final Synthesis response")
                    break
                else:
                    print(f"✅ Got response ({len(content)} chars)")
        
        # Check database after each step
        check_database(session_id)
        time.sleep(1)  # Brief pause between steps
    
    # Final database check
    print("\n🎯 Final synthesis state:")
    check_database(session_id)
    
    # Clean up
    proc.kill()
    print("\n✅ Test completed!")

if __name__ == "__main__":
    import os
    main()