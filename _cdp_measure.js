const { execSync, spawn } = require('child_process');
const fs = require('fs');
const WebSocket = require('ws');

const EDGE_PATH = 'C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe';
const CDP_PORT = 9224;
const TOKEN = 'GENERATE_AT_RUNTIME';

let msgId = 0;
const pending = new Map();

function send(ws, method, params = {}) {
  const id = ++msgId;
  ws.send(JSON.stringify({ id, method, params }));
  return new Promise((resolve, reject) => {
    pending.set(id, { resolve, reject });
    setTimeout(() => {
      if (pending.has(id)) { pending.delete(id); reject(new Error('CDP timeout: ' + method)); }
    }, 15000);
  });
}

async function run() {
  const edgeProc = spawn(EDGE_PATH, [
    '--headless=new', '--remote-debugging-port=' + CDP_PORT,
    '--no-first-run', '--disable-gpu', '--window-size=1440,900'
  ], { detached: true, stdio: 'ignore' });
  edgeProc.unref();
  console.error('Edge PID:', edgeProc.pid);

  for (let i = 0; i < 40; i++) {
    try { execSync(`curl -s -o nul http://localhost:${CDP_PORT}/json 2>nul`, { timeout: 1000 }); break; }
    catch(e) { await new Promise(r => setTimeout(r, 250)); }
  }

  const tabs = JSON.parse(execSync(`curl -s http://localhost:${CDP_PORT}/json`).toString());
  const pageTab = tabs.find(t => t.type === 'page');
  if (!pageTab) { console.error('No page tab found'); process.exit(1); }
  console.error('Target WS:', pageTab.webSocketDebuggerUrl);

  const ws = new WebSocket(pageTab.webSocketDebuggerUrl);
  ws.on('message', (data) => {
    const msg = JSON.parse(data.toString());
    if (msg.id && pending.has(msg.id)) {
      const { resolve, reject } = pending.get(msg.id);
      pending.delete(msg.id);
      if (msg.error) reject(msg.error); else resolve(msg.result);
    }
  });
  await new Promise((resolve, reject) => {
    ws.on('open', resolve); ws.on('error', reject);
    setTimeout(() => reject(new Error('WS timeout')), 10000);
  });
  console.error('WebSocket connected');

    await send(ws, 'Network.enable');
  await send(ws, 'Page.enable');

  // Generate JWT token dynamically
  const crypto = require('crypto');
  const secret = 'your_super_secret_jwt_key_here';
  function b64(buf) { return buf.toString('base64').replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, ''); }
  const now = Math.floor(Date.now() / 1000);
  const header = b64(Buffer.from(JSON.stringify({alg: 'HS256', typ: 'JWT'}), 'utf8'));
  const payload = b64(Buffer.from(JSON.stringify({id: 624, email: 'aylatsmorais@gmail.com', nome: 'Ayla', perfil: 1, exp: now + 86400}), 'utf8'));
  const data = header + '.' + payload;
  const sig = b64(crypto.createHmac('sha256', secret).update(data).digest());
  const dynamicToken = data + '.' + sig;
  console.error('Token exp:', now + 86400);

  await send(ws, 'Network.setCookie', { name: 'token', value: dynamicToken, domain: 'localhost', path: '/' });

  let navResolve;
  const navPromise = new Promise(r => { navResolve = r; });
  const navHandler = (data) => {
    const msg = JSON.parse(data.toString());
    if (msg.method === 'Page.frameStoppedLoading') { navResolve(); ws.removeListener('message', navHandler); }
  };
  ws.on('message', navHandler);
  await send(ws, 'Page.navigate', { url: 'http://localhost:8080/app' });
  await navPromise;
  console.error('Page loaded');
  await new Promise(r => setTimeout(r, 3000));

  const result = await send(ws, 'Runtime.evaluate', {
    expression: `((function() {
      const container = document.querySelector('.container.max-w-7xl');
      const buscar = document.querySelector('#buscar-cifras');
      const setlists = document.querySelector('#setlists-section');
      function chain(el) { const c=[]; let p=el.parentElement; while(p&&p!==document.body){const r=p.getBoundingClientRect();c.push({tag:p.tagName,cls:p.className.substring(0,100),w:Math.round(r.width),l:Math.round(r.left)});p=p.parentElement;} return c; }
      return {
        vp:{w:window.innerWidth,h:window.innerHeight},
        container: container?{w:Math.round(container.getBoundingClientRect().width),cls:container.className}:'NF',
        buscar: buscar?{w:Math.round(buscar.getBoundingClientRect().width),l:Math.round(buscar.getBoundingClientRect().left),cls:buscar.className,chain:chain(buscar)}:'NF',
        setlists: setlists?{w:Math.round(setlists.getBoundingClientRect().width),l:Math.round(setlists.getBoundingClientRect().left),cls:setlists.className,chain:chain(setlists)}:'NF'
      };
    })())`,
    returnByValue: true
  });

  console.log(JSON.stringify(result.result.value, null, 2));

  const scr = await send(ws, 'Page.captureScreenshot', { quality: 90 });
  fs.writeFileSync('_cdp_screenshot.png', Buffer.from(scr.data, 'base64'));
  console.error('Screenshot saved');

  ws.close(); try { edgeProc.kill(); } catch(e) {}
  process.exit(0);
}

run().catch(e => { console.error('ERROR:', e.message); process.exit(1); });