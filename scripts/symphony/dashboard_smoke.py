"""Exercise the actual proxy, dashboard assets, read-only API and WebSocket tunnel."""
import base64
import json
import re
import socket
import urllib.parse
import urllib.request

base = 'http://127.0.0.1:4317'
page = urllib.request.urlopen(base, timeout=5).read().decode()
assert 'Operations Dashboard' in page
assert page.count('src="/operator-summary.js"') == 1
assert '/operator-summary.css' in page
script = urllib.request.urlopen(base + '/operator-summary.js', timeout=5).read().decode()
assert '300000' in script and 'textContent' in script and 'innerHTML' not in script
assert '#operator-summary' in urllib.request.urlopen(base + '/operator-summary.css', timeout=5).read().decode()
assert 'LiveSocket' in page
assert urllib.request.urlopen(base + '/vendor/phoenix/phoenix.js', timeout=5).status == 200
with urllib.request.urlopen(base + '/operator-summary.json', timeout=5) as response:
    assert response.headers['Cache-Control'] == 'no-store'
    summary = json.load(response)
assert summary['state'] == 'idle' and summary['refresh_seconds'] == 300, summary
assert json.load(urllib.request.urlopen(base + '/api/v1/state', timeout=5))['counts']['running'] == 0
csrf = re.search(r'name="csrf-token" content="([^"]+)"', page).group(1)
path = '/live/websocket?' + urllib.parse.urlencode({'_csrf_token': csrf, 'vsn': '2.0.0'})
key = base64.b64encode(b'0123456789abcdef').decode()
with socket.create_connection(('127.0.0.1', 4317), timeout=5) as connection:
    connection.sendall((f'GET {path} HTTP/1.1\r\nHost: 127.0.0.1:4317\r\nOrigin: {base}\r\n'
        f'Upgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: {key}\r\n'
        'Sec-WebSocket-Version: 13\r\n\r\n').encode())
    header = b''
    while b'\r\n\r\n' not in header:
        byte = connection.recv(1)
        assert byte, 'WebSocket handshake closed before completing its headers'
        header += byte
        assert len(header) < 16384
    assert b' 101 ' in header.split(b'\r\n')[0], header
    # Masked ping from client; a real pong proves duplex forwarding, not only HTTP routing.
    mask = b'1234'
    connection.sendall(bytes([0x89, 0x82]) + mask + bytes([ord('o') ^ mask[0], ord('k') ^ mask[1]]))
    stream = connection.makefile('rb')
    assert stream.read(4) == b'\x8a\x02ok'
print('dashboard-card-api-assets-and-websocket-ok')
