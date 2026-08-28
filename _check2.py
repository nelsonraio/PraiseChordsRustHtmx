import hmac, hashlib, base64, json, time, urllib.request, sys
sys.stdout.reconfigure(encoding='utf-8', errors='replace')

secret = b'your_super_secret_jwt_key_here'
def b64(b): return base64.urlsafe_b64encode(b).rstrip(b'=')

header = b64(json.dumps({"alg": "HS256", "typ": "JWT"}).encode())
payload = b64(json.dumps({
    "id": 624,
    "email": "aylatsmorais@gmail.com",
    "nome": "Ayla",
    "perfil": 1,
    "exp": int(time.time()) + 86400  # 24h expiry
}).encode())
sig = b64(hmac.new(secret, header + b'.' + payload, hashlib.sha256).digest())
token = (header + b'.' + payload + b'.' + sig).decode()

req = urllib.request.Request(
    'http://localhost:8080/app',
    headers={'Cookie': 'token=' + token, 'Cache-Control': 'no-store'}
)
resp = urllib.request.urlopen(req)
html = resp.read().decode('utf-8', errors='replace')

open('_served_v2.html', 'w', encoding='utf-8').write(html)
print('Status:', resp.status)
print('Length:', len(html))
print('buscar-cifras:', 'buscar-cifras' in html)
print('setlists-section:', 'setlists-section' in html)
print('bg-white/5:', 'bg-white/5' in html)
print('Cache-Control:', resp.headers.get('cache-control'))